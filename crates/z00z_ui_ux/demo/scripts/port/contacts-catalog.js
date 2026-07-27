"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z contracts must load before Contacts fixtures.");
  }

  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };

  const CONTACT_STATUS_IDS = deepFreeze([
    "known_locally",
    "needs_confirmation",
    "identity_changed",
    "expired",
    "revoked"
  ]);

  const CONTACT_FIXTURES = deepFreeze([
    {
      id: "contact_ada",
      label: "Ada",
      initials: "AD",
      safeNote: "Local design collaborator",
      tags: ["work", "reviewed"],
      fingerprint: "7C91…8B2E",
      source: "Reviewed receiver card",
      lastLocalUseAt: "2026-07-26T09:20:00Z",
      chainId: "mainnet",
      compatibility: "Selected wallet compatible",
      expiresAt: "2026-10-26T00:00:00Z",
      status: "known_locally",
      pinned: true,
      contactIdentityKey: "contact_key_7c91…8b2e",
      reticulumDestinationRef: "reticulum_ref_4a31…98c0",
      walletRecipientRef: "wallet_receiver_ref_9d20…11f7"
    },
    {
      id: "contact_ben",
      label: "Ben",
      initials: "BE",
      safeNote: "Travel payment contact",
      tags: ["personal"],
      fingerprint: "48A0…19D4",
      source: "Payment request",
      lastLocalUseAt: "2026-07-24T14:10:00Z",
      chainId: "mainnet",
      compatibility: "Selected wallet compatible",
      expiresAt: "2026-08-24T00:00:00Z",
      status: "needs_confirmation",
      pinned: false,
      contactIdentityKey: "contact_key_48a0…19d4",
      reticulumDestinationRef: "reticulum_ref_811c…7e22",
      walletRecipientRef: "wallet_receiver_ref_301a…cf42"
    },
    {
      id: "contact_community",
      label: "Community desk",
      initials: "CD",
      safeNote: "Voucher coordination",
      tags: ["community", "voucher"],
      fingerprint: "B132…0F65",
      source: "Native share import",
      lastLocalUseAt: "2026-07-23T08:40:00Z",
      chainId: "mainnet",
      compatibility: "Selected wallet compatible",
      expiresAt: "2026-09-01T00:00:00Z",
      status: "known_locally",
      pinned: false,
      contactIdentityKey: "contact_key_b132…0f65",
      reticulumDestinationRef: "reticulum_ref_2c74…5f09",
      walletRecipientRef: "wallet_receiver_ref_a043…e210"
    },
    {
      id: "contact_ops",
      label: "Operations",
      initials: "OP",
      safeNote: "Confirm the changed identity before use",
      tags: ["work", "attention"],
      fingerprint: "OLD 913F…4D20 → NEW 5AE1…883C",
      source: "Receiver-card update",
      lastLocalUseAt: "2026-07-22T18:00:00Z",
      chainId: "mainnet",
      compatibility: "Revalidation required",
      expiresAt: "2026-08-22T00:00:00Z",
      status: "identity_changed",
      pinned: false,
      contactIdentityKey: "contact_key_5ae1…883c",
      reticulumDestinationRef: "reticulum_ref_d118…5a40",
      walletRecipientRef: "wallet_receiver_ref_c8b4…012f"
    },
    {
      id: "contact_old_service",
      label: "Old service",
      initials: "OS",
      safeNote: "Expired receiver material",
      tags: ["service"],
      fingerprint: "031A…77E8",
      source: "Manual reviewed material",
      lastLocalUseAt: "2026-07-10T12:00:00Z",
      chainId: "testnet-1",
      compatibility: "Different profile",
      expiresAt: "2026-07-20T00:00:00Z",
      status: "expired",
      pinned: false,
      contactIdentityKey: "contact_key_031a…77e8",
      reticulumDestinationRef: "reticulum_ref_5d07…0c31",
      walletRecipientRef: "wallet_receiver_ref_71e2…42d0"
    },
    {
      id: "contact_revoked",
      label: "Revoked card",
      initials: "RC",
      safeNote: "Retained only for local history context",
      tags: ["blocked"],
      fingerprint: "A901…22BC",
      source: "Revocation advisory",
      lastLocalUseAt: "2026-07-08T07:15:00Z",
      chainId: "mainnet",
      compatibility: "Do not use",
      expiresAt: "2026-08-08T00:00:00Z",
      status: "revoked",
      pinned: false,
      contactIdentityKey: "contact_key_a901…22bc",
      reticulumDestinationRef: "reticulum_ref_022a…f180",
      walletRecipientRef: "wallet_receiver_ref_e1d3…0990"
    }
  ]);

  const CONTACT_IMPORT_PREVIEWS = deepFreeze([
    {
      id: "receiver_card",
      label: "Receiver card",
      iconName: "receive",
      summary: "Review bundled public receiver material before saving a local label."
    },
    {
      id: "payment_request",
      label: "Payment request",
      iconName: "send",
      summary: "Extract only supported public receiver material from a reviewed request."
    },
    {
      id: "qr_scan",
      label: "QR scan",
      iconName: "search",
      summary: "Native camera boundary required; no browser camera access in this demo."
    },
    {
      id: "native_share",
      label: "Native share/import",
      iconName: "receive",
      summary: "Tauri/native share boundary required; the demo loads no external file or URL."
    },
    {
      id: "manual",
      label: "Manual public material",
      iconName: "plus",
      summary: "Typed reviewed fields only; no secret or arbitrary address parser."
    }
  ]);

  function createInitialContacts() {
    return structuredClone(CONTACT_FIXTURES);
  }

  function assertContactFixtures() {
    const ids = CONTACT_FIXTURES.map(({ id }) => id);
    if (new Set(ids).size !== ids.length) throw new Error("Contact IDs must be unique.");
    const statuses = new Set(CONTACT_FIXTURES.map(({ status }) => status));
    for (const status of CONTACT_STATUS_IDS) {
      if (!statuses.has(status)) throw new Error(`Missing contact status fixture: ${status}`);
    }
    for (const entry of CONTACT_FIXTURES) {
      if (new Set([
        entry.contactIdentityKey,
        entry.reticulumDestinationRef,
        entry.walletRecipientRef
      ]).size !== 3) {
        throw new Error(`Contact identity domains must remain separate: ${entry.id}`);
      }
    }
    const serialized = JSON.stringify(CONTACT_FIXTURES);
    if (/((?:https?:)?\/\/)|\b(?:public_presence|social_graph|trust_score|phone|email|raw_address|private_key|seed_phrase)\b/i.test(serialized)) {
      throw new Error("Contact fixtures contain forbidden presence, graph, remote, or secret fields.");
    }
    return true;
  }

  assertContactFixtures();

  Object.assign(root.Z00ZDemo, {
    CONTACT_STATUS_IDS,
    CONTACT_FIXTURES,
    CONTACT_IMPORT_PREVIEWS,
    createInitialContacts,
    assertContactFixtures
  });
})(typeof window === "undefined" ? globalThis : window);
