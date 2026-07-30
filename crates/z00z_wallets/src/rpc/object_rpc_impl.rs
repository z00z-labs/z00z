use async_trait::async_trait;
use jsonrpsee::core::RpcResult;
use jsonrpsee::types::ErrorObjectOwned;
use z00z_core::{actions::LifecycleEffectV1, assets::ObjectFamily};
use z00z_storage::settlement::{
    inspect_object_package, ObjectPolicyRegistryV1, ObjectRejectCode, ObjectValidatorVerdict,
    RightAction, RuntimeObjectPackageV1, SettlementActionV1, SettlementLeaf, SettlementStateRoot,
    TerminalId, VoucherAction, VoucherBackingRef,
};

use super::{object_rpc::ObjectRpcServer, AssetRpcImpl};
use crate::db::redb_store::OwnedAssetStatus;
use crate::db::{
    ObjectInventoryFilter, ObjectInventoryPage, OwnedAssetPayload, OwnedObjectFamily,
    OwnedObjectPayload, OwnedRightStatus, OwnedVoucherStatus, WalletOwnedObject,
    WalletPolicyAvailability,
};
use crate::rpc::error_mapping::map_wallet_error_to_rpc;
use crate::rpc::methods::tx_rpc_support;
use crate::rpc::types::object::{
    RuntimeAssetObjectDetail, RuntimeListObjectsResponse, RuntimeListRightInventoryResponse,
    RuntimeListVoucherClaimsResponse, RuntimeObjectListFilter, RuntimeObjectPackageBuildResponse,
    RuntimeObjectPackagePreviewResponse, RuntimeObjectPackageRequest, RuntimeObjectPolicyState,
    RuntimeObjectRecord, RuntimeRightObjectDetail, RuntimeVoucherObjectDetail,
};

const OBJECT_LIST_DEFAULT_LIMIT: usize = 50;
const OBJECT_LIST_MAX_LIMIT: usize = 200;

fn decode_cursor_offset(cursor: Option<&str>) -> RpcResult<usize> {
    match cursor {
        None => Ok(0),
        Some(raw) => raw.parse::<usize>().map_err(|_| {
            ErrorObjectOwned::owned(
                -32602,
                "Invalid cursor: expected offset".to_string(),
                None::<()>,
            )
        }),
    }
}

fn clamp_limit(limit: Option<usize>) -> usize {
    limit
        .unwrap_or(OBJECT_LIST_DEFAULT_LIMIT)
        .clamp(1, OBJECT_LIST_MAX_LIMIT)
}

fn decode_hex32(value: &str, field: &str) -> RpcResult<[u8; 32]> {
    let bytes = hex::decode(value).map_err(|_| {
        ErrorObjectOwned::owned(
            -32602,
            format!("Invalid {field}: expected hex bytes"),
            None::<()>,
        )
    })?;
    <[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| {
        ErrorObjectOwned::owned(
            -32602,
            format!("Invalid {field}: expected 32 bytes"),
            None::<()>,
        )
    })
}

fn policy_state(policy: &crate::db::OwnedObjectPolicy) -> RuntimeObjectPolicyState {
    RuntimeObjectPolicyState {
        policy_id_hex: policy.policy_id.map(hex::encode),
        availability: policy.availability,
        manual_review: policy.manual_review,
        quarantine_reason: policy.quarantine_reason.clone(),
    }
}

fn object_record_from_payload(object: WalletOwnedObject) -> RuntimeObjectRecord {
    match object.payload {
        OwnedObjectPayload::Asset(payload) => RuntimeObjectRecord {
            object_id: object.object_id,
            wallet_id: payload.wallet_id,
            account_id: payload.account_id,
            family: crate::db::OwnedObjectFamily::Asset,
            stable_id_hex: hex::encode(payload.asset_id),
            labels: payload.labels,
            last_updated_ms: payload.last_updated_ms,
            policy: None,
            asset: Some(RuntimeAssetObjectDetail {
                status: payload.status,
                source: payload.source,
                asset_wire: payload.asset_wire,
                spend_ref: payload.spend_ref,
                quarantined: payload.policy.frozen || payload.policy.quarantine_reason.is_some(),
                quarantine_reason: payload.policy.quarantine_reason,
            }),
            voucher: None,
            right: None,
        },
        OwnedObjectPayload::Voucher(payload) => RuntimeObjectRecord {
            object_id: object.object_id,
            wallet_id: payload.wallet_id,
            account_id: payload.account_id,
            family: crate::db::OwnedObjectFamily::Voucher,
            stable_id_hex: hex::encode(payload.terminal_id.as_bytes()),
            labels: payload.labels,
            last_updated_ms: payload.last_updated_ms,
            policy: Some(policy_state(&payload.policy)),
            asset: None,
            voucher: Some(RuntimeVoucherObjectDetail {
                status: payload.status,
                source: payload.source,
                voucher_leaf: payload.voucher_leaf,
            }),
            right: None,
        },
        OwnedObjectPayload::Right(payload) => RuntimeObjectRecord {
            object_id: object.object_id,
            wallet_id: payload.wallet_id,
            account_id: payload.account_id,
            family: crate::db::OwnedObjectFamily::Right,
            stable_id_hex: hex::encode(payload.terminal_id.as_bytes()),
            labels: payload.labels,
            last_updated_ms: payload.last_updated_ms,
            policy: Some(policy_state(&payload.policy)),
            asset: None,
            voucher: None,
            right: Some(RuntimeRightObjectDetail {
                status: payload.status,
                source: payload.source,
                right_leaf: payload.right_leaf,
            }),
        },
    }
}

fn paginated_object_response(
    page: ObjectInventoryPage,
    start: usize,
    limit: usize,
) -> RuntimeListObjectsResponse {
    let has_more = page.items.len() > limit;
    let next_cursor = has_more.then(|| start.saturating_add(limit).to_string());
    let items = page
        .items
        .into_iter()
        .take(limit)
        .map(object_record_from_payload)
        .collect();

    RuntimeListObjectsResponse {
        items,
        next_cursor,
        has_more,
        total_count: None,
    }
}

fn rpc_reject_code(code: ObjectRejectCode) -> &'static str {
    match code {
        ObjectRejectCode::UnknownPolicy => "OBJECT_UNKNOWN_POLICY",
        ObjectRejectCode::UnknownAction => "OBJECT_UNKNOWN_ACTION",
        ObjectRejectCode::InvalidBacking => "OBJECT_INVALID_BACKING",
        ObjectRejectCode::WrongFamilyProof => "OBJECT_WRONG_FAMILY_PROOF",
        ObjectRejectCode::VoucherUsedAsCash => "OBJECT_VOUCHER_USED_AS_CASH",
        ObjectRejectCode::RightUsedAsValue => "OBJECT_RIGHT_USED_AS_VALUE",
        ObjectRejectCode::MissingRight => "OBJECT_MISSING_RIGHT",
        ObjectRejectCode::RightOutOfScope => "OBJECT_RIGHT_OUT_OF_SCOPE",
        ObjectRejectCode::RightExpired => "OBJECT_RIGHT_EXPIRED",
        ObjectRejectCode::RightRevoked => "OBJECT_RIGHT_REVOKED",
        ObjectRejectCode::RightConsumed => "OBJECT_RIGHT_CONSUMED",
        ObjectRejectCode::Replay => "OBJECT_REPLAY",
        ObjectRejectCode::DoubleRedeem => "OBJECT_DOUBLE_REDEEM",
        ObjectRejectCode::ResidualMismatch => "OBJECT_RESIDUAL_MISMATCH",
        ObjectRejectCode::ForcedAcceptance => "OBJECT_FORCED_ACCEPTANCE",
        ObjectRejectCode::StaleRoot => "OBJECT_STALE_ROOT",
        ObjectRejectCode::FeeBoundary => "OBJECT_FEE_BOUNDARY",
        ObjectRejectCode::MissingSignature => "OBJECT_MISSING_SIGNATURE",
        ObjectRejectCode::MissingAttestation => "OBJECT_MISSING_ATTESTATION",
        ObjectRejectCode::ExpiredVoucherUse => "OBJECT_EXPIRED_VOUCHER_USE",
    }
}

fn reject_object_package(code: ObjectRejectCode) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, rpc_reject_code(code).to_string(), None::<()>)
}

fn invalid_object_request(code: &str) -> ErrorObjectOwned {
    ErrorObjectOwned::owned(-32602, code.to_string(), None::<()>)
}

fn package_family(object: &WalletOwnedObject) -> ObjectFamily {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => ObjectFamily::Asset,
        OwnedObjectPayload::Voucher(_) => ObjectFamily::Voucher,
        OwnedObjectPayload::Right(_) => ObjectFamily::Right,
    }
}

fn package_family_db(object: &WalletOwnedObject) -> OwnedObjectFamily {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => OwnedObjectFamily::Asset,
        OwnedObjectPayload::Voucher(_) => OwnedObjectFamily::Voucher,
        OwnedObjectPayload::Right(_) => OwnedObjectFamily::Right,
    }
}

fn current_policy_id(object: &WalletOwnedObject) -> Option<[u8; 32]> {
    object.payload.policy().and_then(|policy| policy.policy_id)
}

fn current_leaf(object: &WalletOwnedObject) -> Option<SettlementLeaf> {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => None,
        OwnedObjectPayload::Voucher(payload) => {
            Some(SettlementLeaf::Voucher(payload.voucher_leaf.clone()))
        }
        OwnedObjectPayload::Right(payload) => Some(SettlementLeaf::Right(payload.right_leaf)),
    }
}

fn current_policy_availability(object: &WalletOwnedObject) -> WalletPolicyAvailability {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => WalletPolicyAvailability::Available,
        OwnedObjectPayload::Voucher(payload) => payload.policy.availability,
        OwnedObjectPayload::Right(payload) => payload.policy.availability,
    }
}

fn current_manual_review(object: &WalletOwnedObject) -> bool {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => false,
        OwnedObjectPayload::Voucher(payload) => payload.policy.manual_review,
        OwnedObjectPayload::Right(payload) => payload.policy.manual_review,
    }
}

// Runtime object packaging has exactly two selector lanes that do not start
// from an existing live object: voucher issue selects explicit backing
// (owned asset or reserve), and right create selects a terminal context.
// There is no generic post-genesis asset-mint selector on the public RPC.
enum PackageTarget {
    LiveObject(WalletOwnedObject),
    IssueAsset(OwnedAssetPayload),
    IssueReserve([u8; 32]),
    CreateContext(TerminalId),
}

impl PackageTarget {
    fn primary_family(&self) -> ObjectFamily {
        match self {
            Self::LiveObject(object) => package_family(object),
            Self::IssueAsset(_) | Self::IssueReserve(_) => ObjectFamily::Voucher,
            Self::CreateContext(_) => ObjectFamily::Right,
        }
    }

    fn response_family(&self) -> OwnedObjectFamily {
        match self {
            Self::LiveObject(object) => package_family_db(object),
            Self::IssueAsset(_) | Self::IssueReserve(_) => OwnedObjectFamily::Voucher,
            Self::CreateContext(_) => OwnedObjectFamily::Right,
        }
    }

    fn response_stable_id_hex(&self, package: &RuntimeObjectPackageV1) -> String {
        match self {
            Self::LiveObject(object) => hex::encode(object.payload.stable_object_key()),
            Self::IssueAsset(_) | Self::IssueReserve(_) => created_voucher_id_hex(package),
            Self::CreateContext(terminal_id) => hex::encode(terminal_id.as_bytes()),
        }
    }
}

fn delta_references_live_object(
    object: &WalletOwnedObject,
    package: &RuntimeObjectPackageV1,
) -> bool {
    let Some(expected) = current_leaf(object) else {
        return false;
    };

    package
        .delta_set
        .deleted_objects
        .iter()
        .chain(package.delta_set.updated_objects.iter())
        .any(|delta| delta.prior_leaf.as_ref() == Some(&expected))
}

fn confirmation_root_matches_state(
    state_root_hex: Option<&str>,
    root: SettlementStateRoot,
) -> bool {
    let Some(state_root_hex) = state_root_hex else {
        return true;
    };
    let Ok(bytes) = hex::decode(state_root_hex) else {
        return false;
    };
    let Ok(bytes) = <[u8; 32]>::try_from(bytes.as_slice()) else {
        return false;
    };
    root == SettlementStateRoot::settlement_v1(bytes)
}

fn confirmation_root_matches(object: &WalletOwnedObject, root: SettlementStateRoot) -> bool {
    let state_root_hex = match &object.payload {
        OwnedObjectPayload::Asset(_) => return true,
        OwnedObjectPayload::Voucher(payload) => payload
            .confirmation_ref
            .as_ref()
            .and_then(|item| item.state_root_hex.as_deref()),
        OwnedObjectPayload::Right(payload) => payload
            .confirmation_ref
            .as_ref()
            .and_then(|item| item.state_root_hex.as_deref()),
    };
    confirmation_root_matches_state(state_root_hex, root)
}

fn asset_confirmation_root_matches(asset: &OwnedAssetPayload, root: SettlementStateRoot) -> bool {
    confirmation_root_matches_state(
        asset
            .confirmation_ref
            .as_ref()
            .and_then(|item| item.state_root_hex.as_deref()),
        root,
    )
}

fn delta_references_issue_asset(
    asset: &OwnedAssetPayload,
    package: &RuntimeObjectPackageV1,
) -> bool {
    package.delta_set.deleted_objects.iter().any(|delta| {
        delta.path.terminal_id.into_bytes() == asset.asset_id
            && delta.path.definition_id.into_bytes() == asset.asset_definition_id
            && delta.path.serial_id.get() == asset.asset_wire.serial_id
    })
}

fn issue_source_available(asset: &OwnedAssetPayload) -> bool {
    asset.status == OwnedAssetStatus::Spendable
        && asset.spend_ref.is_none()
        && !asset.policy.frozen
        && !asset.policy.manual_review
        && asset.policy.quarantine_reason.is_none()
}

fn created_voucher_backing(package: &RuntimeObjectPackageV1) -> Option<VoucherBackingRef> {
    package
        .delta_set
        .created_objects
        .iter()
        .find_map(|delta| match delta.next_leaf.as_ref() {
            Some(SettlementLeaf::Voucher(voucher)) => Some(voucher.backing),
            _ => None,
        })
}

fn created_voucher_id_hex(package: &RuntimeObjectPackageV1) -> String {
    package
        .delta_set
        .created_objects
        .iter()
        .find_map(|delta| match delta.next_leaf.as_ref() {
            Some(SettlementLeaf::Voucher(voucher)) => {
                Some(hex::encode(voucher.terminal_id.as_bytes()))
            }
            _ => None,
        })
        .unwrap_or_default()
}

fn create_context_matches(package: &RuntimeObjectPackageV1, terminal_id: TerminalId) -> bool {
    package
        .delta_set
        .created_objects
        .iter()
        .any(|delta| match delta.next_leaf.as_ref() {
            Some(SettlementLeaf::Right(right)) => {
                delta.path.terminal_id == terminal_id && right.terminal_id == terminal_id
            }
            _ => false,
        })
}

fn action_effect_matches(selected_action: SettlementActionV1, effect: LifecycleEffectV1) -> bool {
    matches!(
        (selected_action, effect),
        (
            SettlementActionV1::Voucher(VoucherAction::Accept),
            LifecycleEffectV1::Accept
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Reject),
            LifecycleEffectV1::Reject | LifecycleEffectV1::Refund
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Transfer),
            LifecycleEffectV1::Transfer
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Transfer),
            LifecycleEffectV1::Offer
        ) | (
            SettlementActionV1::Voucher(VoucherAction::RedeemFull),
            LifecycleEffectV1::Redeem
        ) | (
            SettlementActionV1::Voucher(VoucherAction::RedeemPartial),
            LifecycleEffectV1::PartialRedeem,
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Refund),
            LifecycleEffectV1::Refund
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Expire),
            LifecycleEffectV1::Expire
        ) | (
            SettlementActionV1::Voucher(VoucherAction::Issue),
            LifecycleEffectV1::Issue | LifecycleEffectV1::Offer
        ) | (
            SettlementActionV1::Right(RightAction::Create),
            LifecycleEffectV1::Grant
        ) | (
            SettlementActionV1::Right(RightAction::Transfer),
            LifecycleEffectV1::Delegate
        ) | (
            SettlementActionV1::Right(RightAction::Consume),
            LifecycleEffectV1::Use
        ) | (
            SettlementActionV1::Right(RightAction::Expire),
            LifecycleEffectV1::Expire
        ) | (
            SettlementActionV1::Right(RightAction::Revoke),
            LifecycleEffectV1::Revoke
        ) | (
            SettlementActionV1::Right(RightAction::Challenge),
            LifecycleEffectV1::Challenge
        )
    )
}

#[cfg(test)]
mod test_action_effect_matches {
    use super::*;

    #[test]
    fn voucher_issue_accepts_exact_and_legacy_effects() {
        let action = SettlementActionV1::Voucher(VoucherAction::Issue);
        assert!(action_effect_matches(action, LifecycleEffectV1::Issue));
        assert!(action_effect_matches(action, LifecycleEffectV1::Offer));
        assert!(!action_effect_matches(action, LifecycleEffectV1::Create));
    }

    #[test]
    fn voucher_reject_accepts_exact_and_legacy_effects() {
        let action = SettlementActionV1::Voucher(VoucherAction::Reject);
        assert!(action_effect_matches(action, LifecycleEffectV1::Reject));
        assert!(action_effect_matches(action, LifecycleEffectV1::Refund));
        assert!(!action_effect_matches(action, LifecycleEffectV1::Cancel));
    }
}

fn wallet_status_reject(
    object: &WalletOwnedObject,
    selected_action: SettlementActionV1,
) -> Option<ObjectRejectCode> {
    match &object.payload {
        OwnedObjectPayload::Asset(_) => Some(ObjectRejectCode::WrongFamilyProof),
        OwnedObjectPayload::Voucher(payload) => match payload.status {
            OwnedVoucherStatus::Offered | OwnedVoucherStatus::PendingAccept => {
                match selected_action {
                    SettlementActionV1::Voucher(VoucherAction::Accept)
                    | SettlementActionV1::Voucher(VoucherAction::Reject)
                    | SettlementActionV1::Voucher(VoucherAction::Expire) => None,
                    _ => Some(ObjectRejectCode::ForcedAcceptance),
                }
            }
            OwnedVoucherStatus::Accepted | OwnedVoucherStatus::Redeemable => None,
            OwnedVoucherStatus::PartiallyRedeemed => match selected_action {
                SettlementActionV1::Voucher(VoucherAction::RedeemFull)
                | SettlementActionV1::Voucher(VoucherAction::RedeemPartial)
                | SettlementActionV1::Voucher(VoucherAction::Refund)
                | SettlementActionV1::Voucher(VoucherAction::Expire) => None,
                _ => Some(ObjectRejectCode::DoubleRedeem),
            },
            OwnedVoucherStatus::Redeemed => Some(ObjectRejectCode::DoubleRedeem),
            OwnedVoucherStatus::Rejected | OwnedVoucherStatus::Refunded => {
                Some(ObjectRejectCode::ForcedAcceptance)
            }
            OwnedVoucherStatus::Expired => Some(ObjectRejectCode::ExpiredVoucherUse),
            OwnedVoucherStatus::Quarantined => Some(ObjectRejectCode::UnknownPolicy),
        },
        OwnedObjectPayload::Right(payload) => match payload.status {
            OwnedRightStatus::Granted | OwnedRightStatus::Held | OwnedRightStatus::Delegated => {
                None
            }
            OwnedRightStatus::Consumed => Some(ObjectRejectCode::RightConsumed),
            OwnedRightStatus::Revoked => Some(ObjectRejectCode::RightRevoked),
            OwnedRightStatus::Expired => Some(ObjectRejectCode::RightExpired),
            OwnedRightStatus::Challenged => Some(ObjectRejectCode::RightConsumed),
            OwnedRightStatus::Quarantined => Some(ObjectRejectCode::UnknownPolicy),
        },
    }
}

fn inspect_with_registry(
    request: &RuntimeObjectPackageRequest,
    package: &RuntimeObjectPackageV1,
) -> RpcResult<ObjectValidatorVerdict> {
    let mut registry = ObjectPolicyRegistryV1::default();
    registry
        .register(
            request.policy_descriptor.clone(),
            request.action_pool.clone(),
        )
        .map_err(|error| {
            ErrorObjectOwned::owned(
                -32602,
                format!("OBJECT_POLICY_REGISTRY_INVALID:{error}"),
                None::<()>,
            )
        })?;
    Ok(inspect_object_package(
        package,
        &registry,
        package.prior_root,
        package.expected_new_root,
    ))
}

impl AssetRpcImpl {
    async fn list_objects_impl(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeObjectListFilter>,
    ) -> RpcResult<RuntimeListObjectsResponse> {
        let start = decode_cursor_offset(cursor.as_deref())?;
        let limit = clamp_limit(limit);
        let filter = match filter {
            None => ObjectInventoryFilter::default(),
            Some(filter) => ObjectInventoryFilter {
                account_id: filter.account_id,
                family: filter.family,
                status: None,
                policy_availability: filter.policy_availability,
                holder_commitment: filter
                    .holder_commitment_hex
                    .as_deref()
                    .map(|value| decode_hex32(value, "holder_commitment_hex"))
                    .transpose()?,
            },
        };

        let page = self
            .wallet_service()
            .list_wallet_inventory(&wallet_id, filter, cursor, limit.saturating_add(1))
            .await
            .map_err(map_wallet_error_to_rpc)?;
        Ok(paginated_object_response(page, start, limit))
    }

    async fn list_vouchers_impl(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedVoucherStatus>,
    ) -> RpcResult<RuntimeListVoucherClaimsResponse> {
        let start = decode_cursor_offset(cursor.as_deref())?;
        let limit = clamp_limit(limit);
        let payloads = self
            .wallet_service()
            .list_voucher_claim_rows(&wallet_id, status, cursor, limit.saturating_add(1))
            .await
            .map_err(map_wallet_error_to_rpc)?;
        let has_more = payloads.len() > limit;
        let next_cursor = has_more.then(|| start.saturating_add(limit).to_string());
        let items = payloads
            .into_iter()
            .take(limit)
            .map(|payload| {
                object_record_from_payload(WalletOwnedObject {
                    object_id: None,
                    payload: OwnedObjectPayload::Voucher(payload),
                })
            })
            .collect();

        Ok(RuntimeListVoucherClaimsResponse {
            items,
            next_cursor,
            has_more,
            total_count: None,
        })
    }

    async fn list_rights_impl(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedRightStatus>,
    ) -> RpcResult<RuntimeListRightInventoryResponse> {
        let start = decode_cursor_offset(cursor.as_deref())?;
        let limit = clamp_limit(limit);
        let payloads = self
            .wallet_service()
            .list_right_inventory_rows(&wallet_id, status, cursor, limit.saturating_add(1))
            .await
            .map_err(map_wallet_error_to_rpc)?;
        let has_more = payloads.len() > limit;
        let next_cursor = has_more.then(|| start.saturating_add(limit).to_string());
        let items = payloads
            .into_iter()
            .take(limit)
            .map(|payload| {
                object_record_from_payload(WalletOwnedObject {
                    object_id: None,
                    payload: OwnedObjectPayload::Right(payload),
                })
            })
            .collect();

        Ok(RuntimeListRightInventoryResponse {
            items,
            next_cursor,
            has_more,
            total_count: None,
        })
    }

    async fn load_object_action_target(
        &self,
        wallet_id: &crate::rpc::types::common::PersistWalletId,
        stable_id_hex: &str,
    ) -> RpcResult<WalletOwnedObject> {
        let stable_key = decode_hex32(stable_id_hex, "stable_id_hex")?;
        let object = self
            .wallet_service()
            .lookup_non_asset_owned_object(wallet_id, stable_key)
            .await
            .map_err(map_wallet_error_to_rpc)?;
        object.ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "OBJECT_NOT_FOUND".to_string(), None::<()>)
        })
    }

    async fn resolve_package_target(
        &self,
        wallet_id: &crate::rpc::types::common::PersistWalletId,
        selected_action: SettlementActionV1,
        request: &RuntimeObjectPackageRequest,
    ) -> RpcResult<PackageTarget> {
        let selector_count = usize::from(request.stable_id_hex.is_some())
            + usize::from(request.issue_asset_id_hex.is_some())
            + usize::from(request.issue_reserve_hex.is_some())
            + usize::from(request.create_terminal_id_hex.is_some());

        if selector_count > 1 {
            return Err(invalid_object_request("OBJECT_TARGET_AMBIGUOUS"));
        }

        match selected_action {
            SettlementActionV1::Voucher(VoucherAction::Issue) => {
                if let Some(asset_id_hex) = request.issue_asset_id_hex.as_deref() {
                    let asset_id = decode_hex32(asset_id_hex, "issue_asset_id_hex")?;
                    let asset = self
                        .wallet_service()
                        .lookup_owned_asset_payload(wallet_id, asset_id)
                        .await
                        .map_err(map_wallet_error_to_rpc)?;
                    return asset
                        .map(PackageTarget::IssueAsset)
                        .ok_or_else(|| invalid_object_request("OBJECT_ISSUE_SOURCE_NOT_FOUND"));
                }
                if let Some(reserve_hex) = request.issue_reserve_hex.as_deref() {
                    return Ok(PackageTarget::IssueReserve(decode_hex32(
                        reserve_hex,
                        "issue_reserve_hex",
                    )?));
                }
                if selector_count == 0 {
                    return Err(invalid_object_request("OBJECT_ISSUE_SOURCE_REQUIRED"));
                }
                Err(invalid_object_request("OBJECT_TARGET_INVALID"))
            }
            SettlementActionV1::Right(RightAction::Create) => {
                if let Some(terminal_hex) = request.create_terminal_id_hex.as_deref() {
                    return Ok(PackageTarget::CreateContext(TerminalId::new(decode_hex32(
                        terminal_hex,
                        "create_terminal_id_hex",
                    )?)));
                }
                if selector_count == 0 {
                    return Err(invalid_object_request("OBJECT_CREATE_CONTEXT_REQUIRED"));
                }
                Err(invalid_object_request("OBJECT_TARGET_INVALID"))
            }
            _ => {
                if let Some(stable_id_hex) = request.stable_id_hex.as_deref() {
                    return self
                        .load_object_action_target(wallet_id, stable_id_hex)
                        .await
                        .map(PackageTarget::LiveObject);
                }
                if selector_count == 0 {
                    return Err(invalid_object_request("OBJECT_TARGET_REQUIRED"));
                }
                Err(invalid_object_request("OBJECT_TARGET_INVALID"))
            }
        }
    }

    fn build_runtime_object_package(
        &self,
        primary_family: ObjectFamily,
        request: &RuntimeObjectPackageRequest,
        selected_action: SettlementActionV1,
    ) -> RpcResult<(RuntimeObjectPackageV1, bool)> {
        let policy_descriptor_hash = request
            .policy_descriptor
            .policy_id()
            .map_err(|error| {
                ErrorObjectOwned::owned(
                    -32602,
                    format!("OBJECT_POLICY_INVALID:{error}"),
                    None::<()>,
                )
            })?
            .bytes();
        let action_pool_id = request
            .action_pool
            .action_pool_id()
            .map_err(|error| {
                ErrorObjectOwned::owned(
                    -32602,
                    format!("OBJECT_ACTION_POOL_INVALID:{error}"),
                    None::<()>,
                )
            })?
            .bytes();

        let mut action_effect_valid = false;
        let selected_action_id = request
            .action_pool
            .actions
            .iter()
            .find(|action| action.label == request.action_label)
            .and_then(|action| {
                action_effect_valid =
                    action_effect_matches(selected_action, action.lifecycle_effect);
                action.action_id().ok().map(|id| id.bytes())
            })
            .unwrap_or([0u8; 32]);

        Ok((
            RuntimeObjectPackageV1 {
                primary_family,
                selected_action,
                selected_action_id,
                policy_descriptor_hash,
                action_pool_id,
                required_rights: request.required_rights.clone(),
                object_witnesses: request.object_witnesses.clone(),
                delta_set: request.delta_set.clone(),
                fee_support_ref: request
                    .delta_set
                    .fee_envelope
                    .and_then(|item| item.support_ref),
                prior_root: request.delta_set.prior_root,
                expected_new_root: request.delta_set.expected_new_root,
            },
            action_effect_valid,
        ))
    }

    async fn preview_object_package_impl(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
        forced_action: Option<SettlementActionV1>,
    ) -> RpcResult<RuntimeObjectPackagePreviewResponse> {
        tx_rpc_support::verify_session(self.wallet_service(), &session).await?;
        let wallet_id = session.wallet_id.clone();
        let selected_action = forced_action.or(request.selected_action).ok_or_else(|| {
            ErrorObjectOwned::owned(-32602, "OBJECT_ACTION_REQUIRED".to_string(), None::<()>)
        })?;
        let target = self
            .resolve_package_target(&wallet_id, selected_action, &request)
            .await?;
        let (package, action_effect_valid) =
            self.build_runtime_object_package(target.primary_family(), &request, selected_action)?;

        let verdict = match &target {
            PackageTarget::LiveObject(object) => {
                if !action_effect_valid {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownAction)
                } else if current_policy_availability(object) != WalletPolicyAvailability::Available
                    || current_manual_review(object)
                    || current_policy_id(object) != Some(package.policy_descriptor_hash)
                {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownPolicy)
                } else if !delta_references_live_object(object, &package) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::WrongFamilyProof)
                } else if !confirmation_root_matches(object, package.prior_root) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::StaleRoot)
                } else if let Some(reject) = wallet_status_reject(object, selected_action) {
                    ObjectValidatorVerdict::rejected(&package, reject)
                } else {
                    inspect_with_registry(&request, &package)?
                }
            }
            PackageTarget::IssueAsset(asset) => {
                if !action_effect_valid {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownAction)
                } else if !issue_source_available(asset) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownPolicy)
                } else if !delta_references_issue_asset(asset, &package) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::WrongFamilyProof)
                } else if !asset_confirmation_root_matches(asset, package.prior_root) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::StaleRoot)
                } else {
                    inspect_with_registry(&request, &package)?
                }
            }
            PackageTarget::IssueReserve(expected_reserve) => {
                if !action_effect_valid {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownAction)
                } else if !matches!(
                    created_voucher_backing(&package),
                    Some(
                        VoucherBackingRef::ReserveCommitment(backing)
                            | VoucherBackingRef::GenesisReserve(backing)
                    ) if backing == *expected_reserve
                ) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::InvalidBacking)
                } else {
                    inspect_with_registry(&request, &package)?
                }
            }
            PackageTarget::CreateContext(terminal_id) => {
                if !action_effect_valid {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::UnknownAction)
                } else if !create_context_matches(&package, *terminal_id) {
                    ObjectValidatorVerdict::rejected(&package, ObjectRejectCode::WrongFamilyProof)
                } else {
                    inspect_with_registry(&request, &package)?
                }
            }
        };

        Ok(RuntimeObjectPackagePreviewResponse {
            stable_id_hex: target.response_stable_id_hex(&package),
            family: target.response_family(),
            action_label: request.action_label,
            package,
            verdict,
        })
    }

    async fn build_object_package_impl(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
        forced_action: Option<SettlementActionV1>,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        let preview = self
            .preview_object_package_impl(session, request, forced_action)
            .await?;
        if let Some(reject) = preview.verdict.reject {
            return Err(reject_object_package(reject));
        }

        Ok(RuntimeObjectPackageBuildResponse {
            stable_id_hex: preview.stable_id_hex,
            family: preview.family,
            action_label: preview.action_label,
            package: preview.package,
        })
    }
}

#[async_trait]
impl ObjectRpcServer for AssetRpcImpl {
    async fn list_objects(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        filter: Option<RuntimeObjectListFilter>,
    ) -> RpcResult<RuntimeListObjectsResponse> {
        self.list_objects_impl(wallet_id, limit, cursor, filter)
            .await
    }

    async fn list_vouchers(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedVoucherStatus>,
    ) -> RpcResult<RuntimeListVoucherClaimsResponse> {
        self.list_vouchers_impl(wallet_id, limit, cursor, status)
            .await
    }

    async fn list_rights(
        &self,
        wallet_id: crate::rpc::types::common::PersistWalletId,
        limit: Option<usize>,
        cursor: Option<String>,
        status: Option<OwnedRightStatus>,
    ) -> RpcResult<RuntimeListRightInventoryResponse> {
        self.list_rights_impl(wallet_id, limit, cursor, status)
            .await
    }

    async fn preview_package(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackagePreviewResponse> {
        self.preview_object_package_impl(session, request, None)
            .await
    }

    async fn build_package(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(session, request, None).await
    }

    async fn accept_voucher(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Voucher(VoucherAction::Accept)),
        )
        .await
    }

    async fn reject_voucher(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Voucher(VoucherAction::Reject)),
        )
        .await
    }

    async fn redeem_voucher(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Voucher(VoucherAction::RedeemFull)),
        )
        .await
    }

    async fn refund_voucher(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Voucher(VoucherAction::Refund)),
        )
        .await
    }

    async fn transfer_voucher(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Voucher(VoucherAction::Transfer)),
        )
        .await
    }

    async fn delegate_right(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Right(RightAction::Transfer)),
        )
        .await
    }

    async fn consume_right(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Right(RightAction::Consume)),
        )
        .await
    }

    async fn revoke_right(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Right(RightAction::Revoke)),
        )
        .await
    }

    async fn challenge_right(
        &self,
        session: crate::rpc::types::wallet::SessionToken,
        request: RuntimeObjectPackageRequest,
    ) -> RpcResult<RuntimeObjectPackageBuildResponse> {
        self.build_object_package_impl(
            session,
            request,
            Some(SettlementActionV1::Right(RightAction::Challenge)),
        )
        .await
    }
}
