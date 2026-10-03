//! Runs every local feasibility check against real inputs and reports each
//! one. The same code runs as the `nexees-probe` executable and, through JNI,
//! inside the Android app, so a Desktop pass and a phone pass test one thing.

use crate::import::{self, Limits};
use crate::js_backend;
use crate::lcl_bridge::Lcl;
use crate::remote::{self, Action, Capability, Device, Direction, Envelope, Identity, Platform, Reply, SendError, Status};
use crate::store::Store;
use serde::Serialize;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use zip::write::SimpleFileOptions;

pub struct Config {
    /// A folder that must not exist yet; everything the probe writes goes here.
    pub work: PathBuf,
    pub core_0_1_0: PathBuf,
    pub core_0_3_0: PathBuf,
    /// `docs/evidence/TASK-003/prototypes/feasibility-core/fixtures`.
    pub fixtures: PathBuf,
    /// The LCL user manual's Markdown chapters.
    pub manual: PathBuf,
    /// This executable, to run the crash check in a child process. `None`
    /// inside the Android app, which cannot start a second copy of itself.
    pub crash_child: Option<PathBuf>,
}

#[derive(Debug, Serialize)]
pub struct Check {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct Report {
    pub target: String,
    pub sqlite: String,
    pub checks: Vec<Check>,
    pub passed: bool,
}

struct Checks(Vec<Check>);

impl Checks {
    fn record(&mut self, id: &str, passed: bool, detail: impl Into<String>) {
        self.0.push(Check { id: id.to_string(), passed, detail: detail.into() });
    }
}

pub fn run(config: &Config) -> Report {
    let mut checks = Checks(Vec::new());
    let target = format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS);
    let report = |checks: Checks| {
        let passed = !checks.0.is_empty() && checks.0.iter().all(|c| c.passed);
        Report { target: target.clone(), sqlite: rusqlite::version().to_string(), checks: checks.0, passed }
    };
    if let Err(e) = fs::create_dir(&config.work) {
        checks.record("setup.work", false, format!("{}: {e}", config.work.display()));
        return report(checks);
    }
    let lcl = match Lcl::open(&config.core_0_1_0, &config.core_0_3_0) {
        Ok(lcl) => lcl,
        Err(e) => {
            checks.record("lcl.open", false, e);
            return report(checks);
        }
    };
    checks.record("lcl.open", true, "Core 0.1.0 and Core 0.3.0 opened against their trust anchors");
    lcl_checks(&mut checks, &lcl, &config.fixtures);
    store_checks(&mut checks, &config.work, config.crash_child.as_deref());
    import_checks(&mut checks, &lcl, config);
    js_checks(&mut checks, config);
    help_checks(&mut checks, config);
    remote_checks(&mut checks, config);
    report(checks)
}

fn lcl_checks(checks: &mut Checks, lcl: &Lcl, fixtures: &Path) {
    let valid = fixtures.join("valid_task.lcl.txt");
    let invalid = fixtures.join("invalid_task.lcl.txt");
    let project = fixtures.join("project/main.lcl.txt");
    match (lcl.validate(&valid), lcl.run(&valid)) {
        (Ok(v), Ok(r)) => checks.record(
            "lcl.valid_task",
            v.accepted() && r.terminal_status.as_deref() == Some("status.succeeded"),
            format!("validate {}; run {:?}", v.outcome, r.terminal_status),
        ),
        (v, r) => checks.record("lcl.valid_task", false, format!("{v:?} {r:?}")),
    }
    match lcl.validate(&invalid) {
        Ok(v) => checks.record(
            "lcl.invalid_task",
            v.outcome == "rejected" && v.primary.as_deref() == Some("error.reference.unresolved at 28:17"),
            format!("validate {} with {:?}", v.outcome, v.primary),
        ),
        Err(e) => checks.record("lcl.invalid_task", false, e),
    }
    match (lcl.validate(&project), lcl.run(&project)) {
        (Ok(v), Ok(r)) => checks.record(
            "lcl.project",
            v.accepted() && r.terminal_status.as_deref() == Some("status.succeeded"),
            format!("four-file Core 0.3.0 project: validate {}; run {:?}", v.outcome, r.terminal_status),
        ),
        (v, r) => checks.record("lcl.project", false, format!("{v:?} {r:?}")),
    }
}

fn store_checks(checks: &mut Checks, work: &Path, crash_child: Option<&Path>) {
    let path = work.join("state.sqlite");
    let first = Store::open_at(&path, 1).and_then(|mut store| store.save_revision("w", "a.lcl.txt", b"one"));
    let reopened = Store::open(&path);
    match (first, reopened) {
        (Ok(saved), Ok(store)) => {
            let latest = store.latest("w", "a.lcl.txt").ok().flatten();
            let version = store.version().unwrap_or(0);
            let journal = store.journal_mode().unwrap_or_default();
            let integrity = store.integrity().unwrap_or_default();
            checks.record(
                "store.migration_and_reopen",
                latest.as_ref().is_some_and(|(s, c)| *s == saved && c == b"one")
                    && version == crate::store::SCHEMA_VERSION
                    && journal == "wal"
                    && integrity == "ok",
                format!("v1 data kept after migrating to v{version}; journal {journal}; integrity {integrity}"),
            );
        }
        (a, b) => {
            checks.record("store.migration_and_reopen", false, format!("{:?} {:?}", a.err(), b.err().map(|e| e.to_string())))
        }
    }

    let rollback = Store::open(&path).and_then(|mut store| {
        {
            let tx = store.connection().transaction()?;
            tx.execute(
                "INSERT INTO revisions (workspace, path, revision, sha256, content) VALUES ('w', 'a.lcl.txt', 2, 'x', x'00')",
                [],
            )?;
            // Dropped without commit: rolled back.
        }
        store.latest("w", "a.lcl.txt")
    });
    checks.record(
        "store.rollback",
        matches!(&rollback, Ok(Some((s, _))) if s.revision == 1),
        format!("an uncommitted revision is not visible: {:?}", rollback.as_ref().map(|l| l.as_ref().map(|(s, _)| s.revision))),
    );

    match crash_child {
        None => checks.record("store.crash_recovery", true, "not run in this process; the app's kill and reboot tests cover process death"),
        Some(exe) => {
            let child = std::process::Command::new(exe).arg("crash-write").arg(&path).status();
            let after = Store::open(&path).and_then(|store| Ok((store.latest("w", "a.lcl.txt")?, store.integrity()?)));
            checks.record(
                "store.crash_recovery",
                matches!(&child, Ok(status) if !status.success())
                    && matches!(&after, Ok((Some((s, _)), integrity)) if s.revision == 1 && integrity == "ok"),
                format!("child aborted mid-transaction ({child:?}); after reopen: {:?}", after.map(|(l, i)| (l.map(|(s, _)| s.revision), i))),
            );
        }
    }
}

/// The child half of `store.crash_recovery`: write without committing, then
/// abort the process the way a crash or an OS kill would.
pub fn crash_write(path: &Path) -> ! {
    let mut store = Store::open(path).expect("open store");
    let tx = store.connection().transaction().expect("begin");
    tx.execute(
        "INSERT INTO revisions (workspace, path, revision, sha256, content) VALUES ('w', 'a.lcl.txt', 3, 'x', x'00')",
        [],
    )
    .expect("insert");
    std::process::abort()
}

enum Entry<'a> {
    File(&'a [u8]),
    Symlink(&'a str),
}

/// An archive's name, its entries, and the rejection code it must get
/// (`None` for the one archive that must import).
type ArchiveCase<'a> = (&'a str, Vec<(&'a str, Entry<'a>)>, Option<&'a str>);

fn write_zip(path: &Path, entries: &[(&str, Entry)]) -> zip::result::ZipResult<()> {
    let mut zip = zip::ZipWriter::new(fs::File::create(path)?);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for (name, entry) in entries {
        match entry {
            Entry::File(bytes) => {
                zip.start_file(*name, options)?;
                zip.write_all(bytes)?;
            }
            Entry::Symlink(target) => zip.add_symlink(*name, *target, options)?,
        }
    }
    zip.finish()?;
    Ok(())
}

fn import_checks(checks: &mut Checks, lcl: &Lcl, config: &Config) {
    let fixtures = &config.fixtures;
    let read = |p: &str| fs::read(fixtures.join(p)).unwrap_or_default();
    let (main, values, rules, record) = (
        read("project/main.lcl.txt"),
        read("project/data/values.lcl.txt"),
        read("project/checks/checks.lcl.txt"),
        read("project/tasks/record.lcl.txt"),
    );
    let invalid = read("invalid_task.lcl.txt");
    let zeros = vec![0u8; 900 * 1024];
    let cases: Vec<ArchiveCase> = vec![
        (
            "good",
            vec![
                ("main.lcl.txt", Entry::File(&main)),
                ("data/values.lcl.txt", Entry::File(&values)),
                ("checks/checks.lcl.txt", Entry::File(&rules)),
                ("tasks/record.lcl.txt", Entry::File(&record)),
            ],
            None,
        ),
        ("traversal", vec![("main.lcl.txt", Entry::File(&main)), ("../escape.lcl.txt", Entry::File(&values))], Some("unsafe_name")),
        ("absolute", vec![("/tmp/escape.lcl.txt", Entry::File(&main))], Some("unsafe_name")),
        ("backslash", vec![("..\\escape.lcl.txt", Entry::File(&main))], Some("unsafe_name")),
        ("symlink", vec![("main.lcl.txt", Entry::File(&main)), ("data", Entry::Symlink("/etc"))], Some("not_regular")),
        ("duplicate", vec![("main.lcl.txt", Entry::File(&main)), ("MAIN.lcl.txt", Entry::File(&main))], Some("duplicate")),
        ("bomb", vec![("main.lcl.txt", Entry::File(&main)), ("zeros.bin", Entry::File(&zeros))], Some("compression_ratio")),
        ("invalid", vec![("main.lcl.txt", Entry::File(&invalid))], Some("lcl_invalid")),
    ];
    let archives = config.work.join("archives");
    let staging = config.work.join("staging");
    let workspaces = config.work.join("workspaces");
    let _ = fs::create_dir_all(&archives);
    for (name, entries, expected) in cases {
        let archive = archives.join(format!("{name}.zip"));
        if let Err(e) = write_zip(&archive, &entries) {
            checks.record(&format!("import.{name}"), false, format!("could not build the archive: {e}"));
            continue;
        }
        let result = import::import(&archive, &staging, &workspaces, &format!("imported-{name}"), &Limits::default(), lcl);
        match (result, expected) {
            (Ok(imported), None) => checks.record(
                &format!("import.{name}"),
                imported.verdict.accepted() && imported.files == 4,
                format!("{} files, {} bytes published after LCL validation {}", imported.files, imported.bytes, imported.verdict.outcome),
            ),
            (Err(rejection), Some(code)) => checks.record(
                &format!("import.{name}"),
                rejection.code() == code,
                format!("rejected: {rejection:?}"),
            ),
            (outcome, _) => checks.record(&format!("import.{name}"), false, format!("unexpected: {outcome:?}")),
        }
    }
    let leftovers: Vec<String> = fs::read_dir(&staging)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    let published: Vec<String> = fs::read_dir(&workspaces)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.file_name().to_string_lossy().into_owned()).collect())
        .unwrap_or_default();
    let escaped = config.work.join("escape.lcl.txt").exists() || Path::new("/tmp/escape.lcl.txt").exists();
    checks.record(
        "import.nothing_left_behind",
        leftovers.is_empty() && published == ["imported-good"] && !escaped,
        format!("staging {leftovers:?}; published {published:?}; escaped file present: {escaped}"),
    );
}

fn js_checks(checks: &mut Checks, config: &Config) {
    let workspace = config.work.join("js-workspace");
    let copied = fs::create_dir_all(workspace.join("src"))
        .and_then(|_| fs::create_dir_all(workspace.join("test")))
        .and_then(|_| fs::copy(config.fixtures.join("js/src/sum.js"), workspace.join("src/sum.js")))
        .and_then(|_| fs::copy(config.fixtures.join("js/test/sum.test.js"), workspace.join("test/sum.test.js")));
    if let Err(e) = copied {
        checks.record("js.fail_repair_pass", false, format!("fixture copy failed: {e}"));
        return;
    }
    let run = || -> Result<js_backend::Run, String> {
        let source = fs::read_to_string(workspace.join("src/sum.js")).map_err(|e| e.to_string())?;
        let tests = fs::read_to_string(workspace.join("test/sum.test.js")).map_err(|e| e.to_string())?;
        js_backend::run_tests(&[&source], &tests)
    };
    let before = run();
    // The repair an agent would make, saved through the same store as any edit.
    let repaired = fs::read_to_string(workspace.join("src/sum.js"))
        .map(|s| s.replace("i < values.length - 1", "i < values.length"))
        .map_err(|e| e.to_string())
        .and_then(|s| {
            remote::write_atomically(&workspace.join("src/sum.js"), s.as_bytes()).map_err(|e| e.to_string())?;
            let mut store = Store::open(&config.work.join("state.sqlite")).map_err(|e| e.to_string())?;
            store.save_revision("js-workspace", "src/sum.js", s.as_bytes()).map_err(|e| e.to_string())
        });
    let after = run();
    let failed_first = matches!(&before, Ok(r) if !r.passed() && r.cases.iter().any(|c| !c.ok));
    checks.record(
        "js.fail_repair_pass",
        failed_first && repaired.is_ok() && matches!(&after, Ok(r) if r.passed()),
        format!("before: {before:?}; repair: {repaired:?}; after: {after:?}"),
    );
    let runaway = js_backend::run_tests(&[], "test('runaway', () => { while (true) {} });");
    checks.record(
        "js.runaway_bounded",
        matches!(&runaway, Ok(r) if !r.passed()) || runaway.is_err(),
        format!("an endless loop is stopped and not reported as a pass: {runaway:?}"),
    );
    let authority = js_backend::run_tests(
        &[],
        "test('no ambient authority', () => { assertEqual(typeof require + typeof fetch + typeof process + typeof XMLHttpRequest, 'undefinedundefinedundefinedundefined'); });",
    );
    checks.record(
        "js.no_ambient_authority",
        matches!(&authority, Ok(r) if r.passed()),
        format!("no require, fetch, process or XMLHttpRequest in the engine: {authority:?}"),
    );
}

fn help_checks(checks: &mut Checks, config: &Config) {
    let hostile = fs::read_to_string(config.fixtures.join("help/hostile.md")).unwrap_or_default();
    let html = crate::help::render(&hostile);
    let found = active_tags(&html);
    let kept = html.contains("href=\"chapter-2.md\"")
        && html.contains("href=\"https://example.org/manual\"")
        && html.contains("<img src=\"images/diagram.png\"");
    checks.record(
        "help.hostile_markdown",
        !hostile.is_empty() && found.is_empty() && kept,
        format!("active or remote tags: {found:?}; safe links and bundled image kept: {kept}"),
    );
    // The real LCL user manual, rendered offline chapter by chapter.
    let chapters: Vec<PathBuf> = fs::read_dir(&config.manual)
        .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "md")).collect())
        .unwrap_or_default();
    let mut bytes = (0usize, 0usize);
    let mut active = Vec::new();
    for chapter in &chapters {
        let markdown = fs::read_to_string(chapter).unwrap_or_default();
        let rendered = crate::help::render(&markdown);
        bytes = (bytes.0 + markdown.len(), bytes.1 + rendered.len());
        active.extend(active_tags(&rendered).into_iter().map(|t| format!("{}: {t}", chapter.display())));
    }
    checks.record(
        "help.offline_manual",
        chapters.len() >= 20 && active.is_empty(),
        format!("{} chapters, {} bytes of Markdown to {} bytes of passive HTML; active tags: {active:?}", chapters.len(), bytes.0, bytes.1),
    );
}

/// Every real tag in `html` that is active or loads something remote. Text
/// escapes `<`, so each `<` in the output starts a real tag.
fn active_tags(html: &str) -> Vec<String> {
    let mut found = Vec::new();
    for tag in html.split('<').skip(1).filter_map(|rest| rest.split('>').next()) {
        let tag = tag.to_ascii_lowercase();
        let name = tag.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
        let element = ["script", "iframe", "object", "embed", "style", "form", "input", "meta", "link", "base", "svg", "math"].contains(&name);
        let handler = tag.split_whitespace().skip(1).any(|attribute| {
            let name = attribute.split('=').next().unwrap_or("");
            attribute.contains('=') && name.len() > 2 && name.starts_with("on") && name.chars().all(|c| c.is_ascii_alphabetic())
        });
        let url = |attribute: &str, https_allowed: bool| {
            tag.split(attribute).skip(1).any(|value| {
                let value = value.split('"').next().unwrap_or("");
                crate::help::scheme(value).is_some_and(|s| !(https_allowed && s == "https"))
            })
        };
        if element || handler || url("href=\"", true) || url("src=\"", false) {
            found.push(tag);
        }
    }
    found
}

/// Answers launch requests the way a device with no launcher must.
struct ProbePlatform;

impl Platform for ProbePlatform {
    fn launch(&mut self, _command: &str, app: &str, _expires_at: u64) -> (Status, String) {
        (Status::Unsupported, format!("the probe has no application launcher for {app}"))
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability { action: "write_lcl_file".into(), availability: "available".into(), note: String::new() },
            Capability { action: "launch_app".into(), availability: "unsupported".into(), note: "probe only".into() },
        ]
    }
}

fn remote_checks(checks: &mut Checks, config: &Config) {
    let dir = config.work.join("remote");
    let identities = (
        Identity::load_or_create(&dir.join("phone")),
        Identity::load_or_create(&dir.join("pc")),
        Identity::load_or_create(&dir.join("intruder")),
    );
    let (phone, pc, intruder) = match identities {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        other => {
            checks.record("remote.setup", false, format!("{:?}", (other.0.err(), other.1.err(), other.2.err())));
            return;
        }
    };
    let (server, client, stranger) = match (
        remote::server_config(&phone, &pc.certificate),
        remote::client_config(&pc, &phone.certificate),
        remote::client_config(&intruder, &phone.certificate),
    ) {
        (Ok(a), Ok(b), Ok(c)) => (a, b, c),
        other => {
            checks.record("remote.setup", false, format!("{:?}", (other.0.err(), other.1.err(), other.2.err())));
            return;
        }
    };
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            checks.record("remote.setup", false, e.to_string());
            return;
        }
    };
    let address = listener.local_addr().map(|a| a.to_string()).unwrap_or_default();
    let state = dir.join("phone-state.sqlite");
    let workspaces = dir.join("phone-workspaces");
    let stop = Arc::new(AtomicBool::new(false));
    let receiver = {
        let (stop, state, workspaces) = (stop.clone(), state.clone(), workspaces.clone());
        let (core1, core3) = (config.core_0_1_0.clone(), config.core_0_3_0.clone());
        std::thread::spawn(move || -> Result<(), String> {
            let lcl = Lcl::open(&core1, &core3)?;
            let mut store = Store::open(&state).map_err(|e| e.to_string())?;
            let device = Device { id: "phone-1".into(), peer: "pc-1".into(), inbound: Direction::PcToPhone, workspaces, lcl: &lcl };
            remote::serve(listener, server, &stop, |envelope| remote::dispatch(&device, &mut store, &mut ProbePlatform, envelope), |_| {})
                .map_err(|e| e.to_string())
        })
    };
    let fixture = fs::read_to_string(config.fixtures.join("valid_task.lcl.txt")).unwrap_or_default();
    let mut counter = 0;
    let mut envelope = |action: Action, expires_in: i64, to: &str| {
        counter += 1;
        Envelope {
            protocol: remote::PROTOCOL,
            id: format!("probe-{counter}"),
            from_device: "pc-1".into(),
            to_device: to.into(),
            direction: Direction::PcToPhone,
            expires_at: (crate::unix_now() as i64 + expires_in) as u64,
            action,
        }
    };
    let write = |revision| Action::WriteLclFile {
        workspace: "remote-fixture".into(),
        path: "greeting.lcl.txt".into(),
        content: fixture.clone(),
        expected_revision: revision,
    };
    let send = |e: &Envelope| remote::send(&address, client.clone(), e);
    let status = |r: &Result<Reply, SendError>| match r {
        Ok(reply) => reply.status,
        Err(e) => e.status(),
    };
    let set_grant = |on| Store::open(&state).and_then(|mut s| s.set_grant(Direction::PcToPhone.grant(), on));

    let first = envelope(write(0), 60, "phone-1");
    let denied = send(&first);
    checks.record("remote.grant_absent_denied", status(&denied) == Status::Denied, format!("{denied:?}"));

    let granted = set_grant(true);
    let second = envelope(write(0), 60, "phone-1");
    let written = send(&second);
    let saved = fs::read(workspaces.join("remote-fixture/greeting.lcl.txt")).unwrap_or_default();
    checks.record(
        "remote.granted_write_validated",
        granted.is_ok()
            && matches!(&written, Ok(r) if r.status == Status::Completed
                && r.revision == Some(1)
                && r.sha256.as_deref() == Some(crate::sha256_hex(fixture.as_bytes()).as_str())
                && r.validation.as_ref().is_some_and(|v| v.accepted()))
            && saved == fixture.as_bytes(),
        format!("{written:?}"),
    );
    let replay = send(&second);
    checks.record("remote.replay_refused", status(&replay) == Status::Duplicate, format!("{replay:?}"));
    let stale = send(&envelope(write(0), 60, "phone-1"));
    checks.record("remote.stale_revision_conflict", status(&stale) == Status::Conflict, format!("{stale:?}"));
    let expired = send(&envelope(write(1), -1, "phone-1"));
    checks.record("remote.expired_refused", status(&expired) == Status::Expired, format!("{expired:?}"));
    let misdirected = send(&envelope(write(1), 60, "phone-2"));
    checks.record("remote.wrong_device_denied", status(&misdirected) == Status::Denied, format!("{misdirected:?}"));
    let escape = send(&envelope(
        Action::WriteLclFile { workspace: "remote-fixture".into(), path: "../escape.lcl.txt".into(), content: fixture.clone(), expected_revision: 0 },
        60,
        "phone-1",
    ));
    checks.record("remote.path_escape_denied", status(&escape) == Status::Denied, format!("{escape:?}"));
    let unpaired = remote::send(&address, stranger, &envelope(write(1), 60, "phone-1"));
    checks.record(
        "remote.unpaired_certificate_rejected",
        matches!(unpaired, Err(SendError::Rejected(_))),
        format!("{unpaired:?}"),
    );
    let launch = send(&envelope(Action::LaunchApp { app: "settings".into() }, 60, "phone-1"));
    let launch_status = send(&envelope(Action::Status { command: launch.as_ref().map(|r| r.id.clone()).unwrap_or_default() }, 60, "phone-1"));
    checks.record(
        "remote.launch_state_recorded",
        status(&launch) == Status::Unsupported && status(&launch_status) == Status::Unsupported,
        format!("launch: {launch:?}; later status: {launch_status:?}"),
    );
    let capabilities = send(&envelope(Action::Capabilities, 60, "phone-1"));
    checks.record(
        "remote.capabilities_advertised",
        matches!(&capabilities, Ok(r) if r.capabilities.len() == 2),
        format!("{capabilities:?}"),
    );
    let revoked = set_grant(false);
    let after_revoke = send(&envelope(write(1), 60, "phone-1"));
    checks.record(
        "remote.revoked_denied",
        revoked.is_ok() && status(&after_revoke) == Status::Denied,
        format!("{after_revoke:?}"),
    );
    checks.record("remote.oversized_frame_refused", oversized_refused(&address, client.clone()), "a 1 MiB length prefix is refused without reading the body");

    stop.store(true, Ordering::SeqCst);
    let joined = receiver.join();
    checks.record(
        "remote.receiver_stopped",
        matches!(joined, Ok(Ok(()))),
        format!("{joined:?}"),
    );
    let unavailable = send(&envelope(write(1), 60, "phone-1"));
    checks.record("remote.no_receiver_unavailable", status(&unavailable) == Status::Unavailable, format!("{unavailable:?}"));
}

/// Sends a valid TLS handshake and then a length prefix over the limit. The
/// receiver must close the connection instead of reading or replying.
fn oversized_refused(address: &str, config: Arc<rustls::ClientConfig>) -> bool {
    use std::io::Read;
    let Ok(socket) = std::net::TcpStream::connect(address) else { return false };
    socket.set_read_timeout(Some(std::time::Duration::from_secs(5))).ok();
    let Ok(name) = rustls::pki_types::ServerName::try_from("nexees-device") else { return false };
    let Ok(connection) = rustls::ClientConnection::new(config, name) else { return false };
    let mut stream = rustls::StreamOwned::new(connection, socket);
    if stream.write_all(&(1u32 << 20).to_be_bytes()).and_then(|_| stream.flush()).is_err() {
        return false;
    }
    let mut buffer = [0u8; 16];
    matches!(stream.read(&mut buffer), Ok(0) | Err(_))
}
