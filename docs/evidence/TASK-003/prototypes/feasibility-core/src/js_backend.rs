//! An on-device code/test backend: JavaScript tests run in the Boa engine,
//! which is pure Rust and so builds for Android with no extra toolchain.
//!
//! The engine has no ambient authority: the context has no file, process,
//! network or timer API, only the sources passed in. Loop, recursion and stack
//! limits bound a runaway test.

use boa_engine::{Context, Source};
use serde::Deserialize;

/// The test harness the fixture's tests call. Results stay inside the engine
/// until read back as JSON.
const PRELUDE: &str = r#"
const __results = [];
function test(name, body) {
  try { body(); __results.push({ name, ok: true, error: "" }); }
  catch (e) { __results.push({ name, ok: false, error: String(e) }); }
}
function assertEqual(actual, expected) {
  if (actual !== expected) throw new Error(`expected ${expected}, got ${actual}`);
}
"#;

#[derive(Debug, Clone, Deserialize, serde::Serialize, PartialEq, Eq)]
pub struct Case {
    pub name: String,
    pub ok: bool,
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Run {
    pub cases: Vec<Case>,
}

impl Run {
    pub fn passed(&self) -> bool {
        !self.cases.is_empty() && self.cases.iter().all(|c| c.ok)
    }
}

/// Evaluates `sources` in order, then `tests`, and returns each test case.
/// An error outside a test case (a syntax error or an exceeded limit) is an
/// `Err`: the run did not complete, which is never reported as a pass.
pub fn run_tests(sources: &[&str], tests: &str) -> Result<Run, String> {
    let mut context = Context::default();
    let limits = context.runtime_limits_mut();
    limits.set_loop_iteration_limit(1_000_000);
    limits.set_recursion_limit(256);
    limits.set_stack_size_limit(64 * 1024);
    let mut eval = |code: &str| {
        context
            .eval(Source::from_bytes(code))
            .map_err(|e| e.to_string())
    };
    eval(PRELUDE)?;
    for source in sources {
        eval(source)?;
    }
    eval(tests)?;
    let json = context
        .eval(Source::from_bytes("JSON.stringify(__results)"))
        .map_err(|e| e.to_string())?
        .to_string(&mut context)
        .map_err(|e| e.to_string())?
        .to_std_string_escaped();
    let cases: Vec<Case> = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    Ok(Run { cases })
}
