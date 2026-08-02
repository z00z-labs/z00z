use super::{
    build_stealth_leaf_with_rng, core_build_output_bundle, core_build_output_bundle_with_rng,
    derive_dh_key, derive_s_out, hash_zk, sender_derive_dh_with_r, AssetClass, AssetWire,
    DeterministicRngProvider, Hidden, OutputBundle, ReceiverCard, Stage4OutSeedDomain,
    SystemRngProvider, Z00ZScalar,
};

pub(super) fn build_bob_outs(
    asset_class: AssetClass,
    recipient_name: &str,
    bob_card: &ReceiverCard,
    bob_amounts: &[u64],
    bob_serials: &[u32],
    split_seed: Option<u64>,
) -> Result<Vec<(OutputBundle, usize)>, String> {
    bob_amounts
        .iter()
        .zip(bob_serials.iter())
        .enumerate()
        .map(|(idx, (amount, serial_id))| {
            mk_out(
                recipient_name.to_string(),
                z00z_wallets::tx::TxOutRole::Recipient,
                asset_class,
                bob_card,
                *amount,
                *serial_id,
                split_seed,
                idx,
            )
            .map(|out| (out, idx))
        })
        .collect()
}

pub(super) fn build_change_out(
    asset_class: AssetClass,
    sender_name: &str,
    alice_card: &ReceiverCard,
    change_value: u64,
    serial_id: u32,
    split_seed: Option<u64>,
    idx: usize,
) -> Result<(OutputBundle, usize), String> {
    mk_out(
        sender_name.to_string(),
        z00z_wallets::tx::TxOutRole::Change,
        asset_class,
        alice_card,
        change_value,
        serial_id,
        split_seed,
        idx,
    )
    .map(|out| (out, idx))
}

pub(super) fn role_rank(role: z00z_wallets::tx::TxOutRole) -> u8 {
    match role {
        z00z_wallets::tx::TxOutRole::Recipient => 0,
        z00z_wallets::tx::TxOutRole::Change => 1,
        z00z_wallets::tx::TxOutRole::Fee => 2,
    }
}

pub(super) fn build_fee_out(
    fee_name: &str,
    fee_card: &ReceiverCard,
    fee_value: u64,
    serial_id: u32,
    split_seed: Option<u64>,
    idx: usize,
) -> Result<(OutputBundle, usize), String> {
    mk_out(
        fee_name.to_string(),
        z00z_wallets::tx::TxOutRole::Fee,
        AssetClass::Coin,
        fee_card,
        fee_value,
        serial_id,
        split_seed,
        idx,
    )
    .map(|out| (out, idx))
}

pub(super) fn card_for_role<'a>(
    role: z00z_wallets::tx::TxOutRole,
    alice_card: &'a ReceiverCard,
    bob_card: &'a ReceiverCard,
    fee_card: &'a ReceiverCard,
) -> &'a ReceiverCard {
    match role {
        z00z_wallets::tx::TxOutRole::Recipient => bob_card,
        z00z_wallets::tx::TxOutRole::Change => alice_card,
        z00z_wallets::tx::TxOutRole::Fee => fee_card,
    }
}

fn seed32(seed: u64, party: &str, value: u64, serial_id: u32, idx: usize) -> [u8; 32] {
    let seed_bytes = seed.to_le_bytes();
    let value_bytes = value.to_le_bytes();
    let serial_bytes = serial_id.to_le_bytes();
    let idx_bytes = (idx as u64).to_le_bytes();
    hash_zk::<Stage4OutSeedDomain>(
        "",
        &[
            &seed_bytes,
            party.as_bytes(),
            &value_bytes,
            &serial_bytes,
            &idx_bytes,
        ],
    )
}

fn mk_out(
    party: String,
    role: z00z_wallets::tx::TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    split_seed: Option<u64>,
    idx: usize,
) -> Result<OutputBundle, String> {
    let Some(seed) = split_seed else {
        return core_build_output_bundle(party, role, class, card, value, serial_id);
    };

    let seed = seed32(seed, &party, value, serial_id, idx);
    let mut rng = DeterministicRngProvider::from_seed(seed).rng();
    core_build_output_bundle_with_rng(party, role, class, card, value, serial_id, &mut rng)
}

pub(super) fn mk_out_with_blind(
    party: String,
    role: z00z_wallets::tx::TxOutRole,
    class: AssetClass,
    card: &ReceiverCard,
    value: u64,
    serial_id: u32,
    split_seed: Option<u64>,
    idx: usize,
    blinding: Z00ZScalar,
) -> Result<OutputBundle, String> {
    let view_pk = z00z_crypto::Z00ZRistrettoPoint::try_from_bytes(card.view_pk)
        .map_err(|e| format!("invalid receiver card view_pk: {e}"))?;
    let hidden = Hidden::hide(blinding);

    if let Some(seed) = split_seed {
        let seed = seed32(seed, &party, value, serial_id, idx);
        let mut rng = DeterministicRngProvider::from_seed(seed).rng();
        let r = Z00ZScalar::random(&mut rng).map_err(|error| error.to_string())?;
        let sender = sender_derive_dh_with_r(&view_pk, &r).map_err(|e| e.to_string())?;
        let r_pub = sender.r_pub.to_bytes();
        let k_dh = derive_dh_key(&sender.dh);
        let s_out = derive_s_out(&k_dh, &r_pub, serial_id);
        let leaf = build_stealth_leaf_with_rng(
            &k_dh,
            &r_pub,
            &card.owner_handle,
            value,
            serial_id,
            s_out,
            &hidden,
            &mut rng,
        )
        .map_err(|e| e.to_string())?;

        return Ok(OutputBundle {
            receiver: party,
            role,
            class,
            value,
            leaf,
            k_dh,
            s_out,
        });
    }

    let mut rng = SystemRngProvider.rng();
    let r = Z00ZScalar::random(&mut rng).map_err(|error| error.to_string())?;
    let sender = sender_derive_dh_with_r(&view_pk, &r).map_err(|e| e.to_string())?;
    let r_pub = sender.r_pub.to_bytes();
    let k_dh = derive_dh_key(&sender.dh);
    let s_out = derive_s_out(&k_dh, &r_pub, serial_id);
    let leaf = build_stealth_leaf_with_rng(
        &k_dh,
        &r_pub,
        &card.owner_handle,
        value,
        serial_id,
        s_out,
        &hidden,
        &mut rng,
    )
    .map_err(|e| e.to_string())?;

    Ok(OutputBundle {
        receiver: party,
        role,
        class,
        value,
        leaf,
        k_dh,
        s_out,
    })
}

pub(super) fn sum_input_blindings(
    recv_sec: [u8; 32],
    selected_inputs: &[AssetWire],
) -> Result<Z00ZScalar, String> {
    let blindings = selected_inputs
        .iter()
        .map(|item| {
            let pack = z00z_wallets::tx::resolve_input_pack(recv_sec, item)?;
            Z00ZScalar::try_from_bytes(pack.blinding)
                .map_err(|e| format!("stage4: input blinding scalar decode failed: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut items = blindings.into_iter();
    let Some(first) = items.next() else {
        return Err("stage4: input blinding decode requires at least one input".to_string());
    };

    Ok(items.fold(first, |acc, item| &acc + &item))
}
