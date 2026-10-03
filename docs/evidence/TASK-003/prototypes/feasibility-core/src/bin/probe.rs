//! `nexees-probe <work> <core-0.1.0> <core-0.3.0> <fixtures> <manual>` runs
//! every feasibility check and prints the JSON report. It exits 0 only when
//! every check passed.
//!
//! `nexees-probe crash-write <database>` is the child half of the crash check
//! and is not meant to be run by hand.

use nexees_feasibility::probe::{self, Config};
use std::path::PathBuf;

fn main() {
    let args: Vec<PathBuf> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    if args.len() == 2 && args[0].as_os_str() == "crash-write" {
        probe::crash_write(&args[1]);
    }
    let Ok([work, core_0_1_0, core_0_3_0, fixtures, manual]) = <[PathBuf; 5]>::try_from(args) else {
        eprintln!("usage: nexees-probe <work> <core-0.1.0> <core-0.3.0> <fixtures> <manual>");
        std::process::exit(2);
    };
    let config = Config {
        work,
        core_0_1_0,
        core_0_3_0,
        fixtures,
        manual,
        crash_child: std::env::current_exe().ok(),
    };
    let report = probe::run(&config);
    println!("{}", serde_json::to_string_pretty(&report).expect("the report serializes"));
    std::process::exit(if report.passed { 0 } else { 1 });
}
