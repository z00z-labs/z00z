use super::*;
use z00z_crypto::expert::traits::DomainSeparation;

// Independent golden owned by this test module. Keep this literal separate
// from `definitions.rs`: deriving both sides from the production source would
// make domain changes self-accepting.
const FROZEN_DOMAIN_SNAPSHOT: &str = r#"
AddressChecksumDomain=z00z.wallets.address.checksum
AeadEnvelopeDomain=z00z.crypto.aead.envelope
ReceiverCacheHmacProdDomain=app/z00z_wallets/address/receiver_cache/production
ReceiverCacheHmacTestDomain=app/z00z_wallets/address/receiver_cache/test
CardEntryDomain=z00z.wallets.chain.card_entry
CipherSeedAadTagDomain=z00z.crypto.cipher_seed.aad
CipherSeedChecksumDomain=z00z.crypto.cipher_seed.checksum
EncKeyDomain=z00z.wallet.stealth.enc_key.prod
EncKeyTestDomain=z00z.wallet.stealth.enc_key.test
EncryptionChecksumDomain=z00z.wallets.encryption_checksum
IdentitySignatureDomain=z00z.wallet.stealth.identity_sig
IdentitySignatureTestDomain=z00z.wallet.stealth.identity_sig.test
IndexMacDomain=z00z.wallets.index_mac
KdhExtDomain=z00z.wallet.stealth.k_dh_ext.prod
KdhExtTestDomain=z00z.wallet.stealth.k_dh_ext.test
KdhFlexDomain=z00z.wallet.stealth.k_dh_flex.prod
KdhFlexTestDomain=z00z.wallet.stealth.k_dh_flex.test
MacKeyDomain=z00z.wallet.stealth.mac_key.prod
MacKeyTestDomain=z00z.wallet.stealth.mac_key.test
PackKeyProdDomain=z00z.wallet.stealth.pack_key.prod
PackNonceProdDomain=z00z.wallet.stealth.pack_nonce.prod
PasswordBloomDomain=z00z.wallets.password_bloom
PayRefDomain=z00z.wallets.tx.pay_ref
RecoverRDomain=z00z.wallet.stealth.recover_r.prod
RecoverRTestDomain=z00z.wallet.stealth.recover_r.test
RedbWalletDataKeyDomain=z00z.crypto.redb_wallet_crypto.hkdf.data_key
RedbWalletIndexKeyDomain=z00z.crypto.redb_wallet_crypto.hkdf.index_key
RedbWalletIntegrityKeyDomain=z00z.crypto.redb_wallet_crypto.hkdf.integrity_key
RetryDigestDomain=z00z.wallet.stealth.retry_digest.prod
RistrettoBridgeDomain=z00z.wallet.bip32_to_ristretto
SOutProdDomain=z00z.wallet.stealth.s_out.prod
SchnorrChallengeDomain=z00z.wallets.schnorr_challenge
SenderSaltDomain=z00z.wallet.stealth.sender_salt.prod
SenderSaltTestDomain=z00z.wallet.stealth.sender_salt.test
SnapshotChecksumDomain=z00z.wallets.snapshot_checksum
TxHashDomain=z00z.wallets.tx.hash
TxIdDomain=z00z.wallets.tx_id
WalletAddressHashProdDomain=z00z.wallets.address_hash.prod
WalletBIP44ChangeDomain=z00z.wallets.bip44.change
WalletBIP44Domain=z00z.wallets.bip44
WalletBIP44PaymentDomain=z00z.wallets.bip44.payment
WalletBackupAadTagDomain=z00z.crypto.wallet_backup.aad
WalletBackupChecksumDomain=z00z.crypto.wallet_backup.checksum
WalletBlindingDomain=z00z.wallets.blinding
WalletChangeDomain=z00z.wallets.change
WalletDbIndexDomain=z00z.wallets.db.index
WalletDhKeyHashProdDomain=z00z.wallet.stealth.dh_key.prod
WalletDhKeyHashTestDomain=z00z.wallet.stealth.dh_key.test
WalletEncryptionHkdfInfoDomain=z00z.wallets.encryption.hkdf_info
WalletEncryptionKeyDomain=z00z.wallets.encryption.key_derivation
WalletEphemeralRHashProdDomain=z00z.wallet.stealth.ephemeral_r.prod
WalletEphemeralRHashTestDomain=z00z.wallet.stealth.ephemeral_r.test
WalletFileIdDomain=z00z.wallets.file_id
WalletFingerprintDomain=z00z.wallets.fingerprint
WalletIdDomain=z00z.wallets.wallet.wallet_id
WalletIdentityKeyHashProdDomain=z00z.wallet.stealth.identity_key.prod
WalletIdentityKeyHashTestDomain=z00z.wallet.stealth.identity_key.test
WalletIntegrityDomain=z00z.wallets.wallet_integrity
WalletKeyDerivationDomain=z00z.wallets.key.derivation
WalletKeyFingerprintDomain=z00z.wallets.rpc.key_fingerprint
WalletLeafAdHashProdDomain=z00z.wallet.stealth.leaf_ad.prod
WalletLeafAdHashTestDomain=z00z.wallet.stealth.leaf_ad.test
WalletMasterKeyDomain=z00z.wallets.master_key
WalletMessageSigningDomain=z00z.wallets.message_signing
WalletOwnerTagHashProdDomain=z00z.wallet.stealth.owner_tag.prod
WalletOwnerTagHashTestDomain=z00z.wallet.stealth.owner_tag.test
WalletPasswordVerifierDomain=z00z.wallets.wallet_service.password_verifier
WalletPaymentDomain=z00z.wallets.payment
WalletReceiverIdHashProdDomain=z00z.wallet.stealth.receiver_id.prod
WalletReceiverIdHashTestDomain=z00z.wallet.stealth.receiver_id.test
WalletSeedSaltDomain=z00z.wallets.seed_salt
WalletSessionDomain=z00z.wallets.session
WalletSignNonceProdDomain=z00z.wallets.sign_nonce.prod
WalletSignNonceTestDomain=z00z.wallets.sign_nonce.test
WalletTag16HashProdDomain=z00z.wallet.stealth.tag16.prod
WalletTag16HashTestDomain=z00z.wallet.stealth.tag16.test
WalletViewKeyHashProdDomain=z00z.wallet.stealth.view_key.prod
WalletViewKeyHashTestDomain=z00z.wallet.stealth.view_key.test
Z00ZRedbWalletAadIdDomain=z00z.crypto.redb_wallet_crypto.wallet_aad_id
"#;

fn snapshot_lines_from_str(s: &str) -> Vec<String> {
    let mut lines: Vec<String> = s
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .filter(|line| !line.starts_with('#'))
        .map(ToString::to_string)
        .collect();
    lines.sort();
    lines
}

fn extract_first_string_literal(s: &str) -> Option<String> {
    let start = s.find('"')?;
    let rest = &s[start + 1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn collect_domain_mappings_from_source() -> Vec<(String, String)> {
    let src = include_str!("definitions.rs");
    let mut mappings: Vec<(String, String)> = Vec::new();
    let lines: Vec<&str> = src.lines().collect();

    let mut i = 0usize;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if !trimmed.starts_with("hash_domain!(") {
            i += 1;
            continue;
        }

        let mut buf = String::from(trimmed);
        while !buf.contains(");") && i + 1 < lines.len() {
            i += 1;
            buf.push(' ');
            buf.push_str(lines[i].trim());
        }

        let invocation = buf.strip_prefix("hash_domain!(").unwrap_or("").trim();
        let invocation = invocation.strip_suffix(");").unwrap_or(invocation).trim();

        let Some((type_part, args_part)) = invocation.split_once(',') else {
            i += 1;
            continue;
        };

        let type_name = type_part.trim();
        if type_name.is_empty() {
            i += 1;
            continue;
        }

        let Some(domain) = extract_first_string_literal(args_part) else {
            i += 1;
            continue;
        };

        mappings.push((type_name.to_string(), domain));
        i += 1;
    }

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !trimmed.starts_with("impl ") || !trimmed.contains("DomainSeparation for") {
            continue;
        }

        let Some(after) = trimmed.split_once("DomainSeparation for") else {
            continue;
        };
        let type_name = after
            .1
            .trim_start()
            .split(|ch: char| ch.is_whitespace() || ch == '{')
            .next()
            .unwrap_or("")
            .trim();
        if type_name.is_empty() {
            continue;
        }

        let mut domain: Option<String> = None;
        let mut in_domain_fn = false;
        for &current_line in lines.iter().skip(i) {
            if !in_domain_fn {
                if current_line.contains("fn domain") {
                    in_domain_fn = true;
                }
                continue;
            }

            if let Some(lit) = extract_first_string_literal(current_line) {
                domain = Some(lit);
                break;
            }
        }

        if let Some(domain) = domain {
            mappings.push((type_name.to_string(), domain));
        }
    }

    mappings
}

fn canonical_domain_lines() -> Vec<String> {
    let mut mappings = collect_domain_mappings_from_source();
    mappings.sort_by(|(a, _), (b, _)| a.cmp(b));
    mappings
        .into_iter()
        .map(|(ty, domain)| format!("{ty}={domain}"))
        .collect()
}

#[test]
fn test_domain_strings_are_frozen() {
    let expected = snapshot_lines_from_str(FROZEN_DOMAIN_SNAPSHOT);
    let actual = canonical_domain_lines();
    if std::env::var_os("Z00Z_REGEN_DUMP").is_some() {
        for line in &actual {
            println!("{line}");
        }
    }

    assert_eq!(
        actual, expected,
        "Domain snapshot mismatch. Run `Z00Z_REGEN_DUMP=1 cargo test --release -p z00z_wallets test_domain_strings_are_frozen -- --exact --nocapture` and update `FROZEN_DOMAIN_SNAPSHOT` intentionally."
    );
}

#[test]
fn test_domain_strings_are_unique() {
    let mappings = collect_domain_mappings_from_source();
    assert!(
        mappings.len() >= 10,
        "Expected at least 10 domain mappings; source extraction looks broken"
    );

    let mut seen_types = std::collections::HashSet::new();
    for (ty, _) in &mappings {
        assert!(seen_types.insert(ty), "Duplicate domain type mapping: {ty}");
    }

    let mut seen_domains = std::collections::HashSet::new();
    for (_, domain) in &mappings {
        assert!(
            seen_domains.insert(domain),
            "Duplicate domain found: {domain}"
        );
    }
}

#[test]
fn test_receiver_cache_domains_stable() {
    assert_eq!(
        ReceiverCacheHmacTestDomain::domain(),
        "app/z00z_wallets/address/receiver_cache/test"
    );
    assert_eq!(
        ReceiverCacheHmacProdDomain::domain(),
        "app/z00z_wallets/address/receiver_cache/production"
    );
}

#[test]
fn test_pack_domains_frozen() {
    assert_eq!(
        PackKeyProdDomain::domain(),
        "z00z.wallet.stealth.pack_key.prod"
    );
    assert_eq!(
        PackNonceProdDomain::domain(),
        "z00z.wallet.stealth.pack_nonce.prod"
    );
    assert_eq!(
        WalletLeafAdHashProdDomain::domain(),
        "z00z.wallet.stealth.leaf_ad.prod"
    );
}
