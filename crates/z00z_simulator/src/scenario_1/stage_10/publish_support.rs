use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use z00z_storage::checkpoint::CheckpointFsStore;
use z00z_utils::io::path_exists;

use super::super::stage_9::bundle_lane_impl::{
    load_frag, load_stage9_bridge, FragTx, Stage9Bridge,
};

pub(crate) struct PublishState {
    pub(crate) frag_a: FragTx,
    pub(crate) frag_b: FragTx,
    pub(crate) bridge: Stage9Bridge,
}

pub(crate) fn load_publish_state(
    tx_dir: &Path,
    paths: &crate::config::Stage6PathsCfg,
) -> Result<PublishState, String> {
    let frag_a = load_frag(&tx_dir.join(&paths.frag1_file), "frag1")?;
    let frag_b = load_frag(&tx_dir.join(&paths.frag2_file), "frag2")?;
    let bridge = load_stage9_bridge(&tx_dir.join(&paths.checkpoint_file))?;
    let exec_path = expect_exec_input(tx_dir, &bridge.exec_input_id_hex)?;
    if !path_exists(&exec_path).map_err(|e| e.to_string())? {
        return Err(format!(
            "stage6 publish missing exec_input {}",
            exec_path.display()
        ));
    }
    Ok(PublishState {
        frag_a,
        frag_b,
        bridge,
    })
}

pub(crate) fn write_step_fallbacks(
    stage: &crate::DesignStage,
    covered: &HashSet<String>,
    lines: &mut Vec<String>,
) -> Result<(), String> {
    let mut missing = stage
        .steps
        .iter()
        .filter(|step| !covered.contains(&step.id))
        .map(|step| step.id.clone())
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        missing.sort();
        return Err(format!(
            "stage {} missing canonical coverage for steps: {}",
            stage.stage,
            missing.join(", ")
        ));
    }
    let _ = lines;
    Ok(())
}

fn expect_exec_input(tx_dir: &Path, exec_input_id_hex: &str) -> Result<PathBuf, String> {
    let path = CheckpointFsStore::new(tx_dir)
        .exec_dir()
        .join(format!("{exec_input_id_hex}.bin"));
    if !path_exists(&path).map_err(|e| e.to_string())? {
        return Err(format!("missing exec_input {}", path.display()));
    }
    Ok(path)
}
