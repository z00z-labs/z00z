use std::path::PathBuf;

use z00z_utils::logger::{Logger, StdoutLogger};

fn main() {
    if let Err(error) = run_from_args() {
        StdoutLogger.error(&format!("scenario_2.failed: {error}"));
        std::process::exit(1);
    }
}

fn run_from_args() -> Result<(), z00z_simulator::scenario_2::Scenario2Err> {
    let mut args = std::env::args().skip(1);
    let mut config = None;
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--config" => {
                config = Some(PathBuf::from(args.next().ok_or_else(|| {
                    z00z_simulator::scenario_2::Scenario2Err::Config(
                        "missing value for --config".to_string(),
                    )
                })?));
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => {
                return Err(z00z_simulator::scenario_2::Scenario2Err::Config(format!(
                    "unknown argument: {other}"
                )));
            }
        }
    }

    let summary = match config {
        Some(path) => z00z_simulator::scenario_2::run_with_path(path)?,
        None => z00z_simulator::scenario_2::run()?,
    };
    StdoutLogger.info(&format!(
        "scenario_2.done: run_dir={} blocks={} transactions={}",
        summary.run_dir.display(),
        summary.blocks,
        summary.transactions
    ));
    Ok(())
}

fn print_help() {
    println!("Usage: scenario_2 [--config <path>]");
    println!(
        "Run only as a release workload: cargo run -p z00z_simulator --release --bin scenario_2"
    );
}
