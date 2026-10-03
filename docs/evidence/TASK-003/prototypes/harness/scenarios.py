"""TASK-003 scenarios: RC-T04, RC-T05, RC-T06, RC-T07, RC-T17 and the RC requirements they cover.

    python3 -B scenarios.py <section> <scratch>

<scratch> holds the PC identities (outside the repository: they include
private keys) and receives one JSON line per observation in scenarios.jsonl.
Sections run in order: setup, t05, capabilities, t07, t06, pc, startup; pc_launch runs
once, by the owner's approval. The app must be
installed and <scratch>/pc-state and <scratch>/intruder-state must hold PC
identities made with `nexees-host identity`.
"""

import hashlib
import json
import shutil
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

import device as d

HOST = "/mnt/F/Nexees-toolchains/target/release/nexees-host"
FIXTURES = Path(__file__).resolve().parent.parent / "feasibility-core" / "fixtures"
PHONE_ADDRESS = "127.0.0.1:47201"  # the emulator's redirect to the phone's port 47200
PIN = "1234"


class Run:
    def __init__(self, section, scratch):
        self.section = section
        self.scratch = Path(scratch)
        self.failures = 0
        self.started = time.strftime("%Y-%m-%dT%H:%M:%S%z")

    def record(self, name, expected, observed, passed):
        line = {"run": self.started, "section": self.section, "check": name, "expected": expected, "observed": observed,
                "passed": bool(passed), "at": time.strftime("%Y-%m-%dT%H:%M:%S%z")}
        with open(self.scratch / "scenarios.jsonl", "a") as handle:
            handle.write(json.dumps(line) + "\n")
        print(("PASS " if passed else "FAIL ") + name + " | " + json.dumps(observed)[:300])
        self.failures += 0 if passed else 1

    def send(self, action, to="phone-1", expires_in=60, identity="pc-state"):
        action_file = self.scratch / "action.json"
        action_file.write_text(json.dumps(action))
        out = subprocess.run(
            [HOST, "send", str(self.scratch / identity), PHONE_ADDRESS, str(self.scratch / "phone.cert.der"),
             "pc-1", to, "pc_to_phone", str(action_file), str(expires_in)],
            capture_output=True, text=True, timeout=60)
        if out.returncode != 0:
            return {"id": "", "reply": {"status": "error", "detail": out.stderr.strip()}}
        return json.loads(out.stdout)

    def status_of(self, command):
        return self.send({"kind": "status", "command": command})["reply"]


def write(content, revision, workspace="remote-fixture", path="greeting.lcl.txt"):
    return {"kind": "write_lcl_file", "workspace": workspace, "path": path, "content": content,
            "expected_revision": revision}


def sha256(text):
    return hashlib.sha256(text.encode()).hexdigest()


def open_app():
    d.shell(f"am start -W -n {d.PACKAGE}/.MainActivity")
    time.sleep(1)


def grant_is(on):
    """Taps the grant button only if the grant is not already in the wanted state."""
    open_app()
    wanted = "Revoke PC requests" if on else "Allow PC requests"
    if not d.find(wanted):
        d.tap("Allow PC requests" if on else "Revoke PC requests")
        d.wait_for(lambda: d.find(wanted), timeout=15, what=wanted)


def phone_revisions(scratch):
    """The saved revisions, read from a copy of the app's own database."""
    copy = Path(scratch) / "phone-db"
    shutil.rmtree(copy, ignore_errors=True)
    copy.mkdir()
    for suffix in ("", "-wal", "-shm"):
        data = subprocess.run([d.ADB, "-s", d.SERIAL, "exec-out", "run-as", d.PACKAGE, "cat", f"files/state.sqlite{suffix}"],
                              env=d.ENV, capture_output=True, timeout=60).stdout
        (copy / f"state.sqlite{suffix}").write_bytes(data)
    with sqlite3.connect(copy / "state.sqlite") as db:
        return db.execute("SELECT revision, sha256 FROM revisions WHERE workspace = 'remote-fixture' ORDER BY revision").fetchall()


def setup(run):
    """A fresh install state, standalone use in airplane mode, then the test pairing."""
    d.shell(f"pm clear {d.PACKAGE}")
    open_app()
    time.sleep(1)
    d.tap("Allow")
    granted = "POST_NOTIFICATIONS: granted=true" in d.shell(f"dumpsys package {d.PACKAGE}", timeout=60)
    run.record("setup.notifications_allowed_by_user", "granted by tapping Allow in Android's own dialog", granted, granted)

    d.shell("cmd connectivity airplane-mode enable")
    time.sleep(3)
    offline = d.shell("settings get global airplane_mode_on").strip() == "1"
    mark = d.mark("standalone-probe")
    d.tap("Run probe")
    line = d.wait_log(r"probe passed=", mark, timeout=300)
    name = sorted(d.run_as("ls files/probe").split())[-1].removesuffix(".json")
    report = json.loads(d.run_as(f"cat files/probe/{name}.json"))
    d.write_json(run.scratch / "probe-android-app-airplane.json", {"airplane_mode_on": offline, "report": report})
    run.record("standalone.probe_in_airplane_mode",
               "every check passes inside the app with no network: LCL, storage, ZIP import, JS test repair, help, transport",
               {"airplane_mode_on": offline, "log": line, "checks": len(report["checks"])}, offline and report["passed"])
    mark = d.mark("standalone-manual")
    d.tap("Open manual")
    line = d.wait_log(r"manual ", mark, timeout=60)
    run.record("standalone.manual_offline", "a chapter of the LCL user manual renders in the passive viewer offline",
               {"airplane_mode_on": offline, "log": line}, offline and "html_bytes" in line)
    d.shell("cmd connectivity airplane-mode disable")

    open_app()
    defaults = {"receiver_listening": d.receiver_listening(), "grant_button": d.find("Allow PC requests") is not None,
                "boot_button": d.find("Receiver at boot: off") is not None}
    run.record("setup.defaults_off", "after install the receiver is off, the PC grant is off and start at boot is off",
               defaults, not defaults["receiver_listening"] and defaults["grant_button"] and defaults["boot_button"])

    # The test stand-in for pairing (TASK-060 owns the real flow): the PC's
    # certificate goes into the app's private folder; the phone's comes back.
    d.adb("push", str(run.scratch / "pc-state/identity/identity.cert.der"), "/data/local/tmp/pc.cert.der")
    d.run_as("umask 077 && mkdir -p files/remote && cp /data/local/tmp/pc.cert.der files/remote/peer-pc.cert.der")
    d.shell("rm /data/local/tmp/pc.cert.der")
    d.tap("Start receiver")
    d.wait_for(d.receiver_listening, timeout=60, what="receiver")
    phone = subprocess.run([d.ADB, "-s", d.SERIAL, "exec-out", "run-as", d.PACKAGE, "cat",
                            "files/remote/identity/identity.cert.der"], env=d.ENV, capture_output=True, timeout=60).stdout
    (run.scratch / "phone.cert.der").write_bytes(phone)
    if "tcp:47201" not in d.adb("emu", "redir", "list"):
        d.adb("emu", "redir", "add", "tcp:47201:47200")
    run.record("setup.paired_and_receiving", "the receiver runs only after the user starts it; the PC reaches it through the emulator's redirect",
               {"phone_certificate_sha256": hashlib.sha256(phone).hexdigest(), "redirect": d.adb("emu", "redir", "list").split()},
               len(phone) > 0 and d.receiver_listening())


def t05(run):
    valid = (FIXTURES / "valid_task.lcl.txt").read_text()
    edited = valid.replace('NAME: "Phone-local feasibility fixture"', 'NAME: "Phone-local fixture, edited from the PC"')
    invalid = (FIXTURES / "invalid_task.lcl.txt").read_text()

    grant_is(False)
    r = run.send(write(valid, 0))["reply"]
    run.record("t05.grant_absent_denied", "denied", r, r["status"] == "denied")

    grant_is(True)
    r = run.send(write(valid, 0))["reply"]
    run.record("t05.create_completed", "completed, revision 1, the sent bytes, LCL accepted", r,
               r["status"] == "completed" and r.get("revision") == 1 and r.get("sha256") == sha256(valid)
               and r.get("validation", {}).get("outcome") == "accepted")
    r = run.send(write(edited, 1))["reply"]
    run.record("t05.edit_completed", "completed, revision 2, LCL accepted", r,
               r["status"] == "completed" and r.get("revision") == 2 and r.get("sha256") == sha256(edited)
               and r.get("validation", {}).get("outcome") == "accepted")
    on_phone = d.run_as("cat files/workspaces/remote-fixture/greeting.lcl.txt")
    revisions = phone_revisions(run.scratch)
    run.record("t05.saved_revision_inspected", "the phone's file and its database hold revision 2 with the edit",
               {"file_sha256": sha256(on_phone), "revisions": revisions},
               sha256(on_phone) == sha256(edited) and revisions == [(1, sha256(valid)), (2, sha256(edited))])
    r = run.send(write(valid, 1))["reply"]
    run.record("t05.stale_edit_conflict", "conflict; nothing written", r, r["status"] == "conflict")
    r = run.send(write(invalid, 2))["reply"]
    run.record("t05.invalid_edit_reported", "saved as revision 3 with LCL's rejection reported, not hidden", r,
               r["status"] == "completed" and r.get("revision") == 3
               and r.get("validation", {}).get("primary") == "error.reference.unresolved at 28:17")
    r = run.send(write(valid, 3), to="phone-2")["reply"]
    run.record("t05.wrong_device_denied", "denied", r, r["status"] == "denied")
    r = run.send(write(valid, 3), identity="intruder-state")["reply"]
    run.record("t05.unpaired_pc_rejected", "denied at the TLS handshake", r,
               r["status"] == "denied" and "rejected by TLS" in r["detail"])
    r = run.send(write(valid, 3), expires_in=-5)["reply"]
    run.record("t05.expired_refused", "expired", r, r["status"] == "expired")

    grant_is(False)
    r = run.send(write(valid, 3))["reply"]
    run.record("t05.revoked_denied", "denied", r, r["status"] == "denied")
    run.record("t05.revoke_kept_revisions", "revoking stops new requests and keeps saved work",
               phone_revisions(run.scratch), len(phone_revisions(run.scratch)) == 3)


def capabilities(run):
    grant_is(True)
    r = run.send({"kind": "capabilities"})["reply"]
    listed = {c["action"]: c["availability"] for c in r.get("capabilities", [])}
    run.record("rc14.capabilities", "per-action availability: write available, settings needs the user, maps and screen input unsupported",
               listed, r["status"] == "completed" and listed.get("write_lcl_file") == "available"
               and listed.get("launch_app:settings") == "needs_user_action"
               and listed.get("launch_app:maps") == "unsupported" and listed.get("screen_input") == "unsupported")
    r = run.send({"kind": "launch_app", "app": "maps"})["reply"]
    run.record("rc14.unsupported_app", "unsupported, nothing opened", r, r["status"] == "unsupported")


def wait_status(run, command, wanted, timeout=20):
    try:
        return d.wait_for(lambda: (lambda s: s if s["status"] in wanted else None)(run.status_of(command)),
                          timeout=timeout, what=f"status in {wanted}")
    except TimeoutError:
        return run.status_of(command)


def home():
    d.shell("input keyevent KEYCODE_HOME")
    time.sleep(2)


def keyguard_showing():
    return "isKeyguardShowing=true" in d.shell("dumpsys window", timeout=60)


def enter_pin():
    """Types the test PIN on the lock screen's own pad, as the phone user would."""
    d.wait_for(lambda: d.find("Enter"), timeout=20, what="PIN pad")
    for digit in PIN:
        d.tap(digit, attempts=1)
    d.tap("Enter", attempts=1)
    d.wait_for(lambda: not keyguard_showing(), timeout=20, what="unlocked")


def unlock_and_clear_pin():
    d.shell("input keyevent KEYCODE_WAKEUP")
    time.sleep(1)
    d.shell("wm dismiss-keyguard")
    enter_pin()
    d.shell(f"locksettings clear --old {PIN}")


def dismiss_not_responding():
    """The software-rendered emulator can be slow enough after boot for Android to ask about System UI."""
    if d.find("Wait"):
        d.tap("Wait", attempts=1)
        time.sleep(3)


def tap_notification(title):
    d.shell("cmd statusbar expand-notifications")
    time.sleep(2)
    d.tap(title, exact=False, attempts=3)
    time.sleep(3)


def launch_in_background(run, mark_label, expires_in=120):
    home()
    mark = d.mark(mark_label)
    sent = run.send({"kind": "launch_app", "app": "settings"}, expires_in=expires_in)
    state = wait_status(run, sent["id"], {"needs_user_action", "completed", "denied", "expired"})
    blocked = [l for l in d.adb("logcat", "-d").splitlines()[-4000:]
               if "dev.nexees.feasibility" in l and ("Background activity launch blocked" in l or "Abort background activity starts" in l or "BAL_BLOCK" in l)]
    return sent, state, blocked, mark


def t07(run):
    grant_is(True)
    open_app()
    sent = run.send({"kind": "launch_app", "app": "settings"})
    state = wait_status(run, sent["id"], {"completed", "needs_user_action"})
    resumed = d.resumed_activity()
    run.record("t07.foreground_launch", "completed while Nexees is in the foreground; Settings resumed",
               {"reply": sent["reply"], "state": state, "resumed": resumed},
               sent["reply"]["status"] == "accepted" and state["status"] == "completed" and "settings" in resumed.lower())

    sent, state, blocked, _ = launch_in_background(run, "t07-background")
    in_shade = "PC asks to open settings" in d.notifications()
    run.record("t07.background_needs_user_action",
               "Android blocks the background start; the request becomes needs_user_action with a notification",
               {"reply": sent["reply"], "state": state, "os_block_log": blocked[-2:], "notification": in_shade},
               state["status"] == "needs_user_action" and in_shade and d.resumed_activity().find("feasibility") < 0)
    tap_notification("PC asks to open settings")
    run.record("t07.notification_opens_review_only", "the tap opens the review screen; nothing is opened yet",
               {"state": run.status_of(sent["id"]), "on_screen": [t for t, _, _ in d.screen()][:4]},
               run.status_of(sent["id"])["status"] == "needs_user_action" and d.find("Continue") is not None)
    d.tap("Continue")
    time.sleep(3)
    state = run.status_of(sent["id"])
    resumed = d.resumed_activity()
    run.record("t07.continue_completes", "completed after the user continued; Settings resumed",
               {"state": state, "resumed": resumed}, state["status"] == "completed" and "settings" in resumed.lower())

    sent, state, _, _ = launch_in_background(run, "t07-expiry", expires_in=6)
    time.sleep(8)
    tap_notification("PC asks to open settings")
    d.tap("Continue")
    time.sleep(3)
    state = run.status_of(sent["id"])
    resumed = d.resumed_activity()
    run.record("t07.expired_on_continue", "expired is rechecked when the user continues; nothing opened",
               {"state": state, "resumed": resumed}, state["status"] == "expired" and "settings" not in resumed.lower())

    sent, state, _, _ = launch_in_background(run, "t07-revoked")
    grant_is(False)
    tap_notification("PC asks to open settings")
    d.tap("Continue")
    time.sleep(3)
    # The PC can ask for the outcome only while the grant is on again.
    grant_is(True)
    state = run.status_of(sent["id"])
    resumed = d.resumed_activity()
    run.record("t07.revoked_on_continue", "denied: the grant is rechecked when the user continues",
               {"state": state, "resumed": resumed}, state["status"] == "denied" and "settings" not in resumed.lower())

    sent, state, _, _ = launch_in_background(run, "t07-decline")
    tap_notification("PC asks to open settings")
    d.tap("Decline")
    time.sleep(2)
    state = run.status_of(sent["id"])
    run.record("t07.decline", "denied by the phone user", state, state["status"] == "denied")

    d.shell(f"locksettings set-pin {PIN}")
    sent, state, _, _ = launch_in_background(run, "t07-locked")
    d.shell("input keyevent KEYCODE_SLEEP")
    time.sleep(2)
    d.shell("input keyevent KEYCODE_WAKEUP")
    time.sleep(2)
    locked = keyguard_showing()
    try:
        tap_notification("PC asks to open settings")
    except RuntimeError:
        pass
    before_unlock = run.status_of(sent["id"])
    on_screen = [t for t, _, _ in d.screen()][:8]
    run.record("t07.locked_waits", "while locked the request stays needs_user_action; Android asks for the PIN first",
               {"locked": locked, "state": before_unlock, "on_screen": on_screen},
               locked and before_unlock["status"] == "needs_user_action" and "Enter" in on_screen + [t for t, _, _ in d.screen()])
    enter_pin()
    time.sleep(3)
    if not d.find("Continue"):
        tap_notification("PC asks to open settings")
    d.tap("Continue")
    time.sleep(3)
    state = run.status_of(sent["id"])
    run.record("t07.unlocked_then_continue", "completed only after unlock and the user's continue",
               {"state": state, "resumed": d.resumed_activity()}, state["status"] == "completed")
    d.shell(f"locksettings clear --old {PIN}")

def t06(run):
    """Receiver availability across Android lifecycle states, as observed."""
    valid = (FIXTURES / "valid_task.lcl.txt").read_text()
    revision = [0]

    def probe_write(label):
        content = valid.replace('NAME: "Phone-local feasibility fixture"', f'NAME: "Lifecycle check: {label}"')
        reply = run.send(write(content, revision[0], workspace="lifecycle", path="state.lcl.txt"))["reply"]
        if reply["status"] == "completed":
            revision[0] = reply["revision"]
        return reply

    def ok(reply):
        return reply["status"] == "completed" and reply.get("validation", {}).get("outcome") == "accepted"

    def service():
        out = d.foreground_services()
        return {"process": d.app_pid() or None, "listening": d.receiver_listening(),
                "foreground_service": "isForeground=true" in out, "connected_device_type": "types=0x00000010" in out}

    def kept():
        return len(phone_revisions(run.scratch))

    def reboot():
        d.adb("reboot")
        time.sleep(10)
        d.adb("wait-for-device", timeout=300)
        d.wait_for(lambda: d.shell("getprop sys.boot_completed", check=False).strip() == "1", timeout=300, every=3,
                   what="boot")
        time.sleep(20)
        d.shell("input keyevent KEYCODE_WAKEUP")
        d.shell("wm dismiss-keyguard")
        time.sleep(3)
        dismiss_not_responding()

    grant_is(True)
    if not d.receiver_listening():
        d.tap("Start receiver")
        d.wait_for(d.receiver_listening, timeout=30, what="receiver")
    r = probe_write("foreground")
    run.record("t06.foreground", "completed", {"reply": r, "service": service()}, ok(r))

    home()
    r = probe_write("background")
    shown = "Nexees receiver is running" in d.notifications()
    state = service()
    run.record("t06.background_with_notification",
               "completed; the connectedDevice foreground service and its stop-able notification stay visible",
               {"reply": r, "service": state, "notification": shown}, ok(r) and shown and state["connected_device_type"])

    d.shell(f"locksettings set-pin {PIN}")
    d.shell("input keyevent KEYCODE_SLEEP")
    time.sleep(3)
    locked = keyguard_showing()
    r = probe_write("locked")
    run.record("t06.screen_locked", "completed while the screen is off and locked", {"locked": locked, "reply": r},
               locked and ok(r))

    d.shell("dumpsys deviceidle enable deep")
    d.shell("dumpsys battery unplug")
    d.shell("dumpsys deviceidle force-idle deep")
    idle = d.shell("dumpsys deviceidle get deep").strip()
    r = probe_write("doze")
    run.record("t06.doze", "observed in forced deep Doze with the foreground service running",
               {"deep_idle": idle, "reply": r}, idle == "IDLE" and r["status"] in ("completed", "unavailable", "outcome_unknown"))
    d.shell("dumpsys deviceidle unforce")
    d.shell("dumpsys deviceidle disable")
    d.shell("dumpsys battery reset")
    unlock_and_clear_pin()

    d.shell("cmd connectivity airplane-mode enable")
    time.sleep(4)
    r = probe_write("network loss")
    run.record("t06.network_loss", "unavailable while the phone has no network; nothing claims success",
               {"reply": r, "service": service()}, r["status"] in ("unavailable", "outcome_unknown") and service()["listening"])
    d.shell("cmd connectivity airplane-mode disable")
    try:
        r = d.wait_for(lambda: (lambda x: x if ok(x) else None)(probe_write("network back")), timeout=60, every=3,
                       what="network back")
    except TimeoutError:
        r = probe_write("network back")
    run.record("t06.network_restored", "completed again once the network returns", r, ok(r))

    pid = d.app_pid()
    mark = d.mark("t06-kill")
    d.run_as(f"kill -9 {pid}")
    try:
        restarted = d.wait_log(r"restarted by Android: true", mark, timeout=90)
        d.wait_for(d.receiver_listening, timeout=30, what="receiver after restart")
    except TimeoutError:
        restarted = None
    r = probe_write("after process death")
    run.record("t06.process_killed", "observed: Android restarts the sticky service after process death; the receiver resumes",
               {"killed_pid": pid, "new_pid": d.app_pid() or None, "android_restart_log": restarted, "reply": r},
               restarted is not None and ok(r))

    d.shell(f"am force-stop {d.PACKAGE}")
    time.sleep(5)
    r = probe_write("after force-stop")
    state = service()
    run.record("t06.force_stopped", "unavailable; Android does not restart a force-stopped app and neither does Nexees",
               {"reply": r, "service": state}, r["status"] == "unavailable" and state["process"] is None and not state["listening"])
    open_app()
    d.tap("Start receiver")
    d.wait_for(d.receiver_listening, timeout=30, what="receiver")
    r = probe_write("user restarted")
    run.record("t06.user_restarts", "completed after the phone user starts the receiver again; saved work kept",
               {"reply": r, "remote_fixture_revisions": kept()}, ok(r) and kept() == 3)

    reboot()
    r = probe_write("after reboot, start at boot off")
    run.record("t06.reboot_default", "unavailable after reboot until the phone user starts the receiver (start at boot is off by default)",
               {"reply": r, "service": service(), "boot_log": [l for l in d.log_lines() if "boot completed" in l][-1:]},
               r["status"] == "unavailable" and not service()["listening"])
    open_app()
    run.record("t06.reboot_state_kept", "saved revisions and the grant survive the reboot",
               {"remote_fixture_revisions": kept(), "grant_revocable": d.find("Revoke PC requests") is not None},
               kept() == 3 and d.find("Revoke PC requests") is not None)

    d.tap("Receiver at boot: off")
    d.wait_for(lambda: d.find("Receiver at boot: on"), timeout=10, what="boot flag on")
    reboot()
    try:
        d.wait_for(d.receiver_listening, timeout=60, what="receiver after boot")
    except TimeoutError:
        pass
    r = probe_write("after reboot, start at boot on")
    boot_log = [l for l in d.log_lines() if "boot" in l or "receiver_start" in l or "refused" in l][-4:]
    run.record("t06.reboot_start_at_boot", "observed: with start at boot turned on by the phone user, Android lets the receiver start after reboot",
               {"reply": r, "service": service(), "log": boot_log}, ok(r))
    open_app()
    d.tap("Receiver at boot: on")
    d.wait_for(lambda: d.find("Receiver at boot: off"), timeout=10, what="boot flag off")

CORE = ("/mnt/F/LCL/canonical/LCL_Core_0.1.0", "/mnt/F/LCL/canonical/LCL_Core_0.3.0")
HOPTODESK = "com.hoptodesk.HopToDesk"


def hoptodesk():
    """Running HopToDesk Flatpak instances and whether KWin lists its window."""
    out = subprocess.run(["flatpak", "ps", "--columns=instance,pid,application"], capture_output=True, text=True).stdout
    instances = sorted(line.split()[0] for line in out.splitlines() if line.strip().endswith(HOPTODESK))
    windows = subprocess.run(["gdbus", "call", "--session", "--dest", "org.kde.KWin", "--object-path", "/WindowsRunner",
                              "--method", "org.kde.krunner1.Match", "HopToDesk"], capture_output=True, text=True).stdout
    return {"instances": instances, "window_listed": "'HopToDesk'" in windows}


class PcReceiver:
    """The Desktop receiver, run as the PC user in a chosen environment."""

    def __init__(self, run, env=None, allow_uid=None):
        self.state = run.scratch / "pc-state"
        extra = ["--allow-uid", str(allow_uid)] if allow_uid is not None else []
        self.log = open(run.scratch / "pc-receiver.log", "a")
        self.process = subprocess.Popen(
            [HOST, "receive", str(self.state), "127.0.0.1:47100", str(run.scratch / "phone.cert.der"), *CORE, *extra],
            stdout=self.log, stderr=self.log, env=env)
        # A socket file can be left by the previous receiver; wait for an answer.
        d.wait_for(lambda: self.ipc('{"op": "status"}').get("status") in ("completed", "denied"), timeout=30,
                   what="the receiver to answer")

    def ipc(self, message):
        out = subprocess.run([HOST, "ipc", str(self.state), message], capture_output=True, text=True, timeout=30)
        return json.loads(out.stdout) if out.returncode == 0 else {"status": "error", "detail": out.stderr.strip()}

    def stop(self):
        if self.ipc('{"op": "stop"}').get("status") != "completed":
            self.process.terminate()
        self.process.wait(timeout=30)
        self.log.close()


def ask_pc():
    """The phone user taps the button that asks the PC to open HopToDesk."""
    open_app()
    mark = d.mark("ask-pc")
    d.tap("Ask PC to open HopToDesk")
    line = d.wait_log(r" send \{", mark, timeout=90)
    return json.loads(line.split(" send ", 1)[1])["reply"]


def pc(run):
    import os
    import stat
    grant_is(True)
    # A fresh PC receiver state; the PC identity paired into the phone stays.
    for stale in (run.scratch / "pc-state").glob("state.sqlite*"):
        stale.unlink()
    receiver = PcReceiver(run)
    mode = lambda path: oct(stat.S_IMODE(os.stat(path).st_mode))
    run.record("rc23.local_ipc_private", "state folder 0700 and IPC socket 0600, owned by the PC user",
               {"state": mode(receiver.state), "socket": mode(receiver.state / "ipc.sock")},
               mode(receiver.state) == "0o700" and mode(receiver.state / "ipc.sock") == "0o600")
    status = receiver.ipc('{"op": "status"}')
    run.record("rc23.peer_checked_ipc", "the receiver's own user is served; the phone_to_pc grant is off by default",
               status, status.get("status") == "completed" and status.get("phone_to_pc") is False)
    big = receiver.ipc(json.dumps({"op": "status", "padding": "x" * 5000}))
    run.record("rc23.bounded_ipc_message", "a message over 4096 bytes is refused", big, big.get("status") == "invalid")
    wrong = receiver.ipc('{"op": "grant", "direction": "pc_to_phone", "enabled": true}')
    run.record("rc23.pc_holds_only_its_own_grant", "the PC cannot set the phone's grant", wrong, wrong.get("status") == "denied")

    before = hoptodesk()
    reply = ask_pc()
    phone_side = run.send({"kind": "capabilities"})["reply"]["status"]
    run.record("rc.directional_grants_independent",
               "phone grant on and PC grant off: phone-to-PC is denied while PC-to-phone still works",
               {"phone_to_pc": reply, "pc_to_phone": phone_side, "hoptodesk_unchanged": hoptodesk() == before},
               reply["status"] == "denied" and phone_side == "completed" and hoptodesk() == before)
    receiver.stop()

    other = PcReceiver(run, allow_uid=4242)
    refused = other.ipc('{"op": "status"}')
    run.record("rc23.wrong_peer_uid_denied", "a peer whose kernel-reported uid is not the allowed one is denied",
               refused, refused.get("status") == "denied" and "is not uid 4242" in refused.get("detail", ""))
    other.stop()

    bare = {"PATH": os.environ["PATH"], "HOME": os.environ["HOME"]}
    headless = PcReceiver(run, env=bare)
    granted = headless.ipc('{"op": "grant", "direction": "phone_to_pc", "enabled": true}')
    before = hoptodesk()
    reply = ask_pc()
    run.record("rct04.no_graphical_session_unsupported",
               "outside a graphical login session the launch is unsupported and nothing starts",
               {"grant": granted, "reply": reply, "hoptodesk_unchanged": hoptodesk() == before},
               reply["status"] == "unsupported" and hoptodesk() == before)
    headless.stop()

    receiver = PcReceiver(run)
    permissions = subprocess.run(["flatpak", "info", "--show-permissions", HOPTODESK], capture_output=True, text=True).stdout
    portal = subprocess.run(["flatpak", "permission-show", HOPTODESK], capture_output=True, text=True).stdout
    run.record("rct17.separate_permission_status",
               "HopToDesk's own permissions are separate: read without changing them",
               {"flatpak_permissions": permissions.split(), "portal_tables": sorted({l.split()[0] for l in portal.splitlines() if l.strip()}),
                "screen_cast_or_remote_desktop_grant": any(t in portal for t in ("screencast", "remote-desktop"))},
               "sockets=" in permissions)
    revoked = receiver.ipc('{"op": "grant", "direction": "phone_to_pc", "enabled": false}')
    before = hoptodesk()
    reply = ask_pc()
    run.record("rc.pc_grant_revoked_denied", "after the PC user revokes, the phone's request is denied",
               {"revoke": revoked, "reply": reply}, reply["status"] == "denied" and hoptodesk() == before)
    receiver.stop()
    reply = ask_pc()
    run.record("rc06.no_receiver_unavailable", "with no receiver running the phone reports unavailable",
               reply, reply["status"] == "unavailable")


def pc_launch(run):
    """The one HopToDesk launch the owner approved for TASK-003: run it once.

    It needs the phone_to_pc grant on. Close afterwards only an instance this
    launch started, by its Flatpak instance ID; never the owner's own one.
    """
    granter = PcReceiver(run)
    granter.ipc('{"op": "grant", "direction": "phone_to_pc", "enabled": true}')
    granter.stop()
    receiver = PcReceiver(run)
    before = hoptodesk()
    reply = ask_pc()
    time.sleep(3)
    after = hoptodesk()
    started = sorted(set(after["instances"]) - set(before["instances"]))
    run.record("rct17.allowlisted_launch",
               "one launch through the desktop entry; the state reports what was observed, not a window or screen control",
               {"before": before, "reply": reply, "after": after, "new_instances": started},
               reply["status"] in ("completed", "running") and "does not establish screen control" in reply["detail"])

    receiver.stop()
    return started


def startup(run):
    import os
    profile = run.scratch / "startup-profile"
    shutil.rmtree(profile, ignore_errors=True)
    config = profile / "config"
    real = Path.home() / ".config/autostart"
    real_before = sorted(os.listdir(real)) if real.exists() else []
    linger = lambda: subprocess.run(["loginctl", "show-user", str(os.getuid()), "-p", "Linger"], capture_output=True, text=True).stdout.strip()
    linger_before = linger()
    host = lambda *args: subprocess.run([HOST, "startup", *args], capture_output=True, text=True)

    status = json.loads(host("status", str(config)).stdout)
    run.record("rct04.off_by_default", "nothing is registered until the user enables it", status, status["enabled"] is False)
    command = [HOST, "receive", str(run.scratch / "pc-state"), "127.0.0.1:47100", str(run.scratch / "phone.cert.der"), *CORE]
    enabled = host("enable", str(config), *command)
    entry = config / "autostart/dev.nexees.Receiver.desktop"
    valid = subprocess.run(["desktop-file-validate", str(entry)], capture_output=True, text=True)
    run.record("rct04.enable_isolated", "a valid XDG autostart entry in the isolated profile only",
               {"enable": enabled.stdout.strip(), "validate_exit": valid.returncode, "validate_output": valid.stdout + valid.stderr},
               enabled.returncode == 0 and entry.is_file() and valid.returncode == 0)

    def generate():
        out = profile / "generated"
        shutil.rmtree(out, ignore_errors=True)
        out.mkdir(parents=True)
        env = {"PATH": os.environ["PATH"], "XDG_CONFIG_HOME": str(config), "XDG_CONFIG_DIRS": str(profile / "no-system-dirs")}
        subprocess.run(["/usr/lib/systemd/user-generators/systemd-xdg-autostart-generator", str(out), str(out), str(out)],
                       env=env, capture_output=True, text=True)
        return {p.name: p.read_text() for p in out.rglob("*.service")}

    units = generate()
    unit = next(iter(units.values()), "")
    run.record("rct04.session_lifetime", "systemd's own generator turns it into a unit that starts after graphical login and ends with the session",
               {"units": list(units), "unit": [l for l in unit.splitlines() if l.split("=")[0] in ("ExecStart", "PartOf", "After", "Slice", "ExitType")]},
               len(units) == 1 and "PartOf=graphical-session.target" in unit and "After=graphical-session.target" in unit)
    disabled = host("disable", str(config))
    run.record("rct04.disable", "disabled: the entry is gone and the generator makes no unit",
               {"disable": disabled.stdout.strip(), "units": list(generate())}, not entry.exists() and not generate())
    relative = host("enable", str(config), "nexees-host", "receive")
    shell_string = host("enable", str(config), "/bin/sh", "-c", "nexees-host;reboot")
    run.record("rct04.no_shell_or_relative_command", "a relative command or a shell string is refused",
               {"relative": relative.stderr.strip(), "shell": shell_string.stderr.strip()},
               relative.returncode != 0 and shell_string.returncode != 0 and not entry.exists())
    real_after = sorted(os.listdir(real)) if real.exists() else []
    units_in_session = subprocess.run(["systemctl", "--user", "list-units", "--all", "--no-legend", "*nexees*"], capture_output=True, text=True).stdout
    run.record("rct04.real_session_untouched", "the real autostart folder, user units and linger are unchanged",
               {"autostart_unchanged": real_before == real_after, "nexees_units": units_in_session.strip(),
                "linger_before": linger_before, "linger_after": linger()},
               real_before == real_after and not units_in_session.strip() and linger_before == linger())


def main():
    section, scratch = sys.argv[1], sys.argv[2]
    run = Run(section, scratch)
    {"setup": setup, "t05": t05, "capabilities": capabilities, "t07": t07, "t06": t06, "pc": pc,
     "pc_launch": pc_launch, "startup": startup}[section](run)
    print(f"{section}: {run.failures} failed")
    sys.exit(1 if run.failures else 0)


if __name__ == "__main__":
    main()
