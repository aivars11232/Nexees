"""Drives the disposable TASK-003 emulator for the evidence scenarios.

Every action goes through the isolated adb server named by
/mnt/F/Nexees-toolchains/env.sh. UI actions are real taps found through
uiautomator, the way a phone user would press the button; nothing calls into
the app behind its UI except the debug-only `run-as` reads of its own files.
"""

import json
import os
import re
import subprocess
import time
import xml.etree.ElementTree as ElementTree

TOOLCHAINS = "/mnt/F/Nexees-toolchains"
PACKAGE = "dev.nexees.feasibility"
SERIAL = "emulator-5580"
ENV = dict(os.environ, ANDROID_ADB_SERVER_PORT="5039", ANDROID_USER_HOME=f"{TOOLCHAINS}/android-user-home",
           HOME=f"{TOOLCHAINS}/home")
ADB = f"{TOOLCHAINS}/android-sdk/platform-tools/adb"


def adb(*args, check=True, timeout=120):
    result = subprocess.run([ADB, "-s", SERIAL, *args], env=ENV, capture_output=True, text=True, timeout=timeout)
    if check and result.returncode != 0:
        raise RuntimeError(f"adb {' '.join(args)} failed: {result.stderr.strip()}")
    return result.stdout


def shell(command, check=True, timeout=120):
    return adb("shell", command, check=check, timeout=timeout)


def run_as(command):
    """Runs `command` as the app's user, which only a debuggable build allows."""
    return shell(f"run-as {PACKAGE} sh -c '{command}'")


def screen():
    """Every node on screen with text, as (text, centre x, centre y)."""
    shell("uiautomator dump /data/local/tmp/ui.xml")
    tree = ElementTree.fromstring(shell("cat /data/local/tmp/ui.xml"))
    nodes = []
    for node in tree.iter("node"):
        text = node.get("text") or node.get("content-desc") or ""
        bounds = re.findall(r"\d+", node.get("bounds", ""))
        if text and len(bounds) == 4:
            x1, y1, x2, y2 = map(int, bounds)
            nodes.append((text, (x1 + x2) // 2, (y1 + y2) // 2))
    return nodes


def find(text, exact=True):
    for label, x, y in screen():
        # Button labels are drawn in capitals, so text compares without case.
        if (label.casefold() == text.casefold()) if exact else (text.casefold() in label.casefold()):
            return x, y
    return None


def tap(text, exact=True, attempts=10):
    """Taps the first node showing `text`, scrolling down if it is off screen."""
    for _ in range(attempts):
        position = find(text, exact)
        if position:
            shell(f"input tap {position[0]} {position[1]}")
            return True
        shell("input swipe 540 1800 540 900 300")
        time.sleep(1)
    raise RuntimeError(f"no '{text}' on screen")


def wait_for(predicate, timeout=60, every=1.0, what="condition"):
    deadline = time.time() + timeout
    while time.time() < deadline:
        value = predicate()
        if value:
            return value
        time.sleep(every)
    raise TimeoutError(f"timed out waiting for {what}")


def log_lines(since_mark=None):
    """The app's log lines, optionally only those after a mark line."""
    lines = adb("logcat", "-d", "-s", "NexeesFeasibility:I").splitlines()
    if since_mark:
        indexes = [i for i, line in enumerate(lines) if since_mark in line]
        lines = lines[indexes[-1] + 1:] if indexes else []
    return lines


def mark(label):
    """Writes a unique line to logcat so later reads start after it."""
    token = f"MARK-{label}-{time.time_ns()}"
    shell(f"log -p i -t NexeesFeasibility {token}")
    return token


def wait_log(pattern, since_mark, timeout=60):
    regex = re.compile(pattern)
    return wait_for(lambda: next((l for l in log_lines(since_mark) if regex.search(l)), None), timeout,
                    what=f"log /{pattern}/")


def app_pid():
    return shell(f"pidof {PACKAGE}", check=False).strip()


def resumed_activity():
    out = shell("dumpsys activity activities", timeout=60)
    found = re.search(r"(?:topResumedActivity|mResumedActivity|ResumedActivity)[:=].*?\{[^}]*? ([\w.]+/[\w.$]+)", out)
    return found.group(1) if found else ""


def notifications():
    return shell("dumpsys notification --noredact", timeout=60)


def foreground_services():
    return shell(f"dumpsys activity services {PACKAGE}", timeout=60)


def receiver_listening(port=47200):
    """Whether something in the guest listens on the receiver port."""
    out = shell("cat /proc/net/tcp /proc/net/tcp6", check=False)
    return any(f":{port:04X} " in line and " 0A " in line for line in out.splitlines())


def write_json(path, value):
    with open(path, "w") as handle:
        json.dump(value, handle, indent=2)
        handle.write("\n")
