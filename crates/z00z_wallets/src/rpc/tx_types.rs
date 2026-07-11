//! Transaction-related RPC types
//!
//! Request and response types for tx.* JSON-RPC methods

use serde::{Deserialize, Serialize};

use super::common::PersistWalletId;
pub use super::common::{PersistTxId, RuntimePaginatedResponse, RuntimePaginationParams};
pub use crate::tx::{
    ThinAssetPathRef, ThinIndexEntry, ThinSnapshot, ThinSnapshotContext, ThinSnapshotPin,
    ThinWalletTxPackage,
};
use z00z_core::assets::registry::AssetId;

/// Transaction history filter parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTxHistoryFilter {
    /// Unix timestamp lower bound (inclusive, milliseconds).
    pub from_date: Option<u64>,
    /// Unix timestamp upper bound (inclusive, milliseconds).
    pub to_date: Option<u64>,
    /// Filter by transaction status.
    pub status: Option<TxStatus>,
    /// Filter by minimum amount (inclusive).
    pub min_amount: Option<u64>,
    /// Filter by maximum amount (inclusive).
    pub max_amount: Option<u64>,
}

/// Sorting parameters for transaction history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTxHistorySort {
    /// Field to sort by.
    pub by: TxHistorySortBy,
    /// Sort direction.
    pub direction: SortDirection,
}

/// Field options for tx.get_history sorting.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxHistorySortBy {
    Timestamp,
    Amount,
}

/// Sort direction.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
    Cancelled,
}

/// Public transaction lifecycle projected from durable tx-history rows.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTxLifecycle {
    Created,
    Imported,
    Exported,
    Submitted,
    Admitted,
    Confirmed,
    Failed,
    Cancelled,
    Conflicted,
    AlreadySpent,
}

/// Stable machine-readable transaction error codes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeTxErrorCode {
    InvalidEncoding,
    InvalidPackage,
    InvalidDigest,
    UnsupportedPackageVersion,
    UnsupportedReceiveVersion,
    WrongChain,
    ThinSnapshotInvalid,
    ThinSnapshotMissing,
    ThinSnapshotStale,
    ThinSnapshotConflict,
    InvalidPublicSpendProof,
    NoOwnedOutputs,
    NotImportReady,
    DuplicateConflict,
    AlreadySpent,
    CursorConflict,
    WorkerEvidenceRejected,
    InternalError,
}

/// Submitter role recorded at the tx admission boundary.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TxSubmitterRole {
    Sender,
    Receiver,
}

/// Wallet-side admission receipt for a canonical tx package.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeAdmissionReceipt {
    pub tx_id: PersistTxId,
    pub tx_hash_hex: String,
    pub chain_id: u32,
    pub submitter_role: TxSubmitterRole,
    pub admission_id_hex: String,
    pub admitted_at: u64,
    pub verified: bool,
}

/// Wallet-side confirmation receipt derived from checkpoint-style evidence.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfirmationReceipt {
    pub tx_id: PersistTxId,
    pub tx_hash_hex: String,
    pub block_height: u64,
    pub checkpoint_id_hex: String,
    pub prev_root_hex: String,
    pub new_root_hex: String,
    pub spent_asset_ids_hex: Vec<String>,
    pub created_asset_ids_hex: Vec<String>,
    pub confirmed_at: u64,
    pub verified: bool,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistTxInfo {
    pub id: PersistTxId,
    pub wallet_id: PersistWalletId,
    pub status: TxStatus,
    pub lifecycle: RuntimeTxLifecycle,
    pub amount: u64,
    pub fee: u64,
    /// Transaction creation timestamp (Unix milliseconds).
    pub timestamp: u64,
    /// Optional receipt information (if transaction confirmed)
    pub receipt: Option<PersistReceiptInfo>,
}

/// Transaction receipt information.
///
/// This is a compatibility summary for confirmed transactions. The canonical
/// checkpoint and root evidence lives on `RuntimeConfirmationReceipt`; this
/// wrapper must not claim to carry proof payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistReceiptInfo {
    /// Transaction ID this receipt belongs to
    pub tx_id: PersistTxId,
    /// Block height where transaction was included
    pub block_height: u64,
    /// Legacy compatibility field carrying the checkpoint identifier hex.
    pub block_hash: String,
    /// Transaction index within the block
    pub tx_index: u32,
    /// Number of confirmations (blocks after this one)
    pub confirmations: u32,
    /// Timestamp when transaction was confirmed (Unix milliseconds)
    pub confirmed_at: u64,
    /// Whether receipt has been verified against blockchain
    pub verified: bool,
    /// Reserved compatibility field. Canonical live receipts leave this empty
    /// instead of publishing proof-shaped placeholder strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merkle_proof: Option<String>,
}

/// Send transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeSendTxResponse {
    pub tx_id: PersistTxId,
    pub status: TxStatus,
    pub lifecycle: RuntimeTxLifecycle,
}

/// Transaction history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTxHistoryResponse {
    pub wallet_id: PersistWalletId,
    pub transactions: Vec<PersistTxInfo>,
    pub total_count: usize,
}

/// Transaction details response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTxDetailsResponse {
    pub tx_id: PersistTxId,
    pub wallet_id: PersistWalletId,
    pub status: TxStatus,
    pub lifecycle: RuntimeTxLifecycle,
    pub amount: u64,
    pub fee: u64,
    pub inputs: Vec<AssetId>,
    pub outputs: Vec<AssetId>,
    /// Transaction creation timestamp (Unix milliseconds).
    pub timestamp: u64,
    pub confirmations: u32,
    /// Optional receipt information (if transaction confirmed)
    pub receipt: Option<PersistReceiptInfo>,
    /// Whether receipt has been verified (if present)
    pub receipt_verified: bool,
}

/// List pending transactions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeListPendingTxResponse {
    pub transactions: Vec<PersistTxInfo>,
    pub count: usize,
}

/// Cancel transaction response
pub type RuntimeCancelTxResponse = super::common::RuntimeOperationStatusWithTx;

/// Broadcast transaction response
pub type RuntimeBroadcastTxResponse = super::common::RuntimeOperationStatusWithTx;

/// Build transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeBuildTxResponse {
    pub tx_id: PersistTxId,
    pub raw_tx: String,
}

/// Wallet-owned output found inside a verified tx package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVerifyTxPkgOut {
    pub asset_id_hex: String,
    pub serial_id: u32,
    pub amount: u64,
    pub can_spend: bool,
    pub asset_data: String,
}

/// Verify prepared transaction package response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeVerifyTxPkgResponse {
    pub tx_digest_hex: String,
    pub package_status: String,
    pub is_valid: bool,
    pub lifecycle: RuntimeTxLifecycle,
    pub import_ready: bool,
    pub all_owned_spendable: bool,
    pub owned_outputs: Vec<RuntimeVerifyTxPkgOut>,
    pub errors: Vec<String>,
    pub error_codes: Vec<RuntimeTxErrorCode>,
}

/// Portable PH44 transaction package for offline transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortableWalletTxPackage {
    pub package_version: u16,
    pub chain_id: String,
    pub tx_hash_hex: String,
    pub tx_bytes: Vec<u8>,
    pub metadata_hash_hex: String,
}

/// Import transaction response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeImportTxResponse {
    pub tx_id: PersistTxId,
    pub status: TxStatus,
    pub lifecycle: RuntimeTxLifecycle,
    pub imported_outputs: Vec<RuntimeVerifyTxPkgOut>,
    pub error_codes: Vec<RuntimeTxErrorCode>,
}

/// Reconcile transaction response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeReconcileTxResponse {
    pub tx_id: PersistTxId,
    pub status: TxStatus,
    pub lifecycle: RuntimeTxLifecycle,
    pub confirmation: RuntimeConfirmationReceipt,
}

/// Estimate fee response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeEstimateFeeResponse {
    pub estimated_fee: u64,
    pub fee_per_byte: u64,
}

/// Export transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeExportTxResponse {
    pub success: bool,
    pub export_path: Option<String>,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use z00z_utils::codec::{Codec, JsonCodec, Value};

    #[test]
    fn test_receipt_info_serialization() {
        let receipt = PersistReceiptInfo {
            tx_id: PersistTxId::new("tx123".to_string()),
            block_height: 1000,
            block_hash: "0xabcdef".to_string(),
            tx_index: 5,
            confirmations: 10,
            confirmed_at: 1_703_260_800_000,
            verified: true,
            merkle_proof: None,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&receipt).unwrap();
        let deserialized: PersistReceiptInfo = codec.deserialize(&bytes).unwrap();
        let json: Value = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.tx_id.0, "tx123");
        assert_eq!(deserialized.block_height, 1000);
        assert_eq!(deserialized.confirmations, 10);
        assert!(deserialized.verified);
        assert!(deserialized.merkle_proof.is_none());
        assert!(
            !json.as_object().unwrap().contains_key("merkle_proof"),
            "canonical receipt summaries must omit placeholder proof fields"
        );
    }

    #[test]
    fn test_receipt_proof_optional() {
        let receipt = PersistReceiptInfo {
            tx_id: PersistTxId::new("tx456".to_string()),
            block_height: 2000,
            block_hash: "0xfedcba".to_string(),
            tx_index: 0,
            confirmations: 100,
            confirmed_at: 1_703_347_200_000,
            verified: false,
            merkle_proof: None,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&receipt).unwrap();
        let deserialized: PersistReceiptInfo = codec.deserialize(&bytes).unwrap();
        let json: Value = codec.deserialize(&bytes).unwrap();

        assert_eq!(deserialized.block_height, 2000);
        assert!(!deserialized.verified);
        assert!(deserialized.merkle_proof.is_none());
        assert!(
            !json.as_object().unwrap().contains_key("merkle_proof"),
            "receipt summaries must not expose absent compatibility proof fields"
        );
    }

    #[test]
    fn test_tx_info_with_receipt() {
        let receipt = PersistReceiptInfo {
            tx_id: PersistTxId::new("tx789".to_string()),
            block_height: 1500,
            block_hash: "0x123abc".to_string(),
            tx_index: 3,
            confirmations: 50,
            confirmed_at: 1_703_304_000_000,
            verified: true,
            merkle_proof: None,
        };

        let tx_info = PersistTxInfo {
            id: PersistTxId::new("tx789".to_string()),
            wallet_id: PersistWalletId("wallet1".to_string()),
            status: TxStatus::Confirmed,
            lifecycle: RuntimeTxLifecycle::Confirmed,
            amount: 1000,
            fee: 10,
            timestamp: 1_703_304_000_000,
            receipt: Some(receipt),
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&tx_info).unwrap();
        let deserialized: PersistTxInfo = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.receipt.is_some());
        assert_eq!(deserialized.receipt.unwrap().block_height, 1500);
    }

    #[test]
    fn test_tx_info_without_receipt() {
        let tx_info = PersistTxInfo {
            id: PersistTxId::new("pending-tx".to_string()),
            wallet_id: PersistWalletId("wallet2".to_string()),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Created,
            amount: 500,
            fee: 5,
            timestamp: 1_703_304_100_000,
            receipt: None,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&tx_info).unwrap();
        let deserialized: PersistTxInfo = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.receipt.is_none());
        assert!(matches!(deserialized.status, TxStatus::Pending));
        assert_eq!(deserialized.lifecycle, RuntimeTxLifecycle::Created);
    }

    #[test]
    fn test_tx_details_response_receipt() {
        let receipt = PersistReceiptInfo {
            tx_id: PersistTxId::new("detail-tx".to_string()),
            block_height: 3000,
            block_hash: "0xaabbcc".to_string(),
            tx_index: 7,
            confirmations: 25,
            confirmed_at: 1_703_318_400_000,
            verified: true,
            merkle_proof: None,
        };

        let response = RuntimeTxDetailsResponse {
            tx_id: PersistTxId::new("detail-tx".to_string()),
            wallet_id: PersistWalletId("wallet3".to_string()),
            status: TxStatus::Confirmed,
            lifecycle: RuntimeTxLifecycle::Confirmed,
            amount: 2000,
            fee: 20,
            inputs: vec![[1u8; 32]],
            outputs: vec![[2u8; 32]],
            timestamp: 1_703_318_400_000,
            confirmations: 25,
            receipt: Some(receipt),
            receipt_verified: true,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeTxDetailsResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.receipt.is_some());
        assert!(deserialized.receipt_verified);
        assert_eq!(deserialized.receipt.unwrap().confirmations, 25);
    }

    #[test]
    fn test_confirm_receipt_roots() {
        let confirmation = RuntimeConfirmationReceipt {
            tx_id: PersistTxId::new("confirm-tx".to_string()),
            tx_hash_hex: "aa".repeat(32),
            block_height: 42,
            checkpoint_id_hex: "11".repeat(32),
            prev_root_hex: "22".repeat(32),
            new_root_hex: "33".repeat(32),
            spent_asset_ids_hex: vec!["44".repeat(32)],
            created_asset_ids_hex: vec!["55".repeat(32)],
            confirmed_at: 1_703_318_600_000,
            verified: true,
        };

        let bytes = JsonCodec.serialize(&confirmation).unwrap();
        let json: Value = JsonCodec.deserialize(&bytes).unwrap();
        let object = json.as_object().expect("confirmation receipt object");

        assert!(object.contains_key("checkpoint_id_hex"));
        assert!(object.contains_key("prev_root_hex"));
        assert!(object.contains_key("new_root_hex"));
    }

    #[test]
    fn test_tx_details_no_receipt() {
        let response = RuntimeTxDetailsResponse {
            tx_id: PersistTxId::new("pending-detail".to_string()),
            wallet_id: PersistWalletId("wallet4".to_string()),
            status: TxStatus::Pending,
            lifecycle: RuntimeTxLifecycle::Admitted,
            amount: 750,
            fee: 7,
            inputs: Vec::new(),
            outputs: Vec::new(),
            timestamp: 1_703_318_500_000,
            confirmations: 0,
            receipt: None,
            receipt_verified: false,
        };

        let codec = JsonCodec;
        let bytes = codec.serialize(&response).unwrap();
        let deserialized: RuntimeTxDetailsResponse = codec.deserialize(&bytes).unwrap();

        assert!(deserialized.receipt.is_none());
        assert!(!deserialized.receipt_verified);
        assert_eq!(deserialized.confirmations, 0);
        assert_eq!(deserialized.lifecycle, RuntimeTxLifecycle::Admitted);
    }

    #[test]
    fn test_lifecycle_uses_snake_case() {
        let codec = JsonCodec;
        let cases = [
            (RuntimeTxLifecycle::Created, "\"created\""),
            (RuntimeTxLifecycle::Imported, "\"imported\""),
            (RuntimeTxLifecycle::Exported, "\"exported\""),
            (RuntimeTxLifecycle::Submitted, "\"submitted\""),
            (RuntimeTxLifecycle::Admitted, "\"admitted\""),
            (RuntimeTxLifecycle::Confirmed, "\"confirmed\""),
            (RuntimeTxLifecycle::Failed, "\"failed\""),
            (RuntimeTxLifecycle::Cancelled, "\"cancelled\""),
            (RuntimeTxLifecycle::Conflicted, "\"conflicted\""),
            (RuntimeTxLifecycle::AlreadySpent, "\"already_spent\""),
        ];

        for (value, expected) in cases {
            let bytes = codec.serialize(&value).unwrap();
            assert_eq!(std::str::from_utf8(&bytes).unwrap(), expected);
            let decoded: RuntimeTxLifecycle = codec.deserialize(&bytes).unwrap();
            assert_eq!(decoded, value);
        }
    }

    #[test]
    fn test_error_code_snake_case() {
        let codec = JsonCodec;
        let cases = [
            (RuntimeTxErrorCode::InvalidEncoding, "\"invalid_encoding\""),
            (RuntimeTxErrorCode::InvalidPackage, "\"invalid_package\""),
            (RuntimeTxErrorCode::InvalidDigest, "\"invalid_digest\""),
            (
                RuntimeTxErrorCode::UnsupportedPackageVersion,
                "\"unsupported_package_version\"",
            ),
            (
                RuntimeTxErrorCode::UnsupportedReceiveVersion,
                "\"unsupported_receive_version\"",
            ),
            (RuntimeTxErrorCode::WrongChain, "\"wrong_chain\""),
            (
                RuntimeTxErrorCode::InvalidPublicSpendProof,
                "\"invalid_public_spend_proof\"",
            ),
            (RuntimeTxErrorCode::NoOwnedOutputs, "\"no_owned_outputs\""),
            (RuntimeTxErrorCode::NotImportReady, "\"not_import_ready\""),
            (
                RuntimeTxErrorCode::DuplicateConflict,
                "\"duplicate_conflict\"",
            ),
            (RuntimeTxErrorCode::AlreadySpent, "\"already_spent\""),
            (RuntimeTxErrorCode::CursorConflict, "\"cursor_conflict\""),
            (
                RuntimeTxErrorCode::WorkerEvidenceRejected,
                "\"worker_evidence_rejected\"",
            ),
            (RuntimeTxErrorCode::InternalError, "\"internal_error\""),
        ];

        for (value, expected) in cases {
            let bytes = codec.serialize(&value).unwrap();
            assert_eq!(std::str::from_utf8(&bytes).unwrap(), expected);
            let decoded: RuntimeTxErrorCode = codec.deserialize(&bytes).unwrap();
            assert_eq!(decoded, value);
        }
    }
}
