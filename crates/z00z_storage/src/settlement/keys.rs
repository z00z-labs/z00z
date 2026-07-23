use jmt::KeyHash;
use z00z_crypto::expert::hash_domain;
use z00z_crypto::hash_zk::hash_zk;
use z00z_crypto::poseidon2_framed_words_v1;

use super::{DefinitionId, SerialId, TerminalId};

hash_domain!(StorDefKeyDom, "z00z.storage.key.definition.v1", 1);
hash_domain!(StorSerKeyDom, "z00z.storage.key.serial.v1", 1);

#[must_use]
pub fn definition_key(def_id: DefinitionId) -> KeyHash {
    KeyHash(hash_zk::<StorDefKeyDom>("", &[def_id.as_bytes()]))
}

#[must_use]
pub fn serial_key(def_id: DefinitionId, serial_id: SerialId) -> KeyHash {
    let serial = serial_id.get().to_le_bytes();
    KeyHash(hash_zk::<StorSerKeyDom>("", &[def_id.as_bytes(), &serial]))
}

/// Return the exact project-owned Poseidon2 word framing used by the two
/// hierarchy parent keys. Recursive backends consume this owner instead of
/// reconstructing the byte/domain grammar beside the live storage keys.
pub(crate) fn hierarchy_parent_poseidon2_words_v1(
    definition: bool,
    definition_id: [u8; 32],
    serial_id: u32,
) -> Vec<u64> {
    let empty_context = b"";
    if definition {
        poseidon2_framed_words_v1(
            b"z00z.storage.key.definition.v1",
            &[empty_context, &definition_id],
        )
    } else {
        let serial = serial_id.to_le_bytes();
        poseidon2_framed_words_v1(
            b"z00z.storage.key.serial.v1",
            &[empty_context, &definition_id, &serial],
        )
    }
}

#[must_use]
pub fn terminal_key(terminal_id: TerminalId) -> KeyHash {
    KeyHash(terminal_id.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::hierarchy_parent_poseidon2_words_v1;

    #[test]
    fn test_parent_key_frame_geometry() {
        let definition = hierarchy_parent_poseidon2_words_v1(true, [0; 32], 0);
        let serial = hierarchy_parent_poseidon2_words_v1(false, [0; 32], 0);
        eprintln!(
            "hierarchy Poseidon2 frame words: definition={}, serial={}",
            definition.len(),
            serial.len()
        );
        assert_eq!(definition.len(), 13);
        assert_eq!(serial.len(), 13);
    }
}
