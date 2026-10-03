"""Lists every resolved package of the TASK-003 prototypes with its declared
license, and looks each public name@version up in the OSV database.

    python3 -B advisories.py <package-lock.json> <Cargo.lock> <output.json>

Only public package coordinates are sent (OSV's querybatch API); no project
code, path or lockfile leaves the machine.
"""

import json
import sys
import tomllib
import urllib.request
from collections import Counter
from pathlib import Path

OSV = "https://api.osv.dev/v1"


def post(url, body):
    request = urllib.request.Request(url, json.dumps(body).encode(), {"Content-Type": "application/json"})
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)


def get(url):
    with urllib.request.urlopen(url, timeout=60) as response:
        return json.load(response)


def npm_packages(lock_path):
    lock = json.loads(Path(lock_path).read_text())
    found = {}
    for path, entry in lock["packages"].items():
        if not path or "version" not in entry:
            continue
        name = entry.get("name") or path.rsplit("node_modules/", 1)[-1]
        manifest = Path(lock_path).parent / path / "package.json"
        license_ = entry.get("license")
        if license_ is None and manifest.exists():
            declared = json.loads(manifest.read_text())
            # Older packages declare a "licenses" list instead of "license".
            license_ = declared.get("license") or " OR ".join(
                item.get("type", "") for item in declared.get("licenses") or [] if isinstance(item, dict)) or None
        found[(name, entry["version"])] = {"ecosystem": "npm", "name": name, "version": entry["version"],
                                           "license": license_ if isinstance(license_, str) else json.dumps(license_)}
    return list(found.values())


def crates(lock_path, registry):
    lock = tomllib.loads(Path(lock_path).read_text())
    found = []
    for package in lock["package"]:
        if "source" not in package:
            continue  # path dependencies: this crate and the owner's LCL engine
        manifest = next(Path(registry).glob(f"*/{package['name']}-{package['version']}/Cargo.toml"), None)
        license_ = tomllib.loads(manifest.read_text()).get("package", {}).get("license") if manifest else None
        found.append({"ecosystem": "crates.io", "name": package["name"], "version": package["version"],
                      "license": license_})
    return found


def advisories(packages):
    results = []
    for start in range(0, len(packages), 1000):
        batch = packages[start:start + 1000]
        queries = [{"package": {"name": p["name"], "ecosystem": p["ecosystem"]}, "version": p["version"]} for p in batch]
        results += post(f"{OSV}/querybatch", {"queries": queries})["results"]
    found = []
    for package, result in zip(packages, results):
        for vuln in result.get("vulns", []):
            detail = get(f"{OSV}/vulns/{vuln['id']}")
            found.append({"package": f"{package['ecosystem']}:{package['name']}@{package['version']}", "id": vuln["id"],
                          "aliases": detail.get("aliases", []), "summary": detail.get("summary", ""),
                          "severity": detail.get("database_specific", {}).get("severity"),
                          "withdrawn": detail.get("withdrawn")})
    return found


def main():
    npm_lock, cargo_lock, output = sys.argv[1:4]
    registry = "/mnt/F/Nexees-toolchains/cargo/registry/src"
    packages = npm_packages(npm_lock) + crates(cargo_lock, registry)
    found = advisories(packages)
    report = {
        "queried": {ecosystem: sum(1 for p in packages if p["ecosystem"] == ecosystem) for ecosystem in ("npm", "crates.io")},
        "licenses": {ecosystem: Counter(p["license"] or "(none declared)" for p in packages if p["ecosystem"] == ecosystem).most_common()
                     for ecosystem in ("npm", "crates.io")},
        "advisories": found,
        "packages": packages,
    }
    Path(output).write_text(json.dumps(report, indent=1) + "\n")
    print(json.dumps({k: report[k] for k in ("queried", "advisories")}, indent=1))


if __name__ == "__main__":
    main()
