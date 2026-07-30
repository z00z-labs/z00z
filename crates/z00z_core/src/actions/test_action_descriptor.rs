use std::collections::BTreeSet;

use super::LifecycleEffectV1;

#[test]
fn lifecycle_effect_atomic_basis_is_complete_and_canonical() {
    let expected = [
        (LifecycleEffectV1::Create, "create"),
        (LifecycleEffectV1::Issue, "issue"),
        (LifecycleEffectV1::Offer, "offer"),
        (LifecycleEffectV1::Accept, "accept"),
        (LifecycleEffectV1::Reject, "reject"),
        (LifecycleEffectV1::Transfer, "transfer"),
        (LifecycleEffectV1::Split, "split"),
        (LifecycleEffectV1::Merge, "merge"),
        (LifecycleEffectV1::Lock, "lock"),
        (LifecycleEffectV1::Claim, "claim"),
        (LifecycleEffectV1::Release, "release"),
        (LifecycleEffectV1::Cancel, "cancel"),
        (LifecycleEffectV1::Redeem, "redeem"),
        (LifecycleEffectV1::PartialRedeem, "partial_redeem"),
        (LifecycleEffectV1::Refund, "refund"),
        (LifecycleEffectV1::Burn, "burn"),
        (LifecycleEffectV1::Expire, "expire"),
        (LifecycleEffectV1::Grant, "grant"),
        (LifecycleEffectV1::Delegate, "delegate"),
        (LifecycleEffectV1::Use, "use"),
        (LifecycleEffectV1::Revoke, "revoke"),
        (LifecycleEffectV1::Challenge, "challenge"),
        (LifecycleEffectV1::Resolve, "resolve"),
        (LifecycleEffectV1::Disclose, "disclose"),
    ];

    assert_eq!(
        LifecycleEffectV1::ATOMIC_BASIS,
        expected.map(|(effect, _)| effect)
    );
    assert_eq!(
        LifecycleEffectV1::ATOMIC_BASIS
            .into_iter()
            .collect::<BTreeSet<_>>()
            .len(),
        LifecycleEffectV1::ATOMIC_BASIS.len(),
        "atomic lifecycle effects must remain unique"
    );

    for (effect, wire_name) in expected {
        assert_eq!(effect.as_str(), wire_name);
        assert_eq!(
            serde_yaml::to_value(effect).expect("serialize lifecycle effect"),
            serde_yaml::Value::String(wire_name.to_string())
        );
        assert_eq!(
            serde_yaml::from_value::<LifecycleEffectV1>(serde_yaml::Value::String(
                wire_name.to_string()
            ))
            .expect("deserialize lifecycle effect"),
            effect
        );
    }
}

#[test]
fn no_state_change_is_not_an_atomic_lifecycle_mutation() {
    assert!(!LifecycleEffectV1::ATOMIC_BASIS.contains(&LifecycleEffectV1::NoStateChange));
    assert_eq!(
        serde_yaml::to_value(LifecycleEffectV1::NoStateChange)
            .expect("serialize compatibility effect"),
        serde_yaml::Value::String("no_state_change".to_string())
    );
}
