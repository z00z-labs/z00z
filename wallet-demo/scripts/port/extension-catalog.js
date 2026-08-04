"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z demo contracts must load before the Extension catalogue.");
  }

  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };

  const EXTENSION_OBJECT_FAMILIES = deepFreeze([
    "asset",
    "voucher",
    "permission",
    "claim",
    "service_right",
    "external_asset_right"
  ]);

  const EXTENSION_INTENT_TYPES = deepFreeze([
    "prepare_offline_payment",
    "create_payment_request",
    "issue_private_voucher",
    "create_bounded_permission",
    "create_asset_definition",
    "propose_agent_budget",
    "prepare_wcoins_gateway",
    "create_bounded_subscription",
    "prepare_private_donation",
    "create_private_escrow",
    "create_bounty_claim",
    "issue_private_pass",
    "issue_service_credit",
    "issue_digital_entitlement",
    "prepare_private_payroll",
    "propose_private_contract",
    "prepare_external_asset_lock",
    "prepare_xchain_integration"
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
    evidenceOutput,
    gatewayAggregate = null,
    gatewayRoutes = [],
    gatewayControls = [],
    gatewayMarketNotes = [],
    integrations = [],
    quoteFields = [],
    integrationRecommendation = null,
    integrationInvariants = []
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
    routeId: `extensions.${id}`,
    helpTopicId: `extensions.${id}`,
    executionBoundary: "typed_intent_only",
    remoteCodeAllowed: false,
    walletBridgeAllowed: false,
    actionLabel,
    proposalFields,
    walletChecks,
    settlementPath,
    evidenceOutput,
    gatewayAggregate,
    gatewayRoutes,
    gatewayControls,
    gatewayMarketNotes,
    integrations,
    quoteFields,
    integrationRecommendation,
    integrationInvariants
  });

  const EXTENSION_CATALOG = deepFreeze([
    descriptor({
      id: "pay",
      label: "Pay",
      summary: "Propose a private payment with an explicit recipient, value ceiling, and connectivity mode.",
      createdArtifact: "A typed private-payment proposal for Wallet review.",
      purpose: "Pay one declared recipient without giving the Extension input selection, signing, or settlement authority.",
      useCaseFamily: "private_payment",
      iconName: "extension-pay",
      intentType: "prepare_offline_payment",
      requestedObjectFamilies: ["asset"],
      offlineMode: "prepared_package",
      offlineSummary: "A handoff may be prepared with delayed connectivity; local acceptance and checkpoint settlement remain separate states.",
      disclosures: ["asset_family", "bounded_value", "expiry", "recipient_commitment"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension cannot select wallet inputs, sign, or claim settlement.",
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
      iconName: "extension-request",
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
      iconName: "voucher-list",
      intentType: "issue_private_voucher",
      requestedObjectFamilies: ["voucher", "claim"],
      offlineMode: "local_handoff",
      offlineSummary: "A voucher handoff may be carried locally; redemption and issuer policy are reviewed separately.",
      disclosures: ["voucher_class", "merchant_scope", "uses", "expiry"],
      catalogueState: "approved",
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension proposes a fixed voucher policy; Wallet remains the issuer authority boundary.",
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
      iconName: "permission-list",
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
      id: "create-asset",
      label: "Create Asset",
      summary: "Propose an immutable asset definition with explicit class, denomination, serial, and policy bounds.",
      createdArtifact: "A typed immutable asset-definition and initial-issuance proposal.",
      purpose: "Define a coin, token, or NFT family without granting the Extension registry, signing, issuance, or settlement authority.",
      useCaseFamily: "asset_issuance",
      iconName: "assets",
      intentType: "create_asset_definition",
      requestedObjectFamilies: ["asset"],
      offlineMode: "definition_draft",
      offlineSummary: "The unsigned definition draft can be reviewed offline; registry acceptance and issuance still require Wallet and protocol checks.",
      disclosures: ["asset_family", "definition_fields", "issuance_bounds", "policy_flags"],
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension cannot register the definition, issue assets, select Wallet objects, sign, or claim settlement.",
      actionLabel: "Review asset in Wallet",
      proposalFields: [
        { id: "class", label: "Asset class", type: "select", options: ["Coin", "Token", "NFT"] },
        { id: "name", label: "Asset name", type: "text", placeholder: "Human-readable immutable name", required: true },
        { id: "symbol", label: "Symbol", type: "text", placeholder: "Short display symbol", required: true },
        { id: "decimals", label: "Decimals", type: "number", placeholder: "0", required: true, min: "0", step: "1", integer: true, suffix: "NFT definitions require zero decimals" },
        { id: "serials", label: "Declared serials", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true, suffix: "Number of serial instances declared by the definition" },
        { id: "nominal", label: "Nominal units per serial", type: "number", placeholder: "1", required: true, min: "1", step: "1", integer: true },
        { id: "namespace", label: "Issuer namespace", type: "text", placeholder: "Issuer or organization namespace", required: true },
        { id: "policy", label: "Supply policy", type: "select", options: ["Fixed supply", "Fixed supply · burnable", "Bounded additional issuance"] },
        { id: "metadata-digest", label: "Metadata digest", type: "text", placeholder: "Optional canonical metadata digest, never a local file path" }
      ],
      walletChecks: ["Asset-class constraints", "Immutable definition fields", "Issuer authority", "Supply and policy bounds"],
      settlementPath: "Definition review → registry proposal → issuance package → checkpoint",
      evidenceOutput: "Definition digest, registry decision, and issuance receipt"
    }),
    descriptor({
      id: "agents-budget",
      label: "Agent Budget",
      summary: "Give an agent a bounded spending right instead of wallet credentials.",
      createdArtifact: "A revocable agent-budget permission with provider, value, action-count, and time ceilings.",
      purpose: "Let one named agent propose approved service expenses without receiving Wallet credentials.",
      useCaseFamily: "bounded_agent_action",
      maturity: "concept",
      iconName: "extension-agents-budget",
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
      label: "wCoins Gateway",
      summary: "Propose a bounded deposit or redemption between a Z00Z wrapped coin and its route-specific external reserve.",
      createdArtifact: "A typed stablecoin deposit or redemption proposal bound to one immutable LockerID.",
      purpose: "Use route-specific USD, CHF, EUR, or JPY stablecoin candidates without hiding their distinct collateral, governance, oracle, liquidity, or custody risks.",
      useCaseFamily: "external_asset_gateway",
      iconName: "extension-wbold-gateway",
      intentType: "prepare_wcoins_gateway",
      requestedObjectFamilies: ["asset", "external_asset_right"],
      offlineMode: "reconnect_required",
      offlineSummary: "The proposal and last local route snapshot are inspectable offline; reserves, finality, status, and redemption availability require reconnection.",
      disclosures: ["asset_family", "reference_currency", "protocol_model", "locker_id", "reserve_network", "route", "bounded_value", "external_recipient"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "Each Z00Z asset and external reserve network pair has an independent reference currency, protocol model, LockerID, reserve pool, liabilities, redemption route, status, risk badge, and exposure cap. Z00Z verifies the proposal boundary, not external solvency.",
      actionLabel: "Review gateway route in Wallet",
      proposalFields: [
        {
          id: "route",
          label: "Stable asset route",
          type: "select",
          options: [
            "wBOLD ← BOLD · ETH",
            "wDAI ← DAI · ETH",
            "wCRVUSD ← CRVUSD · ETH",
            "wZCHF ← ZCHF · ETH",
            "wdEURO ← dEURO · ETH",
            "wCJPY ← CJPY · ETH"
          ],
          suffix: "The wCoin is issued inside Z00Z. Ethereum Mainnet is the external reserve network, not the wCoin network."
        },
        { id: "direction", label: "Action", type: "select", options: ["Deposit reserve → receive wCoin", "Redeem wCoin → receive reserve"] },
        { id: "amount", label: "Amount", type: "number", placeholder: "0.00", required: true, min: "0.00000001" },
        { id: "external-recipient", label: "External recipient", type: "text", placeholder: "Required for redemption", suffix: "Must be an address on the selected reserve network when redeeming" },
        { id: "max-fee", label: "Maximum route fee", type: "number", placeholder: "0.00", required: true, min: "0", suffix: "May be zero; Wallet rejects any larger fee" }
      ],
      walletChecks: ["Z00Z asset and reserve-asset mapping", "LockerID, reserve network, and route status", "Reserve and exposure cap", "External finality", "Replay-safe deposit or exit", "Fee and recipient"],
      settlementPath: "External event ↔ route-specific adapter proof ↔ wrapped stablecoin package ↔ checkpoint",
      evidenceOutput: "Z00Z asset, external reserve asset, reserve network, LockerID, external event reference, reserve snapshot, internal receipt, and route status",
      gatewayAggregate: "6 independent lockers",
      gatewayRoutes: [
        {
          id: "wbold",
          label: "wBOLD",
          z00zAsset: "wBOLD",
          externalAsset: "BOLD",
          referenceCurrency: "USD",
          protocolModel: "Liquity V2 overcollateralized CDP",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.bold.v1",
          reservePool: "3,400.00 BOLD",
          liabilities: "3,250.00 wBOLD",
          redemptionRoute: "wBOLD (Z00Z) → BOLD (Ethereum Mainnet)",
          riskBadge: "Governance-minimized",
          status: "Active",
          exposureLimit: "5,000.00 BOLD",
          positioning: "Governance-minimized private stable value.",
          uses: "Private payments · merchants · vouchers · scoped budgets · subscriptions · agent allowances"
        },
        {
          id: "wdai",
          label: "wDAI",
          z00zAsset: "wDAI",
          externalAsset: "DAI",
          referenceCurrency: "USD",
          protocolModel: "Governance-managed collateral system",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.dai.v1",
          reservePool: "6,950.00 DAI",
          liabilities: "6,800.00 wDAI",
          redemptionRoute: "wDAI (Z00Z) → DAI (Ethereum Mainnet)",
          riskBadge: "Governance-managed",
          status: "Active",
          exposureLimit: "10,000.00 DAI",
          positioning: "Highly liquid decentralized stable asset with governance-managed protocol risk.",
          uses: "Large ingress and egress · OTC · existing DAI holders · external DeFi · backup liquidity"
        },
        {
          id: "wcrvusd",
          label: "wCRVUSD",
          z00zAsset: "wCRVUSD",
          externalAsset: "CRVUSD",
          referenceCurrency: "USD",
          protocolModel: "LLAMMA crypto-collateralized debt",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.crvusd.v1",
          reservePool: "2,500.00 CRVUSD",
          liabilities: "2,400.00 wCRVUSD",
          redemptionRoute: "wCRVUSD (Z00Z) → CRVUSD (Ethereum Mainnet)",
          riskBadge: "DAO-managed",
          status: "Active",
          exposureLimit: "4,000.00 CRVUSD",
          positioning: "DAO-managed crypto-collateralized stable asset without issuer address blacklist.",
          uses: "Curve community · DEX liquidity · protocol-risk diversification · private DeFi settlement"
        },
        {
          id: "wzchf",
          label: "wZCHF",
          z00zAsset: "wZCHF",
          externalAsset: "ZCHF",
          referenceCurrency: "CHF",
          protocolModel: "Oracle-free positions, challenges, auctions, and reserve equity",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.zchf.v1",
          reservePool: "1,200.00 ZCHF",
          liabilities: "1,000.00 wZCHF",
          redemptionRoute: "wZCHF (Z00Z) → ZCHF (Ethereum Mainnet)",
          riskBadge: "Mixed collateral",
          status: "Candidate",
          exposureLimit: "2,000.00 ZCHF",
          positioning: "Most established non-USD candidate in this set; collateral and bridge composition remain route-specific risks.",
          uses: "CHF-denominated private settlement · payroll · merchant invoices · treasury diversification"
        },
        {
          id: "wdeuro",
          label: "wdEURO",
          z00zAsset: "wdEURO",
          externalAsset: "dEURO",
          referenceCurrency: "EUR",
          protocolModel: "Oracle-free positions plus governed stablecoin bridges",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.deuro.v1",
          reservePool: "850.00 dEURO",
          liabilities: "750.00 wdEURO",
          redemptionRoute: "wdEURO (Z00Z) → dEURO (Ethereum Mainnet)",
          riskBadge: "Early-stage · bridge-aware",
          status: "Candidate",
          exposureLimit: "1,250.00 dEURO",
          positioning: "Promising euro candidate with an oracle-free position core and additional bridge dependencies.",
          uses: "EUR-denominated private settlement · subscriptions · payroll · merchant invoices"
        },
        {
          id: "wcjpy",
          label: "wCJPY",
          z00zAsset: "wCJPY",
          externalAsset: "CJPY",
          referenceCurrency: "JPY",
          protocolModel: "ETH-backed CDP with Chainlink ETH/USD and USD/JPY feeds",
          reserveNetwork: "Ethereum Mainnet",
          lockerId: "locker.ethereum-mainnet.cjpy.v1",
          reservePool: "180,000 CJPY",
          liabilities: "150,000 wCJPY",
          redemptionRoute: "wCJPY (Z00Z) → CJPY (Ethereum Mainnet)",
          riskBadge: "Thin liquidity · oracle",
          status: "Candidate",
          exposureLimit: "250,000 CJPY",
          positioning: "ETH-backed JPY candidate; activation remains gated on verified market depth and peg quality.",
          uses: "JPY-denominated private settlement · merchant invoices · payroll · regional treasury routing"
        }
      ],
      gatewayControls: [
        "User reserves cannot be confiscated by the gateway.",
        "Wrapped coins cannot be minted without a confirmed deposit.",
        "Every external network is a separate route; an existing asset, network, or LockerID mapping cannot be changed retroactively.",
        "New deposits may be stopped by a circuit breaker.",
        "Any exit pause must have a hard deadline and a trust-minimized escape route after expiry.",
        "Upgrades require a long timelock or a new locker version.",
        "An old locker must retain redemption for its existing liabilities.",
        "Every route enforces its own exposure cap."
      ],
      gatewayMarketNotes: [
        "BOLD is the strongest candidate for minimizing administrative control and the strategic primary route for Z00Z.",
        "DAI is the strongest liquidity route and currently leads this candidate set by a wide margin.",
        "CRVUSD is the strongest compromise between decentralized architecture and meaningful trading activity.",
        "ZCHF is the strongest non-USD candidate in this set by protocol history and present scale, but its collateral and bridge dependencies are not equivalent to BOLD.",
        "dEURO is a promising new EUR candidate; current market activity and bridge composition require conservative pilot caps.",
        "CJPY is an ETH-collateralized JPY candidate; current market depth is insufficient for an unrestricted route."
      ]
    }),
    descriptor({
      id: "subscription",
      label: "Subscription",
      summary: "Propose bounded claims per period without an unlimited merchant allowance.",
      createdArtifact: "A bounded subscription policy split into a finite number of period claims.",
      purpose: "Authorize limited recurring service payments without granting an unlimited merchant pull allowance.",
      useCaseFamily: "recurring_service",
      iconName: "extension-subscription",
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
      iconName: "extension-donation",
      intentType: "prepare_private_donation",
      requestedObjectFamilies: ["asset", "claim"],
      offlineMode: "prepared_package",
      offlineSummary: "A donation package may be prepared locally; beneficiary payout and project reporting remain external facts.",
      disclosures: ["beneficiary", "bounded_value", "recurrence", "receipt_disclosure"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension cannot debit the wallet or attest how the beneficiary uses funds.",
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
      iconName: "extension-escrow",
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
      label: "Bounty",
      summary: "Propose a reward claim whose payout requires an explicit verifier decision.",
      createdArtifact: "A backed bounty definition with verifier, deadline, and evidence requirements.",
      purpose: "Offer a bounded reward whose payout remains dependent on a separate verifier decision and Wallet confirmation.",
      useCaseFamily: "verified_reward",
      maturity: "concept",
      iconName: "extension-bounties",
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
      label: "Ticket & Pass",
      summary: "Issue a private event, transport, membership, or access pass with bounded use.",
      createdArtifact: "A bounded private access-pass definition.",
      purpose: "Grant a declared event, transport, membership, or venue right with explicit uses, validity, transfer, and offline policy.",
      useCaseFamily: "private_access",
      iconName: "extension-tickets-passes",
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
      label: "Service Credit",
      summary: "Model bounded API, data, compute, or access rights as explicit service credits.",
      createdArtifact: "A bounded provider-specific service-credit definition.",
      purpose: "Issue metered service rights that are neither money nor proof that the external service was delivered.",
      useCaseFamily: "service_access",
      iconName: "extension-service-credits",
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
      summary: "Issue a bounded software licence, download entitlement, software add-on, or digital item right.",
      createdArtifact: "A bounded digital-entitlement definition.",
      purpose: "Grant the declared licence or item right without executing content or asserting successful external delivery.",
      useCaseFamily: "digital_entitlement",
      iconName: "extension-digital-goods",
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
      iconName: "extension-payroll",
      intentType: "prepare_private_payroll",
      requestedObjectFamilies: ["asset", "permission", "claim"],
      offlineMode: "batch_draft",
      offlineSummary: "A sanitized batch draft can be reviewed locally; recipients and totals are revalidated before every package.",
      disclosures: ["batch_total", "recipient_count", "schedule", "audit_scope"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension cannot access treasury keys, sign a batch, or expose the payroll graph publicly.",
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
      id: "private-contract",
      label: "Private Agreement",
      summary: "Propose a typed private agreement with explicit party, obligation, validity, disclosure, and decision bounds.",
      createdArtifact: "A typed private-agreement proposal with canonical terms and acceptance boundaries.",
      purpose: "Commit reviewed agreement terms without installing runtime code or granting the Extension legal, signing, adjudication, or value-transfer authority.",
      useCaseFamily: "private_agreement",
      maturity: "concept",
      iconName: "extension-private-contract",
      intentType: "propose_private_contract",
      requestedObjectFamilies: ["claim", "permission"],
      offlineMode: "agreement_draft",
      offlineSummary: "A terms digest and draft can be reviewed offline; both-party acceptance and every later action remain separate evidence stages.",
      disclosures: ["counterparty", "agreement_subject", "obligation_scope", "validity_window", "disclosure_policy"],
      reviewBoundary: "The Extension cannot bind a party, determine enforceability, interpret terms, adjudicate a dispute, or transfer value.",
      actionLabel: "Review private agreement in Wallet",
      proposalFields: [
        { id: "template", label: "Agreement template", type: "select", options: ["Service agreement", "Supply agreement", "Mutual confidentiality", "Licence terms", "Custom typed agreement"] },
        { id: "counterparty", label: "Counterparty", type: "text", placeholder: "Counterparty commitment", required: true },
        { id: "subject", label: "Agreement subject", type: "text", placeholder: "Exact private relationship or deliverable", required: true },
        { id: "obligations", label: "Bounded obligations", type: "text", placeholder: "Declared duties or deliverables", required: true },
        { id: "terms-digest", label: "Terms digest", type: "text", placeholder: "Canonical digest of the reviewed terms", required: true, suffix: "Never a filesystem path or runnable payload" },
        { id: "effective", label: "Effective rule", type: "select", options: ["After both parties accept", "At declared checkpoint", "At scheduled time"] },
        { id: "expiry", label: "Expiry", type: "select", options: ["No automatic expiry", "30 days", "90 days", "1 year"] },
        { id: "disclosure", label: "Disclosure", type: "select", options: ["Parties only", "Named verifier", "Selective proof"] },
        { id: "decision", label: "Decision path", type: "select", options: ["Mutual acceptance only", "Named mediator", "External authority"] }
      ],
      walletChecks: ["Party commitments", "Terms digest and obligations", "Validity and disclosure", "Decision authority"],
      settlementPath: "Agreement proposal → both-party acceptance → evidence receipt → optional scoped action",
      evidenceOutput: "Agreement digest, acceptance receipts, and any later scoped-action receipt"
    }),
    descriptor({
      id: "assets-locker",
      label: "Assets Locker",
      summary: "Represent a bounded private right over one explicit external custody route.",
      createdArtifact: "A typed external lock/import or consume/redeem route proposal.",
      purpose: "Represent one external custody right while preserving the operator, network, reserve, pause, and redemption trust boundary.",
      useCaseFamily: "external_custody_right",
      iconName: "extension-assets-locker",
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
    }),
    descriptor({
      id: "xchain-integration",
      label: "X-Chain Integration",
      summary: "Describe the result you want and let Wallet compare bounded solver plans without making you operate bridges, venues, or adapters.",
      createdArtifact: "A typed execution intent plus a request for normalized solver quotes, bound to one result, destination, deadline, cost ceiling, and fallback policy.",
      purpose: "Keep integration work under the hood while making the achievable result, execution method, required resources, total price, expected speed, finality, and recovery path reviewable.",
      useCaseFamily: "cross_chain_integration",
      maturity: "concept",
      iconName: "extension-xchain-integration",
      intentType: "prepare_xchain_integration",
      requestedObjectFamilies: ["asset", "claim", "service_right", "external_asset_right"],
      offlineMode: "reconnect_required",
      offlineSummary: "The intent, accepted quote, method, limits, and last execution receipts remain inspectable offline; fresh quotes, external state, and completion require reconnection.",
      disclosures: ["desired_result", "provided_value", "minimum_result", "destination", "execution_preference", "deadline", "maximum_total_cost", "fallback_policy"],
      valuePath: "wallet_review",
      feePath: "separate_wallet_review",
      reviewBoundary: "The Extension cannot choose Wallet inputs, silently switch a reviewed route, attest external evidence, exceed result, cost, time, or fallback limits, release custody, publish private payloads, or claim an external effect is final.",
      actionLabel: "Review execution intent in Wallet",
      proposalFields: [
        {
          id: "result-kind",
          label: "What you want to achieve",
          type: "select",
          options: [
            "Receive a private asset in Z00Z",
            "Deliver an asset to an external recipient",
            "Fulfill an external service",
            "Publish public evidence"
          ],
          suffix: "You choose the result. Wallet compares compatible solver and adapter plans."
        },
        {
          id: "source",
          label: "You provide",
          type: "text",
          placeholder: "Exact asset, amount, held right, or public artifact commitment",
          required: true,
          suffix: "Wallet resolves eligible private objects and external evidence; the Extension never selects Wallet inputs."
        },
        {
          id: "result",
          label: "Minimum acceptable result",
          type: "text",
          placeholder: "Exact asset and amount, service outcome, or publication handle",
          required: true,
          suffix: "Every quote must satisfy this result or Wallet rejects it."
        },
        {
          id: "destination",
          label: "Destination",
          type: "text",
          placeholder: "External recipient or service target when required",
          suffix: "For a Z00Z receive, Wallet binds the active receiver automatically."
        },
        {
          id: "preference",
          label: "Execution preference",
          type: "select",
          options: [
            "Best overall result",
            "Lowest total cost",
            "Fastest expected completion",
            "Fewest external trust dependencies"
          ],
          suffix: "This ranks valid quotes; it never weakens the result, deadline, or cost ceiling."
        },
        {
          id: "deadline",
          label: "Execution deadline",
          type: "select",
          options: [
            "15 minutes",
            "1 hour",
            "24 hours"
          ],
          suffix: "Wallet rejects quotes whose expected completion window exceeds this deadline."
        },
        {
          id: "max-cost",
          label: "Maximum total cost",
          type: "number",
          placeholder: "0.00",
          required: true,
          min: "0",
          suffix: "One ceiling for solver, protocol, network, price-impact, and recovery costs."
        },
        {
          id: "fallback",
          label: "Fallback policy",
          type: "select",
          options: [
            "Ask before any route change",
            "Allow an equivalent fallback within all reviewed limits"
          ],
          suffix: "No fallback may change the result, destination, deadline, total-cost ceiling, or add a trust dependency."
        }
      ],
      walletChecks: [
        "Exact source, destination, and minimum result",
        "Comparable signed quote schema and solver identity",
        "Disclosed protocols, adapters, resources, and trust dependencies",
        "All-in cost under the reviewed ceiling",
        "Quote expiry, expected completion window, and finality milestones",
        "Fallback, cancellation, refund, and irreversible-step boundaries",
        "Wallet object selection and final user confirmation"
      ],
      settlementPath: "Result intent → competing solver quotes → normalized plan → Wallet confirmation → staged execution → verified result or bounded recovery",
      evidenceOutput: "Accepted quote, solver and method, protocol steps, resources, all-in cost, time and finality milestones, execution receipts, and recovery outcome",
      integrations: [
        {
          id: "near-intents",
          label: "NEAR Intents",
          role: "Intent discovery and solver competition",
          method: "Collect signed result quotes and normalize the selected solver plan into the Wallet execution schema.",
          resources: "Solver liquidity, route adapters, source funding, deposit evidence, and delivery verification.",
          cost: "Solver margin, adapter, venue, network, and recovery costs under one total ceiling.",
          speed: "Quote expiry, funding window, expected execution, delivery, and external finality milestones.",
          trustBoundary: "Selected solver plus every adapter, venue, custody, and network dependency in its signed plan."
        },
        {
          id: "ethereum",
          label: "Ethereum",
          role: "External settlement and evidence verification",
          method: "Execute reviewed EVM steps such as an EVM Locker or asset-specific Issuer Rail and verify finalized event evidence.",
          resources: "Contract capacity, gas, allowance or custody state, RPC evidence, and finality monitoring.",
          cost: "Network gas, contract or adapter charges, and recovery reserve included in the all-in quote.",
          speed: "Submission, inclusion, confirmation, and route-specific finality windows.",
          trustBoundary: "Reviewed contracts, operators where present, evidence providers, and Ethereum finality assumptions."
        },
        {
          id: "liquity-bold",
          label: "Liquity BOLD",
          role: "Stable liquidity source",
          method: "Use a route-specific BOLD liquidity and settlement plan without treating Liquity as Z00Z finality.",
          resources: "Available BOLD liquidity, redemption or market path, collateral constraints, and Ethereum gas.",
          cost: "Spread, liquidity impact, protocol or solver charges, network costs, and recovery allowance.",
          speed: "Liquidity availability, route execution, Ethereum inclusion, and external finality windows.",
          trustBoundary: "Liquity protocol state plus the selected market, custody, solver, and adapter dependencies."
        },
        {
          id: "hyperliquid",
          label: "Hyperliquid",
          role: "Order-book execution venue",
          method: "Execute a bounded market or limit plan with explicit price, slippage, fill, and withdrawal conditions.",
          resources: "Venue balance or deposit, market depth, venue adapter, withdrawal path, and delivery evidence.",
          cost: "Spread, slippage, trading, deposit or withdrawal, network, solver, and recovery costs.",
          speed: "Deposit readiness, expected fill, withdrawal, delivery, and finality windows.",
          trustBoundary: "Venue operation, selected adapter, custody exposure, market liquidity, and withdrawal availability."
        },
        {
          id: "uniswap",
          label: "Uniswap",
          role: "On-chain AMM execution",
          method: "Execute a bounded swap against disclosed pools with a minimum output, slippage limit, and deadline.",
          resources: "Pool liquidity, token allowance or permit, route contracts, gas, and output-delivery evidence.",
          cost: "Pool fee, price impact, network gas, solver or adapter charge, and recovery allowance.",
          speed: "Quote expiry, transaction inclusion, confirmation, delivery, and external finality windows.",
          trustBoundary: "Selected pools, router contracts, token behavior, adapter logic, and network finality."
        },
        {
          id: "external-solvers",
          label: "External Solvers",
          role: "Competitive and fallback execution",
          method: "Accept only signed plans that implement the same result, cost, timing, trust, evidence, and recovery schema. Any EVM Locker, Issuer Rail, or Celestia DA publication is a disclosed plan step, never a separate user choice.",
          resources: "Solver identity, liquidity or bond, required adapters, source funding, monitoring, and refund capacity.",
          cost: "Every solver, protocol, venue, network, price-impact, and recovery component in one comparable quote.",
          speed: "Quote expiry, start deadline, expected completion, finality milestones, and refund deadline.",
          trustBoundary: "Solver identity and bond or reputation plus every dependency explicitly named by the plan."
        }
      ],
      quoteFields: [
        "Achievable result — exact output, minimum amount or service outcome, and destination.",
        "Execution method — selected solver, protocols, external steps, and trust dependencies.",
        "Required resources — source input, liquidity, gas, custody, allowance, collateral, or public artifact.",
        "Total price — solver, protocol, venue, network, price-impact, and recovery costs under one ceiling.",
        "Expected speed — quote expiry, start, completion, delivery, and finality milestones.",
        "Failure path — cancellation deadline, refund owner, irreversible point, and permitted fallback."
      ],
      integrationRecommendation: "Start from the user's result and compare only plans that meet the same minimum result, destination, deadline, total-cost ceiling, and ranked preference. Keep discovery, simulation, evidence collection, monitoring, and adapter selection automatic. Before confirmation, expose the selected method, resources, all-in price, expected speed, trust dependencies, finality points, irreversible step, and recovery path.",
      integrationInvariants: [
        "Reject every plan below the minimum result or outside the reviewed destination, deadline, total-cost ceiling, or fallback policy.",
        "Normalize every solver quote into the same result, method, resources, price, timing, finality, and recovery schema.",
        "Disclose every protocol, adapter, venue, custody, issuer, and finality dependency before confirmation.",
        "Wallet selects eligible inputs, verifies external evidence, builds the package, and obtains final user confirmation.",
        "Never switch routes silently outside the reviewed fallback policy or after an irreversible step.",
        "Track Z00Z checkpoint finality and every external effect or publication finality as separate statuses.",
        "Fail closed when evidence is stale, contradictory, unavailable, or insufficient; surface cancellation, refund, or manual recovery."
      ]
    })
  ]);

  const EXTENSION_CONNECTION_FIXTURES = deepFreeze([
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

  const EXTENSION_PERMISSION_FIXTURES = deepFreeze([
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

  const EXTENSION_ACTIVITY_FIXTURES = deepFreeze([
    {
      id: "extension_event_0004",
      descriptorId: "agents-budget",
      kind: "intent_accepted",
      outcome: "Accepted for Wallet review",
      occurredAt: "2026-07-26T08:42:00Z",
      summary: "A bounded expense proposal passed app-level review; settlement was not implied."
    },
    {
      id: "extension_event_0003",
      descriptorId: "agents-budget",
      kind: "intent_rejected",
      outcome: "Rejected",
      occurredAt: "2026-07-26T07:10:00Z",
      summary: "A concept proposal was rejected before any wallet operation was created."
    },
    {
      id: "extension_event_0002",
      descriptorId: "service-credits",
      kind: "permission_expired",
      outcome: "Expired",
      occurredAt: "2026-07-25T08:20:00Z",
      summary: "The bounded service-credit presentation grant reached its declared expiry."
    },
    {
      id: "extension_event_0001",
      descriptorId: "create-voucher",
      kind: "permission_reviewed",
      outcome: "Approved locally",
      occurredAt: "2026-07-24T16:05:00Z",
      summary: "A claim-inspection scope was approved without issuing or redeeming a Voucher."
    }
  ]);

  const EXTENSION_DESCRIPTOR_IDS = deepFreeze(EXTENSION_CATALOG.map(({ id }) => id));
  const EXTENSION_DESCRIPTOR_LUT = deepFreeze(Object.fromEntries(
    EXTENSION_CATALOG.map((entry) => [entry.id, entry])
  ));

  function extensionDescriptor(descriptorId) {
    return EXTENSION_DESCRIPTOR_LUT[descriptorId] || null;
  }

  function assertExtensionCatalog() {
    const validMaturity = new Set(demo.PORT_CONTRACT.maturity);
    const validAvailability = new Set(demo.PORT_CONTRACT.availability);
    const validEvidence = new Set(demo.PORT_CONTRACT.evidenceSources);
    const validFreshness = new Set(demo.PORT_CONTRACT.freshness);
    const validPresentation = new Set(demo.PORT_CONTRACT.presentationModes);
    const seenIds = new Set();
    const seenIntentTypes = new Set();
    const seenIcons = new Set();

    if (EXTENSION_CATALOG.length !== 18) {
      throw new Error("The local extension catalogue must contain exactly eighteen descriptors.");
    }

    for (const entry of EXTENSION_CATALOG) {
      if (seenIds.has(entry.id)) throw new Error(`Duplicate extension descriptor ID: ${entry.id}`);
      if (seenIntentTypes.has(entry.intentType)) throw new Error(`Duplicate extension intent type: ${entry.intentType}`);
      if (seenIcons.has(entry.iconName)) throw new Error(`Duplicate extension icon: ${entry.iconName}`);
      seenIds.add(entry.id);
      seenIntentTypes.add(entry.intentType);
      seenIcons.add(entry.iconName);

      if (!validMaturity.has(entry.maturity)
        || !validAvailability.has(entry.availability)
        || !validEvidence.has(entry.evidenceSource)
        || !validFreshness.has(entry.freshness)
        || !validPresentation.has(entry.presentationMode)) {
        throw new Error(`Invalid capability axes for extension descriptor: ${entry.id}`);
      }
      if (!EXTENSION_INTENT_TYPES.includes(entry.intentType)) {
        throw new Error(`Unknown extension intent type: ${entry.intentType}`);
      }
      if (!entry.requestedObjectFamilies.length
        || entry.requestedObjectFamilies.some((family) => !EXTENSION_OBJECT_FAMILIES.includes(family))) {
        throw new Error(`Invalid object family for extension descriptor: ${entry.id}`);
      }
      if (!entry.createdArtifact?.trim() || !entry.purpose?.trim()) {
        throw new Error(`Missing creation or purpose explanation for extension descriptor: ${entry.id}`);
      }
      if (!entry.proposalFields.length || entry.proposalFields.some((field) => !field.id || !field.label || !field.type)) {
        throw new Error(`Invalid proposal field for extension descriptor: ${entry.id}`);
      }
      if (entry.proposalFields.some((field) => field.type === "number" && Number(field.min) < 0)) {
        throw new Error(`Negative numeric bound for extension descriptor: ${entry.id}`);
      }
      if (entry.publisher.verified
        || entry.availability !== "unavailable"
        || entry.presentationMode !== "roadmap_preview"
        || entry.executionBoundary !== "typed_intent_only"
        || entry.remoteCodeAllowed
        || entry.walletBridgeAllowed) {
        throw new Error(`Unsafe extension execution claim: ${entry.id}`);
      }
    }

    for (const entry of [...EXTENSION_CONNECTION_FIXTURES, ...EXTENSION_PERMISSION_FIXTURES, ...EXTENSION_ACTIVITY_FIXTURES]) {
      if (!EXTENSION_DESCRIPTOR_LUT[entry.descriptorId]) {
        throw new Error(`Unknown extension fixture descriptor: ${entry.descriptorId}`);
      }
    }

    if (/((?:https?:)?\/\/)|\b(?:url|domain|iframe|bundle|executable|sourceCode)\b/i.test(JSON.stringify(EXTENSION_CATALOG))) {
      throw new Error("The local extension catalogue must not contain remote or executable application descriptors.");
    }
    return true;
  }

  assertExtensionCatalog();

  Object.assign(root.Z00ZDemo, {
    EXTENSION_OBJECT_FAMILIES,
    EXTENSION_INTENT_TYPES,
    EXTENSION_CATALOG,
    EXTENSION_DESCRIPTOR_IDS,
    EXTENSION_DESCRIPTOR_LUT,
    EXTENSION_CONNECTION_FIXTURES,
    EXTENSION_PERMISSION_FIXTURES,
    EXTENSION_ACTIVITY_FIXTURES,
    extensionDescriptor,
    assertExtensionCatalog
  });
})(typeof window === "undefined" ? globalThis : window);
