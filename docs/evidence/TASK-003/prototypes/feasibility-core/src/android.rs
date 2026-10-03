//! The JNI surface the Android prototype app calls. One entry point takes a
//! JSON request and returns a JSON reply, so the Kotlin side holds no logic
//! of its own beyond Android lifecycle and UI.
//!
//! Paths are under the app's private files folder, which the app passes in
//! as `files`; the app extracts its bundled assets (the LCL Core packages, the
//! fixtures and the LCL user manual) to `files/assets` before calling in.

use crate::lcl_bridge::Lcl;
use crate::probe::{self, Config};
use crate::remote::{self, Action, Capability, Device, Direction, Envelope, Identity, Platform, Status};
use crate::store::Store;
use jni::objects::{JClass, JString};
use jni::EnvUnowned;
use serde_json::{json, Value};
use std::collections::VecDeque;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

const THIS_DEVICE: &str = "phone-1";
const PAIRED_PC: &str = "pc-1";

struct Receiver {
    port: u16,
    stop: Arc<AtomicBool>,
    thread: JoinHandle<Result<(), String>>,
}

static RECEIVER: Mutex<Option<Receiver>> = Mutex::new(None);
static LOG: Mutex<VecDeque<String>> = Mutex::new(VecDeque::new());
/// What the app last observed Android allows, per action.
static CAPABILITIES: Mutex<Vec<Capability>> = Mutex::new(Vec::new());

fn log(line: String) {
    let mut log = LOG.lock().unwrap_or_else(|e| e.into_inner());
    if log.len() == 64 {
        log.pop_front();
    }
    log.push_back(line);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_nexees_feasibility_NativeCore_call<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    request: JString<'caller>,
) -> JString<'caller> {
    let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
        let request = request.to_string();
        JString::from_str(env, handle(&request).to_string())
    });
    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}

fn handle(request: &str) -> Value {
    let Ok(request) = serde_json::from_str::<Value>(request) else {
        return json!({"error": "the request is not JSON"});
    };
    let Some(files) = request["files"].as_str().map(PathBuf::from) else {
        return json!({"error": "files is required"});
    };
    let result = match request["op"].as_str().unwrap_or("") {
        "probe" => run_probe(&files, request["name"].as_str().unwrap_or("probe")),
        "identity" => Identity::load_or_create(&files.join("remote/identity"))
            .map(|identity| json!({"fingerprint": identity.fingerprint()}))
            .map_err(|e| e.to_string()),
        "grant" => set_grant(&files, &request),
        "grants" => open_store(&files)
            .and_then(|store| store.grant(Direction::PcToPhone.grant()).map_err(|e| e.to_string()))
            .map(|enabled| json!({"pc_to_phone": enabled})),
        "receiver_start" => start_receiver(&files, request["port"].as_u64().unwrap_or(47200) as u16),
        "receiver_stop" => stop_receiver(),
        "receiver_status" => Ok(receiver_status()),
        "pending_launches" => pending_launches(&files),
        "command" => command(&files, request["id"].as_str().unwrap_or("")),
        "command_update" => update_command(&files, &request),
        "capabilities_set" => serde_json::from_value::<Vec<Capability>>(request["capabilities"].clone())
            .map(|list| {
                *CAPABILITIES.lock().unwrap_or_else(|e| e.into_inner()) = list;
                json!({"status": "completed"})
            })
            .map_err(|e| e.to_string()),
        "send" => send(&files, &request),
        "help" => help(&files, request["chapter"].as_str().unwrap_or("")),
        "chapters" => chapters(&files),
        "workspace_file" => workspace_file(&files, &request),
        other => Err(format!("unknown op {other:?}")),
    };
    result.unwrap_or_else(|e| json!({"error": e}))
}

fn assets(files: &Path) -> PathBuf {
    files.join("assets")
}

fn open_store(files: &Path) -> Result<Store, String> {
    Store::open(&files.join("state.sqlite")).map_err(|e| e.to_string())
}

fn open_lcl(files: &Path) -> Result<Lcl, String> {
    Lcl::open(&assets(files).join("LCL_Core_0.1.0"), &assets(files).join("LCL_Core_0.3.0"))
}

fn run_probe(files: &Path, name: &str) -> Result<Value, String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err("the probe name must be letters, digits and hyphens".to_string());
    }
    std::fs::create_dir_all(files.join("probe")).map_err(|e| e.to_string())?;
    let config = Config {
        work: files.join("probe").join(name),
        core_0_1_0: assets(files).join("LCL_Core_0.1.0"),
        core_0_3_0: assets(files).join("LCL_Core_0.3.0"),
        fixtures: assets(files).join("fixtures"),
        manual: assets(files).join("users_manual"),
        crash_child: None,
    };
    let report = probe::run(&config);
    let value = serde_json::to_value(&report).map_err(|e| e.to_string())?;
    std::fs::write(files.join("probe").join(format!("{name}.json")), value.to_string()).map_err(|e| e.to_string())?;
    Ok(value)
}

/// The phone holds only the grant for requests the phone executes. The app
/// calls this from an explicit user action and nowhere else.
fn set_grant(files: &Path, request: &Value) -> Result<Value, String> {
    let (Some("pc_to_phone"), Some(enabled)) = (request["direction"].as_str(), request["enabled"].as_bool()) else {
        return Err("this device holds only the pc_to_phone grant".to_string());
    };
    open_store(files)?.set_grant("pc_to_phone", enabled).map_err(|e| e.to_string())?;
    Ok(json!({"pc_to_phone": enabled}))
}

/// Queues launches for the app's launcher, which knows whether Android lets
/// the app start an activity now.
struct AndroidPlatform;

impl Platform for AndroidPlatform {
    fn launch(&mut self, _command: &str, app: &str, _expires_at: u64) -> (Status, String) {
        let known = CAPABILITIES
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|c| c.action == format!("launch_app:{app}") && c.availability != "unsupported");
        if known {
            (Status::Accepted, "queued for the phone's launcher; ask for this command's status".to_string())
        } else {
            (Status::Unsupported, format!("{app} is not an app this phone can open through a public intent"))
        }
    }

    fn capabilities(&self) -> Vec<Capability> {
        let mut list = vec![Capability {
            action: "write_lcl_file".to_string(),
            availability: "available".to_string(),
            note: "isolated phone-local LCL workspaces only".to_string(),
        }];
        list.extend(CAPABILITIES.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned());
        list
    }
}

fn start_receiver(files: &Path, port: u16) -> Result<Value, String> {
    let mut slot = RECEIVER.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(receiver) = slot.as_ref() {
        if !receiver.thread.is_finished() {
            return Ok(json!({"status": "running", "port": receiver.port}));
        }
    }
    let identity = Identity::load_or_create(&files.join("remote/identity")).map_err(|e| e.to_string())?;
    let peer = remote::read_certificate(&files.join("remote/peer-pc.cert.der"))
        .map_err(|e| format!("no paired PC certificate: {e}"))?;
    let config = remote::server_config(&identity, &peer)?;
    // Every interface: the port answers only a client holding the paired
    // PC's certificate, and only while the user keeps the receiver running.
    let listener = TcpListener::bind(("0.0.0.0", port)).map_err(|e| e.to_string())?;
    let stop = Arc::new(AtomicBool::new(false));
    let thread = {
        let (files, stop) = (files.to_path_buf(), stop.clone());
        std::thread::spawn(move || -> Result<(), String> {
            let lcl = open_lcl(&files)?;
            let mut store = open_store(&files)?;
            let device = Device {
                id: THIS_DEVICE.into(),
                peer: PAIRED_PC.into(),
                inbound: Direction::PcToPhone,
                workspaces: files.join("workspaces"),
                lcl: &lcl,
            };
            let served = remote::serve(
                listener,
                config,
                &stop,
                |envelope| remote::dispatch(&device, &mut store, &mut AndroidPlatform, envelope),
                |line| log(line.to_string()),
            );
            log(format!("receiver stopped: {served:?}"));
            served.map_err(|e| e.to_string())
        })
    };
    *slot = Some(Receiver { port, stop, thread });
    log(format!("receiver listening on port {port}"));
    Ok(json!({"status": "running", "port": port}))
}

fn stop_receiver() -> Result<Value, String> {
    let receiver = RECEIVER.lock().unwrap_or_else(|e| e.into_inner()).take();
    let Some(receiver) = receiver else {
        return Ok(json!({"status": "stopped"}));
    };
    receiver.stop.store(true, Ordering::SeqCst);
    let joined = receiver.thread.join().map_err(|_| "the receiver thread panicked".to_string())?;
    Ok(json!({"status": "stopped", "result": format!("{joined:?}")}))
}

fn receiver_status() -> Value {
    let running = RECEIVER
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .is_some_and(|r| !r.thread.is_finished());
    let log: Vec<String> = LOG.lock().unwrap_or_else(|e| e.into_inner()).iter().cloned().collect();
    json!({"running": running, "log": log})
}

fn pending_launches(files: &Path) -> Result<Value, String> {
    let commands = open_store(files)?.commands_with_status("accepted").map_err(|e| e.to_string())?;
    let launches: Vec<Value> = commands
        .iter()
        .filter_map(|c| {
            c.action
                .strip_prefix("launch_app:")
                .map(|app| json!({"id": c.id, "app": app, "expires_at": c.expires_at}))
        })
        .collect();
    Ok(json!({"launches": launches, "now": crate::unix_now()}))
}

fn command(files: &Path, id: &str) -> Result<Value, String> {
    let command = open_store(files)?.command(id).map_err(|e| e.to_string())?;
    Ok(match command {
        Some(c) => json!({"id": c.id, "action": c.action, "status": c.status, "detail": c.detail, "expires_at": c.expires_at, "now": crate::unix_now()}),
        None => json!({"id": id, "status": "outcome_unknown", "detail": "no such command"}),
    })
}

fn update_command(files: &Path, request: &Value) -> Result<Value, String> {
    let (Some(id), Some(status), Some(detail)) =
        (request["id"].as_str(), request["status"].as_str(), request["detail"].as_str())
    else {
        return Err("id, status and detail are required".to_string());
    };
    let status = Status::parse(status);
    open_store(files)?.set_command_status(id, status.as_str(), detail).map_err(|e| e.to_string())?;
    Ok(json!({"id": id, "status": status.as_str()}))
}

/// A phone-to-PC request, signed by this phone's identity and pinned to the
/// paired PC's certificate.
fn send(files: &Path, request: &Value) -> Result<Value, String> {
    let address = request["address"].as_str().ok_or("address is required")?;
    let action: Action = serde_json::from_value(request["action"].clone()).map_err(|e| e.to_string())?;
    let identity = Identity::load_or_create(&files.join("remote/identity")).map_err(|e| e.to_string())?;
    let peer = remote::read_certificate(&files.join("remote/peer-pc.cert.der")).map_err(|e| e.to_string())?;
    let config = remote::client_config(&identity, &peer)?;
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let envelope = Envelope {
        protocol: remote::PROTOCOL,
        id: format!("{THIS_DEVICE}-{nanos:x}"),
        from_device: THIS_DEVICE.into(),
        to_device: PAIRED_PC.into(),
        direction: Direction::PhoneToPc,
        expires_at: crate::unix_now() + request["expires_in"].as_u64().unwrap_or(60),
        action,
    };
    Ok(match remote::send(address, config, &envelope) {
        Ok(reply) => json!({"id": envelope.id, "reply": reply}),
        Err(e) => json!({"id": envelope.id, "reply": {"status": e.status().as_str(), "detail": e.to_string()}}),
    })
}

fn chapters(files: &Path) -> Result<Value, String> {
    let mut names: Vec<String> = std::fs::read_dir(assets(files).join("users_manual"))
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".md"))
        .collect();
    names.sort();
    Ok(json!({"chapters": names}))
}

fn help(files: &Path, chapter: &str) -> Result<Value, String> {
    if !chapter.ends_with(".md") || chapter.contains('/') || chapter.starts_with('.') {
        return Err("not a manual chapter".to_string());
    }
    let markdown = std::fs::read_to_string(assets(files).join("users_manual").join(chapter)).map_err(|e| e.to_string())?;
    Ok(json!({"chapter": chapter, "html": crate::help::render(&markdown)}))
}

fn workspace_file(files: &Path, request: &Value) -> Result<Value, String> {
    let (Some(workspace), Some(path)) = (request["workspace"].as_str(), request["path"].as_str()) else {
        return Err("workspace and path are required".to_string());
    };
    let latest = open_store(files)?.latest(workspace, path).map_err(|e| e.to_string())?;
    Ok(match latest {
        Some((saved, content)) => json!({
            "revision": saved.revision,
            "sha256": saved.sha256,
            "content": String::from_utf8_lossy(&content),
            "on_disk_sha256": std::fs::read(files.join("workspaces").join(workspace).join(path))
                .map(|bytes| crate::sha256_hex(&bytes))
                .unwrap_or_default(),
        }),
        None => json!({"revision": 0}),
    })
}
