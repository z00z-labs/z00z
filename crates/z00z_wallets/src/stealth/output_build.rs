struct SenderPath {
    r_pub: [u8; 32],
    owner_tag: [u8; 32],
    k_dh: [u8; 32],
    tag_mode: TagMode,
}

enum AssetIdMode {
    Explicit([u8; 32]),
    HashFromSOut,
}

struct OutputBuildState {
    output: TxStealthOutput,
    ctx: SenderValidationCtx,
    asset_id: [u8; 32],
    blinding: Z00ZScalar,
}

// Canonical wallet-owned raw sender seam for the lightweight serial lane.
// Stateful callers reach this helper through select_r(...), while higher-level
// compatibility adapters stay on separate stateless helpers below.
fn build_output_ctx(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
) -> Result<(TxStealthOutput, SenderValidationCtx), StealthError> {
    build_output_ctx_with_serial(
        receiver_card,
        payment_request,
        sender_wallet,
        tx_digest,
        out_index,
        amount,
        asset_id,
        LIGHT_SERIAL_ID,
    )
}

// Canonical wallet-owned raw sender seam for explicit serial-aware callers such
// as Stage 3. This keeps helper/formula ownership in one place instead of
// letting runtime adapters rebuild the output path themselves.
fn build_output_ctx_with_serial(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    sender_wallet: &mut SenderWallet,
    tx_digest: &[u8; 32],
    out_index: u32,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<(TxStealthOutput, SenderValidationCtx), StealthError> {
    let (r, _) = select_r(
        sender_wallet,
        &receiver_card.owner_handle,
        tx_digest,
        out_index,
    )?;

    build_output_ctx_with_r(
        receiver_card,
        payment_request,
        &r,
        amount,
        asset_id,
        serial_id,
    )
}

// Stateless helper entrypoint for callers that already hold the scalar they
// want to use. Compatibility adapters can route through this seam, but they
// must not be described as the wallet-owned hedged-r sender policy.
fn build_output_ctx_with_r(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    r: &Z00ZScalar,
    amount: u64,
    asset_id: &[u8; 32],
    serial_id: u32,
) -> Result<(TxStealthOutput, SenderValidationCtx), StealthError> {
    let state = build_output_state_with_r(
        receiver_card,
        payment_request,
        amount,
        AssetIdMode::Explicit(*asset_id),
        serial_id,
        r,
    )?;

    Ok((state.output, state.ctx))
}

fn approve_card<'a>(
    receiver_card: &ReceiverCard,
    build_check: BuildCheck<'a>,
) -> Result<(), StealthError> {
    let BuildCheck { pins, chain_id: _ } = build_check;

    receiver_card
        .validate_structure()
        .map_err(|_| StealthError::InvalidStealthInput)?;
    receiver_card
        .validate_ecc_points()
        .map_err(|_| StealthError::InvalidStealthInput)?;
    receiver_card
        .validate_signature()
        .map_err(|_| StealthError::InvalidStealthInput)?;

    let pin = pins
        .get(&receiver_card.owner_handle)
        .ok_or(StealthError::InvalidStealthInput)?;

    if !matches!(pin.trust_level, TrustLevel::Pinned)
        || pin.view_pk == [0u8; 32]
        || pin.view_pk != receiver_card.view_pk
        || pin.identity_pk != receiver_card.identity_pk
    {
        return Err(StealthError::InvalidStealthInput);
    }

    Ok(())
}

fn approve_req<'a>(
    payment_request: Option<&PaymentRequest>,
    build_check: BuildCheck<'a>,
) -> Result<(), StealthError> {
    if let Some(request) = payment_request {
        let mut shadow_pins = build_check.pins.clone();
        let outcome = request
            .validate_all(&mut shadow_pins, build_check.chain_id)
            .map_err(|_| StealthError::InvalidStealthInput)?;
        if !matches!(outcome, ValidationOutcome::Approved) {
            return Err(StealthError::InvalidStealthInput);
        }
    }

    Ok(())
}

fn build_output_state_with_r(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    asset_id_mode: AssetIdMode,
    serial_id: u32,
    r: &Z00ZScalar,
) -> Result<OutputBuildState, StealthError> {
    let mut rng = SystemRngProvider.rng();
    build_output_state_with_rng(
        receiver_card,
        payment_request,
        amount,
        asset_id_mode,
        serial_id,
        r,
        &mut rng,
    )
}

fn build_output_state_with_blinding(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    asset_id_mode: AssetIdMode,
    serial_id: u32,
    r: &Z00ZScalar,
    blinding: &Z00ZScalar,
) -> Result<OutputBuildState, StealthError> {
    validate_request_bind(receiver_card, payment_request, amount)?;

    let sender_path = derive_sender_path(receiver_card, payment_request, r)?;
    let s_out = derive_s_out(&sender_path.k_dh, &sender_path.r_pub, serial_id);
    let asset_id = match asset_id_mode {
        AssetIdMode::Explicit(asset_id) => asset_id,
        AssetIdMode::HashFromSOut => {
            z00z_crypto::hash_zk::hash_zk::<z00z_crypto::domains::AssetIdDomain>("", &[&s_out])
        }
    };
    let commitment =
        create_commitment(amount, blinding).map_err(|_| StealthError::InvalidStealthInput)?;
    let mut c_amount = [0u8; 32];
    c_amount.copy_from_slice(commitment.as_bytes());
    let leaf_ad = compute_leaf_ad(
        &asset_id,
        serial_id,
        &sender_path.r_pub,
        &sender_path.owner_tag,
        &c_amount,
    );
    let plaintext = AssetPackPlain {
        value: amount,
        blinding: blinding.to_bytes(),
        s_out,
    }
    .to_bytes();
    let enc_pack = ZkPack::encrypt(
        &sender_path.k_dh,
        &leaf_ad,
        &sender_path.r_pub,
        &asset_id,
        serial_id,
        &plaintext,
    );
    let tag16 = Some(match sender_path.tag_mode {
        TagMode::CardBound => compute_tag16(&sender_path.k_dh, &leaf_ad),
        TagMode::RequestBound { req_id } => compute_tag16_with_req(&sender_path.k_dh, &req_id),
    });
    let output = TxStealthOutput {
        r_pub: sender_path.r_pub,
        owner_tag: sender_path.owner_tag,
        tag16,
        enc_pack,
        c_amount,
    };
    let ctx = SenderValidationCtx {
        k_dh: sender_path.k_dh,
        owner_handle: receiver_card.owner_handle,
        asset_id,
        serial_id,
        tag_mode: sender_path.tag_mode,
    };

    Ok(OutputBuildState {
        output,
        ctx,
        asset_id,
        blinding: blinding.dangerous_clone(),
    })
}

fn build_leaf_state(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    serial_id: u32,
) -> Result<OutputBuildState, StealthError> {
    let mut rng = SystemRngProvider.rng();
    build_leaf_state_rng(receiver_card, payment_request, amount, serial_id, &mut rng)
}

fn build_leaf_state_rng<R: rand::CryptoRng + rand::RngCore>(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    serial_id: u32,
    rng: &mut R,
) -> Result<OutputBuildState, StealthError> {
    let r = select_r_rng(rng)?;
    build_output_state_with_rng(
        receiver_card,
        payment_request,
        amount,
        AssetIdMode::HashFromSOut,
        serial_id,
        &r,
        rng,
    )
}

fn build_output_state_with_rng<R: rand::CryptoRng + rand::RngCore>(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
    asset_id_mode: AssetIdMode,
    serial_id: u32,
    r: &Z00ZScalar,
    rng: &mut R,
) -> Result<OutputBuildState, StealthError> {
    validate_request_bind(receiver_card, payment_request, amount)?;

    let sender_path = derive_sender_path(receiver_card, payment_request, r)?;
    let s_out = derive_s_out(&sender_path.k_dh, &sender_path.r_pub, serial_id);
    let asset_id = match asset_id_mode {
        AssetIdMode::Explicit(asset_id) => asset_id,
        AssetIdMode::HashFromSOut => {
            z00z_crypto::hash_zk::hash_zk::<z00z_crypto::domains::AssetIdDomain>("", &[&s_out])
        }
    };
    let (blinding, c_amount) = make_amount_with_rng(amount, rng)?;
    let leaf_ad = compute_leaf_ad(
        &asset_id,
        serial_id,
        &sender_path.r_pub,
        &sender_path.owner_tag,
        &c_amount,
    );
    let plaintext = AssetPackPlain {
        value: amount,
        blinding: blinding.to_bytes(),
        s_out,
    }
    .to_bytes();
    let enc_pack = ZkPack::encrypt(
        &sender_path.k_dh,
        &leaf_ad,
        &sender_path.r_pub,
        &asset_id,
        serial_id,
        &plaintext,
    );
    let tag16 = Some(match sender_path.tag_mode {
        TagMode::CardBound => compute_tag16(&sender_path.k_dh, &leaf_ad),
        TagMode::RequestBound { req_id } => compute_tag16_with_req(&sender_path.k_dh, &req_id),
    });
    let output = TxStealthOutput {
        r_pub: sender_path.r_pub,
        owner_tag: sender_path.owner_tag,
        tag16,
        enc_pack,
        c_amount,
    };
    let ctx = SenderValidationCtx {
        k_dh: sender_path.k_dh,
        owner_handle: receiver_card.owner_handle,
        asset_id,
        serial_id,
        tag_mode: sender_path.tag_mode,
    };

    Ok(OutputBuildState {
        output,
        ctx,
        asset_id,
        blinding,
    })
}

fn select_r_rng<R: rand::CryptoRng + rand::RngCore>(
    rng: &mut R,
) -> Result<Z00ZScalar, StealthError> {
    Z00ZScalar::random(rng).map_err(|_| StealthError::InvalidEphemeralScalar)
}

fn derive_sender_path(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    r: &Z00ZScalar,
) -> Result<SenderPath, StealthError> {
    let view_pk = decode_public_key(&receiver_card.view_pk)?;
    let dh = compute_dh_sender(r, &view_pk)?;
    let k_dh = derive_stealth_key(payment_request, &dh);
    let owner_tag = compute_owner_tag(&receiver_card.owner_handle, &k_dh);
    let r_pub = encode_r_pub(&compute_r_pub(r)?);

    Ok(SenderPath {
        r_pub,
        owner_tag,
        k_dh,
        tag_mode: make_tag_mode(payment_request),
    })
}

fn make_amount_with_rng<R: rand::CryptoRng + rand::RngCore>(
    amount: u64,
    rng: &mut R,
) -> Result<(Z00ZScalar, [u8; 32]), StealthError> {
    let blinding = Z00ZScalar::random(rng).map_err(|_| StealthError::InvalidEphemeralScalar)?;
    let commitment =
        create_commitment(amount, &blinding).map_err(|_| StealthError::InvalidStealthInput)?;
    let mut c_amount = [0u8; 32];
    c_amount.copy_from_slice(commitment.as_bytes());
    Ok((blinding, c_amount))
}

fn make_tag_mode(payment_request: Option<&PaymentRequest>) -> TagMode {
    match payment_request {
        Some(request) => TagMode::RequestBound {
            req_id: request.req_id,
        },
        None => TagMode::CardBound,
    }
}

fn validate_request_bind(
    receiver_card: &ReceiverCard,
    payment_request: Option<&PaymentRequest>,
    amount: u64,
) -> Result<(), StealthError> {
    if let Some(request) = payment_request {
        // Accepted-flow validation requires the request and receiver card to
        // describe the same route before request-bound behavior is allowed.
        if request.verify().is_err() {
            return Err(StealthError::InvalidStealthInput);
        }

        let same_route = request.owner_handle == receiver_card.owner_handle
            && request.view_pk == receiver_card.view_pk
            && request.identity_pk == receiver_card.identity_pk;
        if !same_route || amount_mismatch(request, amount) {
            return Err(StealthError::InvalidStealthInput);
        }

        if matches!(request.check_validity(), ValidityStatus::Expired) {
            return Err(StealthError::InvalidStealthInput);
        }
    }

    Ok(())
}

fn amount_mismatch(request: &PaymentRequest, amount: u64) -> bool {
    request
        .amount
        .is_some_and(|fixed_amount| fixed_amount != amount)
}

fn derive_stealth_key(payment_request: Option<&PaymentRequest>, dh: &[u8; 32]) -> [u8; 32] {
    if let Some(request) = payment_request {
        return derive_k_dh_with_req(dh, &request.req_id);
    }

    derive_k_dh(dh)
}

fn select_r(
    sender_wallet: &mut SenderWallet,
    owner_handle: &[u8; 32],
    tx_digest: &[u8; 32],
    out_index: u32,
) -> Result<(Z00ZScalar, [u8; 32]), StealthError> {
    select_r_seeded(
        sender_wallet,
        owner_handle,
        tx_digest,
        out_index,
        get_rng_bytes(),
    )
}

fn select_r_seeded(
    sender_wallet: &mut SenderWallet,
    owner_handle: &[u8; 32],
    tx_digest: &[u8; 32],
    out_index: u32,
    rng_bytes: [u8; 32],
) -> Result<(Z00ZScalar, [u8; 32]), StealthError> {
    let secret_salt = sender_wallet.secret_salt;
    select_r_with(sender_wallet, owner_handle, |retry_index| {
        if retry_index == 0 {
            return derive_r_hedged(&rng_bytes, &secret_salt, tx_digest, out_index);
        }

        generate_r_retry(&rng_bytes, &secret_salt, tx_digest, out_index, retry_index)
    })
}

fn select_r_with<F>(
    sender_wallet: &mut SenderWallet,
    owner_handle: &[u8; 32],
    mut derive_r: F,
) -> Result<(Z00ZScalar, [u8; 32]), StealthError>
where
    F: FnMut(u32) -> Result<Z00ZScalar, StealthError>,
{
    for retry_index in 0..=MAX_R_RETRY {
        let r = derive_r(retry_index)?;
        let r_pub = compute_r_pub(&r)?;
        let r_pub_bytes = encode_r_pub(&r_pub);

        match sender_wallet
            .recent_r
            .check_and_insert(owner_handle, &r_pub_bytes)
        {
            Ok(()) => return Ok((r, r_pub_bytes)),
            Err(WalletError::DuplicateEphemeralR) if retry_index < MAX_R_RETRY => continue,
            Err(WalletError::DuplicateEphemeralR) => return Err(StealthError::RetryLimitReached),
            Err(_) => return Err(StealthError::InvalidStealthInput),
        }
    }

    Err(StealthError::RetryLimitReached)
}
