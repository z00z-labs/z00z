use z00z_storage::checkpoint::recursive_v2::{
    DiagnosticSingleStepEnvelopeV2, LiveGateStageV2, PostwriteVerifiedV2, PreparedReceiptV2,
    RecursiveNovaStepInputV2,
};

fn require_clone<T: Clone>() {}

fn main() {
    require_clone::<LiveGateStageV2>();
    require_clone::<PostwriteVerifiedV2>();
    require_clone::<PreparedReceiptV2>();
    let _diagnostic = DiagnosticSingleStepEnvelopeV2;
    let _forged: RecursiveNovaStepInputV2 = Default::default();
}
