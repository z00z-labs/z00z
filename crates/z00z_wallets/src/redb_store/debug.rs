#[cfg(all(feature = "wallet_debug_tools", test))]
use z00z_utils::codec::Value;

#[path = "debug_export.rs"]
mod debug_export;
#[path = "debug_types.rs"]
mod debug_types;

#[cfg(feature = "wallet_debug_tools")]
pub(crate) use self::debug_export::debug_export_wallet;
#[cfg(feature = "wallet_debug_tools")]
pub(crate) use self::debug_types::{
    DebugIndexKey, DebugMetaEntry, DebugObjectEntry, DebugSecretEntry, DebugTableRow,
    DebugWalletDump,
};
#[cfg(all(feature = "wallet_debug_tools", test))]
pub(crate) fn decode_object_json(kind_id: u8, payload_version: u16, data: &[u8]) -> Option<Value> {
    self::debug_types::decode_object_json(kind_id, payload_version, data)
}
