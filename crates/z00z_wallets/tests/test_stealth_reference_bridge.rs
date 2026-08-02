use z00z_crypto::Z00ZScalar;
use z00z_utils::rng::SystemRngProvider;
use z00z_wallets::{
    key::{ReceiverKeys, ReceiverSecret},
    stealth::ecdh::{receiver_derive_dh, sender_derive_dh_with_r},
    stealth::kdf::derive_k_dh_with_req,
    stealth::kdf::{compute_owner_tag, derive_k_dh},
    stealth::{verify_owner_tag, verify_owner_tag_with_req},
};

#[test]
fn test_req_dh_modes_split() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let req_id = [0x71u8; 32];
    let r = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();

    let sender = sender_derive_dh_with_r(&receiver_keys.view_pk, &r).expect("sender");
    let recv = receiver_derive_dh(receiver_keys.reveal_view_sk(), &sender.r_pub).expect("recv");

    let base_sender = derive_k_dh(&sender.dh.to_bytes());
    let base_recv = derive_k_dh(&recv.to_bytes());
    assert_eq!(base_sender, base_recv);

    let sender_dh = sender.dh.to_bytes();
    let recv_dh = recv.to_bytes();
    let req_sender = derive_k_dh_with_req(&sender_dh, &req_id);
    let req_recv = derive_k_dh_with_req(&recv_dh, &req_id);

    assert_eq!(req_sender, req_recv);
    assert_ne!(req_sender, base_sender);
}

#[test]
fn test_req_owner_gate_split() {
    let receiver_secret = ReceiverSecret::generate().expect("receiver secret");
    let receiver_keys = ReceiverKeys::from_receiver_secret(receiver_secret).expect("keys");
    let req_id = [0x72u8; 32];
    let r = Z00ZScalar::random(&mut SystemRngProvider.rng()).unwrap();

    let sender = sender_derive_dh_with_r(&receiver_keys.view_pk, &r).expect("sender");
    let r_pub = sender.r_pub.to_bytes();

    let base_k = derive_k_dh(&sender.dh.to_bytes());
    let req_k = derive_k_dh_with_req(&sender.dh.to_bytes(), &req_id);
    let base_tag = compute_owner_tag(&receiver_keys.owner_handle, &base_k);
    let req_tag = compute_owner_tag(&receiver_keys.owner_handle, &req_k);

    assert!(verify_owner_tag(&receiver_keys, &r_pub, &base_tag).expect("base verify"));
    assert!(!verify_owner_tag(&receiver_keys, &r_pub, &req_tag).expect("plain mismatch"));
    assert!(
        verify_owner_tag_with_req(&receiver_keys, &r_pub, &req_tag, Some(&req_id))
            .expect("req verify")
    );
}
