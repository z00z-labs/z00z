"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z demo contracts must load before the dApp catalogue.");
  }

  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };

  const DAPP_OBJECT_FAMILIES = deepFreeze([
    "asset",
    "voucher",
    "permission",
    "claim",
    "service_right"
  ]);

  const DAPP_INTENT_TYPES = deepFreeze([
    "prepare_offline_payment",
    "issue_private_voucher",
    "prepare_external_asset_lock",
    "authorize_scoped_expense",
    "redeem_service_credit",
    "propose_agent_budget"
  ]);

  const descriptor = ({
    id,
    label,
    summary,
    useCaseFamily,
    maturity = "target",
    iconName,
    intentType,
    requestedObjectFamilies,
    offlineMode,
    offlineSummary,
    disclosures,
    catalogueState = "discoverable",
    valuePath = "none",
    feePath = "none",
    reviewBoundary
  }) => deepFreeze({
    id,
    label,
    summary,
    useCaseFamily,
    maturity,
    availability: "unavailable",
    evidenceSource: "fixture",
    freshness: "not_applicable",
    presentationMode: "roadmap_preview",
    iconName,
    intentType,
    requestedObjectFamilies,
    publisher: {
      label: "Z00Z curated demo",
      provenance: "bundled_local_descriptor",
      verified: false
    },
    offlineBehavior: {
      mode: offlineMode,
      summary: offlineSummary
    },
    disclosures,
    catalogueState,
    valuePath,
    feePath,
    reviewBoundary,
    helpTopicId: "dapps.discover",
    executionBoundary: "typed_intent_only",
    remoteCodeAllowed: false,
    walletBridgeAllowed: false
  });

  const DAPP_CATALOG = deepFreeze([
    descriptor({
      id: "offline-pay",
      label: "Offline Pay",
      summary: "Prepare a bounded private cash handoff for later reconciliation.",
      useCaseFamily: "private_payment",
      iconName: "send",
      intentType: "prepare_offline_payment",
      requestedObjectFamilies: ["asset"],
      offlineMode: "prepared_package",
      offlineSummary: "The handoff may be prepared offline; reconciliation remains explicit after connectivity returns.",
      disclosures: ["asset_family", "bounded_value", "expiry", "recipient_commitment"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Preparing a handoff never proves later acceptance or settlement."
    }),
    descriptor({
      id: "private-voucher",
      label: "Private Voucher",
      summary: "Coordinate bounded aid or community distribution with explicit claims.",
      useCaseFamily: "bounded_distribution",
      iconName: "voucher",
      intentType: "issue_private_voucher",
      requestedObjectFamilies: ["voucher", "claim"],
      offlineMode: "local_handoff",
      offlineSummary: "A voucher handoff may be carried locally; redemption and issuer policy are reviewed separately.",
      disclosures: ["voucher_class", "claim_constraints", "uses", "expiry"],
      catalogueState: "approved",
      reviewBoundary: "The descriptor cannot issue, redeem, or attest ownership of a Voucher."
    }),
    descriptor({
      id: "external-asset-locker",
      label: "External Asset Locker",
      summary: "Represent a bounded private right over explicitly external custody.",
      useCaseFamily: "external_custody_right",
      iconName: "lock",
      intentType: "prepare_external_asset_lock",
      requestedObjectFamilies: ["asset", "permission"],
      offlineMode: "reconnect_required",
      offlineSummary: "The local proposal is inspectable offline; external custody status is never inferred without its authority.",
      disclosures: ["asset_family", "custody_class", "permission_scope", "expiry"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Z00Z does not become custodian and the demo cannot verify an external locker."
    }),
    descriptor({
      id: "scoped-expenses",
      label: "Scoped Expenses",
      summary: "Review organizational spend under bounded permission and value limits.",
      useCaseFamily: "organizational_spend",
      iconName: "permission",
      intentType: "authorize_scoped_expense",
      requestedObjectFamilies: ["asset", "permission"],
      offlineMode: "policy_snapshot",
      offlineSummary: "A local policy snapshot can be inspected offline; authorization is re-checked before wallet review.",
      disclosures: ["permission_scope", "bounded_value", "uses", "expiry"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Approval is limited to the displayed scope and never implies generic signing."
    }),
    descriptor({
      id: "service-credits",
      label: "Service Credits",
      summary: "Model bounded API, data, compute, or access rights as explicit service credits.",
      useCaseFamily: "service_access",
      iconName: "coin",
      intentType: "redeem_service_credit",
      requestedObjectFamilies: ["service_right", "permission"],
      offlineMode: "proof_carry",
      offlineSummary: "A bounded right can be carried for later presentation; service delivery remains externally authoritative.",
      disclosures: ["service_class", "permission_scope", "uses", "expiry"],
      reviewBoundary: "A service credit is not a universal entitlement or proof that a service was delivered."
    }),
    descriptor({
      id: "agent-budget",
      label: "Agent Budget",
      summary: "Concept-only composed Permission with separate value and fee review.",
      useCaseFamily: "bounded_agent_action",
      maturity: "concept",
      iconName: "spark",
      intentType: "propose_agent_budget",
      requestedObjectFamilies: ["asset", "permission"],
      offlineMode: "proposal_only",
      offlineSummary: "Only a local proposal is retained; every value path requires a fresh wallet review.",
      disclosures: ["permission_scope", "bounded_value", "uses", "expiry", "delegation"],
      valuePath: "separate_wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "No autonomous execution, generic signing, hidden value, or hidden fee is permitted."
    })
  ]);

  const DAPP_CONNECTION_FIXTURES = deepFreeze([
    {
      id: "connection_offline_pay",
      descriptorId: "offline-pay",
      status: "pending",
      requestedAt: "2026-07-26T09:15:00Z",
      humanIntent: "Prepare one offline payment package for later recipient handoff.",
      action: "Prepare payment handoff",
      objectFamily: "asset",
      exactScope: "Native Z00Z asset · one prepared handoff",
      uses: "1 use",
      expiry: "2026-07-27T09:15:00Z",
      delegation: "Not allowed",
      value: { amount: "24.00", unit: "Z00Z" },
      fee: { amount: "0.001", unit: "Z00Z", path: "Separate wallet review" },
      disclosures: ["asset family", "bounded value", "expiry", "recipient commitment"],
      revokeBehavior: "Cancel the unused local approval; an exported package remains independently inspectable.",
      reauth: "Wallet password required before the value path can enter Wallet review."
    },
    {
      id: "connection_scoped_expenses",
      descriptorId: "scoped-expenses",
      status: "active",
      requestedAt: "2026-07-25T13:40:00Z",
      humanIntent: "Propose expenses inside the displayed weekly operations scope.",
      action: "Propose scoped expense",
      objectFamily: "permission",
      exactScope: "Operations · native asset · maximum 75 Z00Z per request",
      uses: "8 of 12 remaining",
      expiry: "2026-08-02T00:00:00Z",
      delegation: "Attenuation only; cannot expand value, assets, recipients, or expiry",
      value: { amount: "Per request · up to 75.00", unit: "Z00Z" },
      fee: { amount: "Shown per request", unit: "Z00Z", path: "Separate wallet review" },
      disclosures: ["permission scope", "bounded value", "uses", "expiry"],
      revokeBehavior: "Future proposals are rejected; already reviewed wallet operations keep their own outcome.",
      reauth: "Required for each value-bearing Wallet review."
    },
    {
      id: "connection_service_credits",
      descriptorId: "service-credits",
      status: "expired",
      requestedAt: "2026-07-18T08:20:00Z",
      humanIntent: "Present a bounded compute-service credit.",
      action: "Present service credit",
      objectFamily: "service_right",
      exactScope: "Compute class C2 · 3 uses",
      uses: "0 of 3 remaining",
      expiry: "2026-07-25T08:20:00Z",
      delegation: "Not allowed",
      value: { amount: "None", unit: "Not applicable" },
      fee: { amount: "None", unit: "Not applicable", path: "No fee path" },
      disclosures: ["service class", "uses", "expiry"],
      revokeBehavior: "Expired authority cannot be reused; a new intent requires a new review.",
      reauth: "A replacement grant requires fresh confirmation."
    }
  ]);

  const DAPP_PERMISSION_FIXTURES = deepFreeze([
    {
      id: "permission_scoped_expenses",
      descriptorId: "scoped-expenses",
      connectionId: "connection_scoped_expenses",
      status: "active",
      scope: "Operations · native asset · maximum 75 Z00Z per request",
      uses: "8 of 12 remaining",
      expiresAt: "2026-08-02T00:00:00Z",
      delegation: "Attenuation only",
      revokeBehavior: "Reject future proposals without rewriting prior wallet outcomes."
    },
    {
      id: "permission_private_voucher",
      descriptorId: "private-voucher",
      connectionId: null,
      status: "expiring",
      scope: "Community voucher class · claim inspection only",
      uses: "2 of 5 remaining",
      expiresAt: "2026-07-27T18:00:00Z",
      delegation: "Not allowed",
      revokeBehavior: "Stop future claim inspection; issued objects remain wallet-owned."
    },
    {
      id: "permission_service_credits",
      descriptorId: "service-credits",
      connectionId: "connection_service_credits",
      status: "expired",
      scope: "Compute class C2 · presentation only",
      uses: "0 of 3 remaining",
      expiresAt: "2026-07-25T08:20:00Z",
      delegation: "Not allowed",
      revokeBehavior: "Already expired; no usable authority remains."
    }
  ]);

  const DAPP_ACTIVITY_FIXTURES = deepFreeze([
    {
      id: "dapp_event_0004",
      descriptorId: "scoped-expenses",
      kind: "intent_accepted",
      outcome: "Accepted for Wallet review",
      occurredAt: "2026-07-26T08:42:00Z",
      summary: "A bounded expense proposal passed app-level review; settlement was not implied."
    },
    {
      id: "dapp_event_0003",
      descriptorId: "agent-budget",
      kind: "intent_rejected",
      outcome: "Rejected",
      occurredAt: "2026-07-26T07:10:00Z",
      summary: "A concept proposal was rejected before any wallet operation was created."
    },
    {
      id: "dapp_event_0002",
      descriptorId: "service-credits",
      kind: "permission_expired",
      outcome: "Expired",
      occurredAt: "2026-07-25T08:20:00Z",
      summary: "The bounded service-credit presentation grant reached its declared expiry."
    },
    {
      id: "dapp_event_0001",
      descriptorId: "private-voucher",
      kind: "permission_reviewed",
      outcome: "Approved locally",
      occurredAt: "2026-07-24T16:05:00Z",
      summary: "A claim-inspection scope was approved without issuing or redeeming a Voucher."
    }
  ]);

  const DAPP_DESCRIPTOR_IDS = deepFreeze(DAPP_CATALOG.map(({ id }) => id));
  const DAPP_DESCRIPTOR_LUT = deepFreeze(Object.fromEntries(
    DAPP_CATALOG.map((entry) => [entry.id, entry])
  ));

  function dappDescriptor(descriptorId) {
    return DAPP_DESCRIPTOR_LUT[descriptorId] || null;
  }

  function assertDappCatalog() {
    const validMaturity = new Set(demo.PORT_CONTRACT.maturity);
    const validAvailability = new Set(demo.PORT_CONTRACT.availability);
    const validEvidence = new Set(demo.PORT_CONTRACT.evidenceSources);
    const validFreshness = new Set(demo.PORT_CONTRACT.freshness);
    const validPresentation = new Set(demo.PORT_CONTRACT.presentationModes);
    const seenIds = new Set();
    const seenIntentTypes = new Set();
    const seenIcons = new Set();

    if (DAPP_CATALOG.length !== 6) {
      throw new Error("The local dApp catalogue must contain exactly six descriptors.");
    }

    for (const entry of DAPP_CATALOG) {
      if (seenIds.has(entry.id)) throw new Error(`Duplicate dApp descriptor ID: ${entry.id}`);
      if (seenIntentTypes.has(entry.intentType)) throw new Error(`Duplicate dApp intent type: ${entry.intentType}`);
      if (seenIcons.has(entry.iconName)) throw new Error(`Duplicate dApp icon: ${entry.iconName}`);
      seenIds.add(entry.id);
      seenIntentTypes.add(entry.intentType);
      seenIcons.add(entry.iconName);

      if (!validMaturity.has(entry.maturity)
        || !validAvailability.has(entry.availability)
        || !validEvidence.has(entry.evidenceSource)
        || !validFreshness.has(entry.freshness)
        || !validPresentation.has(entry.presentationMode)) {
        throw new Error(`Invalid capability axes for dApp descriptor: ${entry.id}`);
      }
      if (!DAPP_INTENT_TYPES.includes(entry.intentType)) {
        throw new Error(`Unknown dApp intent type: ${entry.intentType}`);
      }
      if (!entry.requestedObjectFamilies.length
        || entry.requestedObjectFamilies.some((family) => !DAPP_OBJECT_FAMILIES.includes(family))) {
        throw new Error(`Invalid object family for dApp descriptor: ${entry.id}`);
      }
      if (entry.publisher.verified
        || entry.availability !== "unavailable"
        || entry.presentationMode !== "roadmap_preview"
        || entry.executionBoundary !== "typed_intent_only"
        || entry.remoteCodeAllowed
        || entry.walletBridgeAllowed) {
        throw new Error(`Unsafe dApp execution claim: ${entry.id}`);
      }
    }

    for (const entry of [...DAPP_CONNECTION_FIXTURES, ...DAPP_PERMISSION_FIXTURES, ...DAPP_ACTIVITY_FIXTURES]) {
      if (!DAPP_DESCRIPTOR_LUT[entry.descriptorId]) {
        throw new Error(`Unknown dApp fixture descriptor: ${entry.descriptorId}`);
      }
    }

    if (/((?:https?:)?\/\/)|\b(?:url|domain|iframe|bundle|executable|sourceCode)\b/i.test(JSON.stringify(DAPP_CATALOG))) {
      throw new Error("The local dApp catalogue must not contain remote or executable application descriptors.");
    }
    return true;
  }

  assertDappCatalog();

  Object.assign(root.Z00ZDemo, {
    DAPP_OBJECT_FAMILIES,
    DAPP_INTENT_TYPES,
    DAPP_CATALOG,
    DAPP_DESCRIPTOR_IDS,
    DAPP_DESCRIPTOR_LUT,
    DAPP_CONNECTION_FIXTURES,
    DAPP_PERMISSION_FIXTURES,
    DAPP_ACTIVITY_FIXTURES,
    dappDescriptor,
    assertDappCatalog
  });
})(typeof window === "undefined" ? globalThis : window);
