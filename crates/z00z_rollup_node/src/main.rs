#![forbid(unsafe_code)]

use z00z_rollup_node::{maybe_run_hjmt_process_devnet, AggRunArgs, NodeConfig};

fn main() {
    match run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(2);
        }
    }
}

fn run() -> Result<(), String> {
    let argv = std::env::args().skip(1).collect::<Vec<_>>();
    if argv.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }
    if argv.is_empty() {
        print_help();
        return Err("missing rollup-node arguments".to_string());
    }

    let args = AggRunArgs::parse_cli_argv(&argv)?;
    let launch = NodeConfig::from_agg_run_args(&args).map_err(|err| err.to_string())?;
    let hjmt = launch.config.hjmt.as_ref().ok_or_else(|| {
        "hjmt config must be loaded for the live aggregator process contract".to_string()
    })?;
    let agg = hjmt
        .proc(launch.aggregator_id)
        .ok_or_else(|| format!("unknown aggregator id {}", launch.aggregator_id.as_u16()))?;

    println!(
        "z00z_rollup_node ready: mode=aggregator aggregator_id={} profile={} listen_addr={} route_generation={}",
        launch.aggregator_id.as_u16(),
        hjmt.profile,
        agg.network.listen_addr,
        hjmt.routing_generation(),
    );
    maybe_run_hjmt_process_devnet(&launch)?;
    Ok(())
}

fn print_help() {
    println!(
        "\
Usage:
  z00z_rollup_node --mode aggregator --aggregator-config <path> --planner-config <path> --storage-config <path>

Live scope:
  Only --mode aggregator is executable in the current Phase 067 process contract.

Required arguments:
  --mode <mode>                   Must be `aggregator`
  --aggregator-config <path>      Path to aggregator-config.yaml
  --planner-config <path>         Path to planner-config.yaml
  --storage-config <path>         Path to storage-config.yaml
"
    );
}
