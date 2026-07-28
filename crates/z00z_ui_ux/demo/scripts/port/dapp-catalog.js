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
    "service_right",
    "external_asset_right"
  ]);

  const DAPP_INTENT_TYPES = deepFreeze([
    "prepare_offline_payment",
    "create_payment_request",
    "issue_private_voucher",
    "create_bounded_permission",
    "propose_agent_budget",
    "prepare_wbold_gateway",
    "create_bounded_subscription",
    "prepare_private_donation",
    "create_private_escrow",
    "create_bounty_claim",
    "issue_private_pass",
    "issue_service_credit",
    "issue_digital_entitlement",
    "prepare_private_payroll",
    "prepare_external_asset_lock"
  ]);

  const descriptor = ({
    id,
    label,
    summary,
    createdArtifact,
    purpose,
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
    reviewBoundary,
    actionLabel,
    proposalFields,
    walletChecks,
    settlementPath,
    evidenceOutput
  }) => deepFreeze({
    id,
    label,
    summary,
    createdArtifact,
    purpose,
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
    routeId: `dapps.${id}`,
    helpTopicId: `dapps.${id}`,
    executionBoundary: "typed_intent_only",
    remoteCodeAllowed: false,
    walletBridgeAllowed: false,
    actionLabel,
    proposalFields,
    walletChecks,
    settlementPath,
    evidenceOutput
  });

  const DAPP_CATALOG = deepFreeze([
    descriptor({
      id: "pay",
      label: "Pay",
      summary: "Propose a private payment with an explicit recipient, value ceiling, and connectivity mode.",
      createdArtifact: "A typed private-payment proposal for Wallet review.",
      purpose: "Pay one declared recipient without giving the dApp input selection, signing, or settlement authority.",
      useCaseFamily: "private_payment",
      iconName: "dapp-pay",
      intentType: "prepare_offline_payment",
      requestedObjectFamilies: ["asset"],
      offlineMode: "prepared_package",
      offlineSummary: "A handoff may be prepared with delayed connectivity; local acceptance and checkpoint settlement remain separate states.",
      disclosures: ["asset_family", "bounded_value", "expiry", "recipient_commitment"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The dApp cannot select wallet inputs, sign, or claim settlement.",
      actionLabel: "Review payment in Wallet",
      proposalFields: [
        { id: "recipient", label: "Recipient or payment request", type: "text", placeholder: "Receiver card or request ID", required: true },
        { id: "asset", label: "Asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "amount", label: "Amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001", suffix: "Must be greater than zero · selected asset" },
        { id: "expiry", label: "Proposal expiry", type: "select", options: ["30 minutes", "24 hours", "7 days"] },
        { id: "connectivity", label: "Connectivity", type: "select", options: ["Online", "Delayed connectivity", "Prepared offline handoff"] }
      ],
      walletChecks: ["Recipient scope", "Spendable inputs", "Value and fee", "Connectivity risk"],
      settlementPath: "Payment package → local presentation → publication → checkpoint",
      evidenceOutput: "Payment receipt and checkpoint reference"
    }),
    descriptor({
      id: "request",
      label: "Request",
      summary: "Create a private invoice or payment request without exposing a public account graph.",
      createdArtifact: "A portable signed payment-request proposal for QR or file handoff.",
      purpose: "Ask a payer for bounded value without creating debit authority or exposing a reusable public account.",
      useCaseFamily: "private_invoice",
      iconName: "dapp-request",
      intentType: "create_payment_request",
      requestedObjectFamilies: ["claim"],
      offlineMode: "portable_request",
      offlineSummary: "The request can travel as a QR or file; payment and settlement are separate wallet decisions.",
      disclosures: ["requested_asset", "bounded_value", "expiry", "request_commitment"],
      catalogueState: "approved",
      reviewBoundary: "A request is not a debit authority and cannot pull value from another wallet.",
      actionLabel: "Review request in Wallet",
      proposalFields: [
        { id: "asset", label: "Requested asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "amount", label: "Requested amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "amount-rule", label: "Amount rule", type: "select", options: ["Exact total", "Minimum total"] },
        { id: "payment-mode", label: "Payment mode", type: "select", options: ["Single payment", "Partial payments allowed", "Multiple payers allowed"] },
        { id: "expiry", label: "Expires", type: "select", options: ["1 hour", "24 hours", "7 days"] },
        { id: "business-reference", label: "Business reference", type: "text", placeholder: "Optional private order or invoice reference" },
        { id: "attachment-digest", label: "Attachment digest", type: "text", placeholder: "Optional document hash, never a local path" },
        { id: "memo", label: "Private memo", type: "text", placeholder: "Shared only with the payer" }
      ],
      walletChecks: ["Receiver card scope", "Request expiry", "Disclosure fields", "Proof-of-payment return path"],
      settlementPath: "Signed request → payer review → payment package → checkpoint",
      evidenceOutput: "Request digest and proof-of-payment reference"
    }),
    descriptor({
      id: "create-voucher",
      label: "Create Voucher",
      summary: "Propose conditional value with fixed, inspectable policy primitives.",
      createdArtifact: "A fixed-policy voucher definition and issuance proposal.",
      purpose: "Create conditional value whose backing, redemption scope, uses, expiry, transfer, and refund policy remain inspectable.",
      useCaseFamily: "voucher_issuance",
      iconName: "voucher",
      intentType: "issue_private_voucher",
      requestedObjectFamilies: ["voucher", "claim"],
      offlineMode: "local_handoff",
      offlineSummary: "A voucher handoff may be carried locally; redemption and issuer policy are reviewed separately.",
      disclosures: ["voucher_class", "merchant_scope", "uses", "expiry"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The dApp proposes a fixed voucher policy; Wallet remains the issuer authority boundary.",
      actionLabel: "Review voucher in Wallet",
      proposalFields: [
        { id: "class", label: "Voucher class", type: "text", placeholder: "Coffee, travel, event credit", required: true },
        { id: "asset", label: "Backing asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "value", label: "Face value", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "scope", label: "Merchant or service scope", type: "text", placeholder: "Exact merchant or service class", required: true },
        { id: "uses", label: "Maximum redemptions", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "partial", label: "Partial redemption", type: "select", options: ["Forbidden", "Allowed up to remaining value"] },
        { id: "expiry", label: "Expiry", type: "select", options: ["24 hours", "30 days", "90 days"] },
        { id: "transferability", label: "Transferability", type: "select", options: ["Not transferable", "One transfer", "Transferable until first use"] },
        { id: "refund", label: "Unused-value fallback", type: "select", options: ["Return to issuer at expiry", "Holder requests issuer refund", "No automatic refund"] }
      ],
      walletChecks: ["Issuer authority", "Backing value", "Policy bounds", "Refund path"],
      settlementPath: "Voucher definition → issue package → holder claim → redemption checkpoint",
      evidenceOutput: "Voucher definition digest and issuance receipt"
    }),
    descriptor({
      id: "create-permission",
      label: "Create Permission",
      summary: "Propose bounded authority without sharing a wallet key or generic signing access.",
      createdArtifact: "A bounded permission definition for one recipient and exact scope.",
      purpose: "Delegate limited authority without sharing a private key or generic signing capability.",
      useCaseFamily: "bounded_authority",
      iconName: "permission",
      intentType: "create_bounded_permission",
      requestedObjectFamilies: ["permission"],
      offlineMode: "policy_snapshot",
      offlineSummary: "The scope can be inspected offline; current authority is re-checked before issuance.",
      disclosures: ["permission_scope", "uses", "expiry", "delegation"],
      catalogueState: "approved",
      feePath: "separate_wallet_review",
      reviewBoundary: "The permission cannot expand its own scope, value, uses, delegation, or expiry.",
      actionLabel: "Review permission in Wallet",
      proposalFields: [
        { id: "recipient", label: "Recipient", type: "text", placeholder: "Receiver or agent commitment", required: true },
        { id: "scope", label: "Exact authority scope", type: "text", placeholder: "Action, object family, and service", required: true },
        { id: "uses", label: "Number of uses", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "expiry", label: "Expiry", type: "select", options: ["30 minutes", "24 hours", "30 days"] },
        { id: "delegation", label: "Delegation", type: "select", options: ["Forbidden", "Attenuation only"] }
      ],
      walletChecks: ["Issuer authority", "Exact scope", "Uses and expiry", "Delegation ceiling"],
      settlementPath: "Permission definition → wallet issuance → recipient acceptance",
      evidenceOutput: "Permission digest and issuance receipt"
    }),
    descriptor({
      id: "agents-budget",
      label: "Agents Budget",
      summary: "Give an agent a bounded spending right instead of wallet credentials.",
      createdArtifact: "A revocable agent-budget permission with provider, value, action-count, and time ceilings.",
      purpose: "Let one named agent propose approved service expenses without receiving Wallet credentials.",
      useCaseFamily: "bounded_agent_action",
      maturity: "concept",
      iconName: "dapp-agents-budget",
      intentType: "propose_agent_budget",
      requestedObjectFamilies: ["asset", "permission"],
      offlineMode: "proposal_only",
      offlineSummary: "Only the bounded proposal persists locally; each value or fee path remains a Wallet review.",
      disclosures: ["service_allowlist", "bounded_value", "uses", "expiry", "delegation"],
      valuePath: "separate_wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "No private key, generic signing, hidden provider, or unbounded autonomous spend is permitted.",
      actionLabel: "Review agent budget in Wallet",
      proposalFields: [
        { id: "agent", label: "Agent identity", type: "text", placeholder: "Local agent commitment", required: true },
        { id: "services", label: "Service allowlist", type: "text", placeholder: "Compute, storage, or exact provider", required: true },
        { id: "asset", label: "Budget asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "period-limit", label: "Budget per day", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "action-limit", label: "Maximum per action", type: "number", placeholder: "0.00", required: true, min: "0.00000001", suffix: "Cannot exceed the daily budget" },
        { id: "max-actions", label: "Maximum actions per day", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "approval", label: "Human approval threshold", type: "number", placeholder: "0.00", required: true, min: "0.00000001", suffix: "Cannot exceed the per-action ceiling" },
        { id: "expiry", label: "Expiry", type: "select", options: ["30 minutes", "24 hours", "7 days"] }
      ],
      walletChecks: ["Agent identity", "Provider allowlist", "Value and fee ceilings", "Emergency revoke"],
      settlementPath: "Budget permission → per-action proposal → Wallet confirmation → settlement",
      evidenceOutput: "Budget grant and per-action audit receipts"
    }),
    descriptor({
      id: "wbold-gateway",
      label: "wBOLD Gateway",
      summary: "Propose a bounded deposit or redemption route for externally backed wBOLD.",
      createdArtifact: "A typed external deposit or redemption route proposal.",
      purpose: "Bridge one declared BOLD or wBOLD amount through an explicit operator while keeping external custody risk visible.",
      useCaseFamily: "external_asset_gateway",
      iconName: "dapp-wbold-gateway",
      intentType: "prepare_wbold_gateway",
      requestedObjectFamilies: ["asset", "external_asset_right"],
      offlineMode: "reconnect_required",
      offlineSummary: "The proposal is inspectable offline; external finality, backing, and redemption status require their authorities.",
      disclosures: ["asset_family", "route", "bounded_value", "external_recipient"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Z00Z verifies the internal proposal boundary, not external custody, solvency, or redemption.",
      actionLabel: "Review gateway route in Wallet",
      proposalFields: [
        { id: "direction", label: "Route", type: "select", options: ["Deposit BOLD → receive wBOLD", "Redeem wBOLD → receive BOLD"] },
        { id: "amount", label: "Amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "external-recipient", label: "External recipient", type: "text", placeholder: "Required for redemption", suffix: "Must match the selected external network when redeeming" },
        { id: "operator", label: "Locker route", type: "select", options: ["BOLD Locker · Ethereum"] },
        { id: "max-fee", label: "Maximum route fee", type: "number", placeholder: "0.00", required: true, min: "0", suffix: "May be zero; Wallet rejects any larger fee" }
      ],
      walletChecks: ["Route status", "External finality", "Replay-safe deposit or exit", "Fee and recipient"],
      settlementPath: "External event ↔ adapter proof ↔ wBOLD package ↔ checkpoint",
      evidenceOutput: "External event reference, internal receipt, and route status"
    }),
    descriptor({
      id: "subscription",
      label: "Subscription",
      summary: "Propose bounded claims per period without an unlimited merchant allowance.",
      createdArtifact: "A bounded subscription policy split into a finite number of period claims.",
      purpose: "Authorize limited recurring service payments without granting an unlimited merchant pull allowance.",
      useCaseFamily: "recurring_service",
      iconName: "dapp-subscription",
      intentType: "create_bounded_subscription",
      requestedObjectFamilies: ["voucher", "permission", "service_right"],
      offlineMode: "period_slices",
      offlineSummary: "Only already issued period slices can travel offline; a new period requires policy review.",
      disclosures: ["provider", "amount_per_period", "period_count", "expiry"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The provider never receives an unlimited pull permission or generic wallet access.",
      actionLabel: "Review subscription in Wallet",
      proposalFields: [
        { id: "provider", label: "Provider", type: "text", placeholder: "Exact service provider", required: true },
        { id: "plan", label: "Service plan", type: "text", placeholder: "Plan or service class", required: true },
        { id: "asset", label: "Billing asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "amount", label: "Amount per period", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "period", label: "Period", type: "select", options: ["Weekly", "Monthly", "Yearly"] },
        { id: "periods", label: "Maximum periods", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "expiry", label: "Policy expiry", type: "select", options: ["30 days", "1 year", "At final period"] }
      ],
      walletChecks: ["Provider scope", "Amount per period", "Maximum periods", "Cancellation and expiry"],
      settlementPath: "Subscription policy → bounded period claim → confirmation → checkpoint",
      evidenceOutput: "Subscription policy and per-period receipts"
    }),
    descriptor({
      id: "donation",
      label: "Donation",
      summary: "Propose a private one-time or bounded recurring contribution with selective disclosure.",
      createdArtifact: "A private one-time or finite recurring donation proposal.",
      purpose: "Contribute bounded value to one beneficiary while controlling receipt disclosure.",
      useCaseFamily: "private_donation",
      iconName: "dapp-donation",
      intentType: "prepare_private_donation",
      requestedObjectFamilies: ["asset", "claim"],
      offlineMode: "prepared_package",
      offlineSummary: "A donation package may be prepared locally; beneficiary payout and project reporting remain external facts.",
      disclosures: ["beneficiary", "bounded_value", "recurrence", "receipt_disclosure"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The dApp cannot debit the wallet or attest how the beneficiary uses funds.",
      actionLabel: "Review donation in Wallet",
      proposalFields: [
        { id: "beneficiary", label: "Beneficiary or project", type: "text", placeholder: "Verified recipient commitment", required: true },
        { id: "asset", label: "Donation asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "amount", label: "Amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "schedule", label: "Schedule", type: "select", options: ["One time", "Monthly · 3 periods", "Monthly · 12 periods"] },
        { id: "disclosure", label: "Donor receipt", type: "select", options: ["Private receipt", "Disclose amount to beneficiary", "Anonymous aggregate only"] }
      ],
      walletChecks: ["Beneficiary identity", "Amount and recurrence", "Disclosure purpose", "Cancellation boundary"],
      settlementPath: "Donation proposal → Wallet package → beneficiary receipt → checkpoint",
      evidenceOutput: "Private donor receipt and optional aggregate reference"
    }),
    descriptor({
      id: "escrow",
      label: "Escrow",
      summary: "Propose a bounded conditional release without making Z00Z the arbitrator.",
      createdArtifact: "A conditional-value proposal with release evidence, timeout, and mandatory fallback.",
      purpose: "Hold bounded value until declared evidence or timeout selects an explicit release path.",
      useCaseFamily: "private_escrow",
      maturity: "concept",
      iconName: "dapp-escrow",
      intentType: "create_private_escrow",
      requestedObjectFamilies: ["voucher", "permission"],
      offlineMode: "policy_snapshot",
      offlineSummary: "Terms can be inspected offline; release, dispute, and timeout authority require current evidence.",
      disclosures: ["counterparty", "release_condition", "timeout", "arbitrator_scope"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Z00Z provides a conditional object; an independent service supplies any arbitration decision.",
      actionLabel: "Review escrow in Wallet",
      proposalFields: [
        { id: "counterparty", label: "Counterparty", type: "text", placeholder: "Recipient commitment", required: true },
        { id: "asset", label: "Escrow asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "amount", label: "Escrow amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "condition", label: "Release condition", type: "text", placeholder: "Milestone or delivery evidence", required: true },
        { id: "timeout", label: "Timeout", type: "select", options: ["24 hours", "7 days", "30 days"] },
        { id: "fallback", label: "Fallback recipient", type: "text", placeholder: "Return or alternate recipient", required: true, suffix: "Mandatory timeout destination" },
        { id: "arbitrator", label: "Arbitration service", type: "text", placeholder: "Optional independent authority" }
      ],
      walletChecks: ["Counterparty", "Release and timeout", "Fallback path", "Arbitrator scope"],
      settlementPath: "Escrow voucher → evidence or timeout → release proposal → checkpoint",
      evidenceOutput: "Escrow terms, release receipt, and evidence digest"
    }),
    descriptor({
      id: "bounties",
      label: "Bounties",
      summary: "Propose a reward claim whose payout requires an explicit verifier decision.",
      createdArtifact: "A backed bounty definition with verifier, deadline, and evidence requirements.",
      purpose: "Offer a bounded reward whose payout remains dependent on a separate verifier decision and Wallet confirmation.",
      useCaseFamily: "verified_reward",
      maturity: "concept",
      iconName: "dapp-bounties",
      intentType: "create_bounty_claim",
      requestedObjectFamilies: ["claim", "permission", "asset"],
      offlineMode: "proof_carry",
      offlineSummary: "A submission digest can be carried locally; verification and payout authority remain explicit.",
      disclosures: ["task_scope", "reward", "verifier", "submission_deadline"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Publishing a bounty does not validate a submission or authorize payout.",
      actionLabel: "Review bounty in Wallet",
      proposalFields: [
        { id: "task", label: "Task or result scope", type: "text", placeholder: "Exact deliverable", required: true },
        { id: "verifier", label: "Verifier", type: "text", placeholder: "Independent verifier commitment", required: true },
        { id: "asset", label: "Reward asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "reward", label: "Reward", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "deadline", label: "Submission deadline", type: "select", options: ["24 hours", "7 days", "30 days"] },
        { id: "evidence", label: "Evidence requirement", type: "text", placeholder: "Digest, receipt, or proof family", required: true }
      ],
      walletChecks: ["Reward backing", "Verifier authority", "Deadline", "Claim and evidence scope"],
      settlementPath: "Bounty claim → submission evidence → verifier right → payout package",
      evidenceOutput: "Bounty definition, verifier decision, and payout receipt"
    }),
    descriptor({
      id: "tickets-passes",
      label: "Tickets & Passes",
      summary: "Issue a private event, transport, membership, or access pass with bounded use.",
      createdArtifact: "A bounded private access-pass definition.",
      purpose: "Grant a declared event, transport, membership, or venue right with explicit uses, validity, transfer, and offline policy.",
      useCaseFamily: "private_access",
      iconName: "dapp-tickets-passes",
      intentType: "issue_private_pass",
      requestedObjectFamilies: ["voucher", "service_right"],
      offlineMode: "local_verification",
      offlineSummary: "A pass may be presented offline inside its freshness and verifier policy; settlement remains explicit.",
      disclosures: ["service_scope", "uses", "validity_window", "transferability"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "A pass proves only the declared access right, not identity or universal admission.",
      actionLabel: "Review pass in Wallet",
      proposalFields: [
        { id: "service", label: "Event or service", type: "text", placeholder: "Event, transport, membership, or venue", required: true },
        { id: "uses", label: "Number of uses", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "validity", label: "Validity", type: "select", options: ["Single day", "7 days", "30 days"] },
        { id: "transferability", label: "Transferability", type: "select", options: ["Not transferable", "Transfer once", "Transfer until first use"] },
        { id: "offline", label: "Offline verification", type: "select", options: ["Disabled", "Short freshness window", "Venue policy"] }
      ],
      walletChecks: ["Issuer identity", "Service scope", "Uses and validity", "Offline verifier policy"],
      settlementPath: "Pass definition → holder claim → presentation → redemption checkpoint",
      evidenceOutput: "Pass digest and bounded redemption receipt"
    }),
    descriptor({
      id: "service-credits",
      label: "Service Credits",
      summary: "Model bounded API, data, compute, or access rights as explicit service credits.",
      createdArtifact: "A bounded provider-specific service-credit definition.",
      purpose: "Issue metered service rights that are neither money nor proof that the external service was delivered.",
      useCaseFamily: "service_access",
      iconName: "dapp-service-credits",
      intentType: "issue_service_credit",
      requestedObjectFamilies: ["service_right", "permission"],
      offlineMode: "proof_carry",
      offlineSummary: "A bounded right can be carried for later presentation; service delivery remains externally authoritative.",
      disclosures: ["service_class", "permission_scope", "uses", "expiry"],
      catalogueState: "approved",
      reviewBoundary: "A service credit is not money, a universal entitlement, or proof that service was delivered.",
      actionLabel: "Review credit issuance in Wallet",
      proposalFields: [
        { id: "provider", label: "Provider", type: "text", placeholder: "Exact service provider", required: true },
        { id: "service", label: "Service class", type: "text", placeholder: "API, compute, storage, or access", required: true },
        { id: "quota", label: "Quota", type: "number", placeholder: "100", required: true, min: "0.00000001", suffix: "Must be greater than zero in the selected unit" },
        { id: "unit", label: "Metering unit", type: "select", options: ["Calls", "Minutes", "GiB", "Requests"] },
        { id: "expiry", label: "Expiry", type: "select", options: ["24 hours", "30 days", "90 days"] },
        { id: "delegation", label: "Delegation", type: "select", options: ["Forbidden", "Attenuation only"] }
      ],
      walletChecks: ["Provider identity", "Service and quota", "Issuance authority", "Expiry and redemption policy"],
      settlementPath: "Service-right definition → issuance → later presentation → provider decision",
      evidenceOutput: "Credit definition and issuance receipt; later presentation can produce a redemption receipt"
    }),
    descriptor({
      id: "digital-goods",
      label: "Digital Goods",
      summary: "Issue a bounded software licence, download entitlement, plugin, or digital item right.",
      createdArtifact: "A bounded digital-entitlement definition.",
      purpose: "Grant the declared licence or item right without executing content or asserting successful external delivery.",
      useCaseFamily: "digital_entitlement",
      iconName: "dapp-digital-goods",
      intentType: "issue_digital_entitlement",
      requestedObjectFamilies: ["service_right", "voucher"],
      offlineMode: "proof_carry",
      offlineSummary: "The entitlement can be carried locally; content delivery and quality remain provider facts.",
      disclosures: ["provider", "item_class", "uses", "expiry"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The right grants only the declared item or use; it does not execute downloaded code.",
      actionLabel: "Review digital good in Wallet",
      proposalFields: [
        { id: "provider", label: "Provider", type: "text", placeholder: "Publisher or service provider", required: true },
        { id: "item", label: "Item or licence", type: "text", placeholder: "Exact digital entitlement", required: true },
        { id: "uses", label: "Uses or devices", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "expiry", label: "Validity", type: "select", options: ["Perpetual", "30 days", "1 year"] },
        { id: "transferability", label: "Transferability", type: "select", options: ["Not transferable", "Transfer once", "Transferable before activation"] }
      ],
      walletChecks: ["Provider identity", "Item scope", "Uses and validity", "Delivery commitment"],
      settlementPath: "Entitlement definition → issue package → provider presentation",
      evidenceOutput: "Entitlement digest and delivery/redemption receipt"
    }),
    descriptor({
      id: "payroll",
      label: "Payroll",
      summary: "Propose a private batch payout with aggregate totals and narrow employee receipts.",
      createdArtifact: "A sanitized private payroll-batch proposal with expected count and aggregate ceiling.",
      purpose: "Prepare per-recipient payouts and scoped receipts without exposing treasury keys or a public payroll graph.",
      useCaseFamily: "private_payroll",
      maturity: "concept",
      iconName: "dapp-payroll",
      intentType: "prepare_private_payroll",
      requestedObjectFamilies: ["asset", "permission", "claim"],
      offlineMode: "batch_draft",
      offlineSummary: "A sanitized batch draft can be reviewed locally; recipients and totals are revalidated before every package.",
      disclosures: ["batch_total", "recipient_count", "schedule", "audit_scope"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The dApp cannot access treasury keys, sign a batch, or expose the payroll graph publicly.",
      actionLabel: "Review payroll batch in Wallet",
      proposalFields: [
        { id: "batch", label: "Payroll batch", type: "text", placeholder: "Private batch label", required: true },
        { id: "asset", label: "Payout asset", type: "select", options: ["zBOLD", "Z00Z"] },
        { id: "recipient-set", label: "Recipient set reference", type: "text", placeholder: "Imported encrypted batch or local batch ID", required: true, suffix: "A typed local reference, never a filesystem path" },
        { id: "recipients", label: "Expected recipient count", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true, suffix: "Wallet revalidates the imported recipient set" },
        { id: "total", label: "Aggregate amount ceiling", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "schedule", label: "Schedule", type: "select", options: ["Prepare now", "Next checkpoint", "Scheduled date"] },
        { id: "audit", label: "Audit output", type: "select", options: ["Aggregate only", "Employee receipts", "Scoped auditor package"] }
      ],
      walletChecks: ["Treasury authority", "Recipient commitments", "Aggregate total and fees", "Disclosure scope"],
      settlementPath: "Payroll batch → per-recipient packages → publication → aggregate evidence",
      evidenceOutput: "Employee receipts and aggregate batch proof"
    }),
    descriptor({
      id: "assets-locker",
      label: "Assets Locker",
      summary: "Represent a bounded private right over one explicit external custody route.",
      createdArtifact: "A typed external lock/import or consume/redeem route proposal.",
      purpose: "Represent one external custody right while preserving the operator, network, reserve, pause, and redemption trust boundary.",
      useCaseFamily: "external_custody_right",
      iconName: "dapp-assets-locker",
      intentType: "prepare_external_asset_lock",
      requestedObjectFamilies: ["asset", "external_asset_right", "permission"],
      offlineMode: "reconnect_required",
      offlineSummary: "The proposal is inspectable offline; custody, reserve, pause, and redemption status require external evidence.",
      disclosures: ["asset_family", "custody_class", "route", "bounded_value", "external_recipient"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Z00Z does not become custodian and cannot guarantee external reserves or redemption.",
      actionLabel: "Review locker route in Wallet",
      proposalFields: [
        { id: "asset", label: "External asset", type: "text", placeholder: "Exact asset and network", required: true },
        { id: "action", label: "Action", type: "select", options: ["Lock and import right", "Consume right and redeem"] },
        { id: "amount", label: "Amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "route", label: "Locker operator and route", type: "text", placeholder: "One explicit custody route", required: true },
        { id: "external-recipient", label: "External recipient", type: "text", placeholder: "Required when consuming the right to redeem", suffix: "Must match the external asset network" },
        { id: "risk", label: "Trust tier", type: "select", options: ["External asset · explicit operator"] },
        { id: "max-fee", label: "Maximum route fee", type: "number", placeholder: "0.00", required: true, min: "0" }
      ],
      walletChecks: ["Asset and network", "Operator status", "Replay protection", "Reserve/redemption disclosure"],
      settlementPath: "External lock ↔ adapter proof ↔ private right ↔ external release",
      evidenceOutput: "Custody event reference, internal receipt, and redemption status"
    })
  ]);

  const DAPP_CONNECTION_FIXTURES = deepFreeze([
    {
      id: "connection_offline_pay",
      descriptorId: "pay",
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
      descriptorId: "agents-budget",
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
      humanIntent: "Issue one bounded compute-service credit.",
      action: "Issue service credit",
      objectFamily: "service_right",
      exactScope: "Compute class C2 · issue 3 metered uses",
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
      descriptorId: "agents-budget",
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
      descriptorId: "create-voucher",
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
      scope: "Compute class C2 · bounded issuance only",
      uses: "0 of 3 remaining",
      expiresAt: "2026-07-25T08:20:00Z",
      delegation: "Not allowed",
      revokeBehavior: "Already expired; no usable authority remains."
    }
  ]);

  const DAPP_ACTIVITY_FIXTURES = deepFreeze([
    {
      id: "dapp_event_0004",
      descriptorId: "agents-budget",
      kind: "intent_accepted",
      outcome: "Accepted for Wallet review",
      occurredAt: "2026-07-26T08:42:00Z",
      summary: "A bounded expense proposal passed app-level review; settlement was not implied."
    },
    {
      id: "dapp_event_0003",
      descriptorId: "agents-budget",
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
      descriptorId: "create-voucher",
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

    if (DAPP_CATALOG.length !== 15) {
      throw new Error("The local dApp catalogue must contain exactly fifteen descriptors.");
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
      if (!entry.createdArtifact?.trim() || !entry.purpose?.trim()) {
        throw new Error(`Missing creation or purpose explanation for dApp descriptor: ${entry.id}`);
      }
      if (!entry.proposalFields.length || entry.proposalFields.some((field) => !field.id || !field.label || !field.type)) {
        throw new Error(`Invalid proposal field for dApp descriptor: ${entry.id}`);
      }
      if (entry.proposalFields.some((field) => field.type === "number" && Number(field.min) < 0)) {
        throw new Error(`Negative numeric bound for dApp descriptor: ${entry.id}`);
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
