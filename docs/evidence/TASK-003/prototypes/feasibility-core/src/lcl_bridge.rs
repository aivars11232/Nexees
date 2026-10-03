//! The existing LCL engine used as a library.
//!
//! This mirrors what `lcl validate` and `lcl run` do in the LCL CLI
//! (`lcl-cli/src/main.rs`): open the Core 0.1.0 engine and the Core 0.3.0
//! project engine, find the document's project root, and let
//! [`Engines::engine_for`] pick the engine the document declares. Nothing here
//! parses or judges LCL itself.

use lcl_project::Project;
use lcl_protocol::{Engine, Engines, Granted, Inputs, Report};
use serde::{Deserialize, Serialize};
use std::path::Path;

pub struct Lcl {
    engines: Engines,
}

/// The parts of an engine report the feasibility checks compare.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Verdict {
    /// `accepted`, `rejected` or `refused`.
    pub outcome: String,
    /// The last stage the engine reached.
    pub reached: String,
    /// Set only by `run`, e.g. `status.succeeded`.
    pub terminal_status: Option<String>,
    /// The primary diagnostic as `id at line:column`, if any.
    pub primary: Option<String>,
}

impl Verdict {
    fn from_report(report: &Report) -> Verdict {
        Verdict {
            outcome: report.outcome.as_str().to_string(),
            reached: report.reached.as_str().to_string(),
            terminal_status: report.terminal_status().map(str::to_string),
            primary: report.primary().map(|d| {
                format!("{} at {}:{}", d.id, d.position.line, d.position.column)
            }),
        }
    }

    pub fn accepted(&self) -> bool {
        self.outcome == "accepted"
    }
}

impl Lcl {
    /// Opens the canonical Core packages. The engine checks each package
    /// against the trust anchor compiled into it and refuses a changed one.
    pub fn open(core_0_1_0: &Path, core_0_3_0: &Path) -> Result<Lcl, String> {
        let core = Engine::open(core_0_1_0).map_err(|e| format!("Core 0.1.0: {e}"))?;
        let project =
            Engine::open_project(core_0_3_0, &[]).map_err(|e| format!("Core 0.3.0: {e}"))?;
        let engines = Engines::new(core, None)
            .and_then(|engines| engines.with_project(project))
            .map_err(|e| e.to_string())?;
        Ok(Lcl { engines })
    }

    /// Steps 1 to 9 of the canonical processing. No effect can occur.
    pub fn validate(&self, document: &Path) -> Result<Verdict, String> {
        self.request(document, false)
    }

    /// Steps 1 to 13 with no capability granted: no file, process or network
    /// access is available to the document.
    pub fn run(&self, document: &Path) -> Result<Verdict, String> {
        self.request(document, true)
    }

    fn request(&self, document: &Path, run: bool) -> Result<Verdict, String> {
        let (root, _) = lcl_project::locate_document(document).map_err(|e| e.to_string())?;
        let project = if root.join(lcl_project::MANIFEST_FILE).is_file() {
            Project::open(&root)
        } else {
            Project::rootless(&root)
        }
        .map_err(|e| e.to_string())?;
        let provider = project.provider().map_err(|e| e.to_string())?;
        let path = document.canonicalize().map_err(|e| e.to_string())?;
        let unit = provider.root_unit(&path).map_err(|e| e.to_string())?;
        let engine = self.engines.engine_for(&unit);
        let report = if run {
            let (mut stdlib, mut host) =
                lcl_protocol::surface(engine, &Granted::none()).map_err(|e| e.to_string())?;
            engine.run(&unit, &provider, &Inputs::new(), &mut stdlib, &mut host)
        } else {
            engine.validate(&unit, &provider, &Inputs::new())
        };
        Ok(Verdict::from_report(&report))
    }
}
