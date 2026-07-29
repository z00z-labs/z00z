"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z demo contracts must load before the navigation model.");
  }

  const freeze = (value) => Object.freeze(value);
  const target = (kind, value = {}) => freeze({ kind, ...value });
  const branchTarget = () => target("branch");
  const groupTarget = () => target("group");
  const workspaceTarget = (
    routeId,
    defaultLabelKey = "navigation.overview",
    defaultIconId = "overview"
  ) => target("workspace", { routeId, defaultLabelKey, defaultIconId });
  const routeTarget = (routeId) => target("route", { routeId });
  const helpTarget = (helpRouteId) => target("help", { helpRouteId });
  const actionTarget = (actionId) => target("action", { actionId });
  const node = ({
    id,
    parentId = null,
    order,
    labelKey,
    iconId,
    target: nodeTarget,
    capabilityId = null,
    presentationMode = "product",
    helpTopicId = null,
    isVisible = true,
    sectionBreakBefore = false
  }) => freeze({
    id,
    parentId,
    order,
    labelKey,
    iconId,
    iconTone: "neutral",
    target: nodeTarget,
    capabilityId,
    presentationMode,
    helpTopicId,
    isVisible,
    sectionBreakBefore
  });
  const profile = ({
    id,
    maturity,
    availability,
    evidenceSource,
    freshness,
    presentationMode
  }) => freeze({
    id,
    maturity,
    availability,
    evidenceSource,
    freshness,
    presentationMode
  });

  const CAPABILITY_PROFILES = freeze([
    profile({ id: "wallet.quarantine", maturity: "target", availability: "unavailable", evidenceSource: "none", freshness: "not_applicable", presentationMode: "product" }),
    profile({ id: "wallet.swap", maturity: "live", availability: "unavailable", evidenceSource: "fixture", freshness: "not_applicable", presentationMode: "product" }),
    profile({ id: "wallet.staking", maturity: "live", availability: "unavailable", evidenceSource: "fixture", freshness: "not_applicable", presentationMode: "product" }),
    profile({ id: "telemetry.reticulum", maturity: "live", availability: "unavailable", evidenceSource: "none", freshness: "unknown", presentationMode: "product" }),
    profile({ id: "telemetry.onionnet", maturity: "target", availability: "unavailable", evidenceSource: "none", freshness: "unknown", presentationMode: "product" }),
    profile({ id: "telemetry.quic", maturity: "target", availability: "unavailable", evidenceSource: "none", freshness: "unknown", presentationMode: "product" }),
    profile({ id: "telemetry.aggregators", maturity: "live", availability: "unavailable", evidenceSource: "none", freshness: "unknown", presentationMode: "product" }),
    profile({ id: "telemetry.watchers", maturity: "live", availability: "unavailable", evidenceSource: "fixture", freshness: "unknown", presentationMode: "roadmap_preview" }),
    profile({ id: "telemetry.explorer", maturity: "target", availability: "unavailable", evidenceSource: "fixture", freshness: "unknown", presentationMode: "roadmap_preview" }),
    profile({ id: "dapps", maturity: "concept", availability: "unavailable", evidenceSource: "fixture", freshness: "not_applicable", presentationMode: "roadmap_preview" }),
    profile({ id: "messenger", maturity: "target", availability: "unavailable", evidenceSource: "fixture", freshness: "not_applicable", presentationMode: "roadmap_preview" }),
    profile({ id: "contacts", maturity: "concept", availability: "unavailable", evidenceSource: "fixture", freshness: "not_applicable", presentationMode: "product" })
  ]);

  const NAVIGATION_NODES = freeze([
    node({ id: "wallet", order: 10, labelKey: "navigation.wallet", iconId: "wallet", target: branchTarget(), helpTopicId: "wallet.assets" }),
    node({ id: "wallet.assets-rights", parentId: "wallet", order: 20, labelKey: "navigation.assets", iconId: "assets", target: workspaceTarget("wallet.assets", "navigation.assets", "assets"), helpTopicId: "wallet.assets" }),
    node({ id: "wallet.vouchers", parentId: "wallet.assets-rights", order: 20, labelKey: "navigation.vouchers", iconId: "voucher-list", target: routeTarget("wallet.vouchers"), helpTopicId: "wallet.vouchers" }),
    node({ id: "wallet.permissions", parentId: "wallet.assets-rights", order: 30, labelKey: "navigation.permissions", iconId: "permission-list", target: routeTarget("wallet.permissions"), helpTopicId: "wallet.permissions" }),
    node({ id: "wallet.quarantine", parentId: "wallet.assets-rights", order: 40, labelKey: "navigation.quarantine", iconId: "alert", target: routeTarget("wallet.quarantine"), capabilityId: "wallet.quarantine", helpTopicId: "wallet.quarantine", isVisible: false }),
    node({ id: "wallet.send", parentId: "wallet", order: 60, labelKey: "navigation.send", iconId: "send", target: routeTarget("wallet.send"), helpTopicId: "wallet.send" }),
    node({ id: "wallet.receive", parentId: "wallet", order: 70, labelKey: "navigation.receive", iconId: "receive", target: routeTarget("wallet.receive"), helpTopicId: "wallet.receive" }),
    node({ id: "wallet.import", parentId: "wallet", order: 80, labelKey: "navigation.import", iconId: "import", target: routeTarget("wallet.import"), helpTopicId: "wallet.import" }),
    node({ id: "wallet.merge-split", parentId: "wallet", order: 85, labelKey: "navigation.mergeSplit", iconId: "merge-split", target: routeTarget("wallet.merge-split"), helpTopicId: "wallet.merge-split" }),
    node({ id: "wallet.history", parentId: "wallet", order: 90, labelKey: "navigation.history", iconId: "activity", target: routeTarget("wallet.history"), helpTopicId: "wallet.history" }),
    node({ id: "wallet.staking", parentId: "wallet", order: 110, labelKey: "navigation.earn", iconId: "earn", target: workspaceTarget("wallet.staking.stake", "navigation.stake", "earn"), capabilityId: "wallet.staking", helpTopicId: "wallet.staking.stake" }),
    node({ id: "wallet.staking.unstake", parentId: "wallet.staking", order: 20, labelKey: "navigation.unstake", iconId: "restore", target: routeTarget("wallet.staking.unstake"), capabilityId: "wallet.staking", helpTopicId: "wallet.staking.unstake" }),
    node({ id: "wallet.backup", parentId: "wallet", order: 120, labelKey: "navigation.backup", iconId: "backup", target: routeTarget("wallet.backup"), helpTopicId: "wallet.backup" }),
    node({ id: "wallet.settings", parentId: "wallet", order: 130, labelKey: "navigation.walletSettings", iconId: "settings", target: workspaceTarget("wallet.settings.general", "navigation.general", "overview"), helpTopicId: "wallet.settings.general" }),
    node({ id: "wallet.settings.security", parentId: "wallet.settings", order: 20, labelKey: "navigation.security", iconId: "shield", target: routeTarget("wallet.settings.security"), helpTopicId: "wallet.settings.security" }),
    node({ id: "wallet.settings.backup", parentId: "wallet.settings", order: 30, labelKey: "navigation.backup", iconId: "backup", target: routeTarget("wallet.settings.backup"), helpTopicId: "wallet.settings.backup" }),
    node({ id: "wallet.settings.policies", parentId: "wallet.settings", order: 40, labelKey: "navigation.policies", iconId: "check", target: routeTarget("wallet.settings.policies"), helpTopicId: "wallet.settings.policies" }),
    node({ id: "wallet.settings.advanced", parentId: "wallet.settings", order: 50, labelKey: "navigation.advanced", iconId: "advanced", target: routeTarget("wallet.settings.advanced"), helpTopicId: "wallet.settings.advanced" }),

    node({ id: "telemetry", order: 51, labelKey: "navigation.telemetry", iconId: "network", target: branchTarget(), helpTopicId: "telemetry.reticulum.overview" }),
    node({ id: "telemetry.reticulum", parentId: "telemetry", order: 10, labelKey: "navigation.reticulum", iconId: "reticulum-node", target: workspaceTarget("telemetry.reticulum.overview"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.overview" }),
    node({ id: "telemetry.reticulum.node", parentId: "telemetry.reticulum", order: 20, labelKey: "navigation.node", iconId: "reticulum-node", target: routeTarget("telemetry.reticulum.node"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.node" }),
    node({ id: "telemetry.reticulum.interfaces", parentId: "telemetry.reticulum", order: 30, labelKey: "navigation.interfaces", iconId: "reticulum-interface", target: routeTarget("telemetry.reticulum.interfaces"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.interfaces" }),
    node({ id: "telemetry.reticulum.radio", parentId: "telemetry.reticulum", order: 40, labelKey: "navigation.radio", iconId: "network", target: routeTarget("telemetry.reticulum.radio"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.radio" }),
    node({ id: "telemetry.reticulum.entrypoints", parentId: "telemetry.reticulum", order: 50, labelKey: "navigation.entrypoints", iconId: "entry", target: routeTarget("telemetry.reticulum.entrypoints"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.entrypoints" }),
    node({ id: "telemetry.reticulum.paths", parentId: "telemetry.reticulum", order: 60, labelKey: "navigation.paths", iconId: "reticulum-paths", target: routeTarget("telemetry.reticulum.paths"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.paths" }),
    node({ id: "telemetry.reticulum.probes", parentId: "telemetry.reticulum", order: 70, labelKey: "navigation.probes", iconId: "probe", target: routeTarget("telemetry.reticulum.probes"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.probes" }),
    node({ id: "telemetry.reticulum.links", parentId: "telemetry.reticulum", order: 80, labelKey: "navigation.links", iconId: "reticulum-link", target: routeTarget("telemetry.reticulum.links"), capabilityId: "telemetry.reticulum", helpTopicId: "telemetry.reticulum.links" }),
    node({ id: "telemetry.onionnet", parentId: "telemetry", order: 20, labelKey: "navigation.onionnet", iconId: "shield", target: workspaceTarget("telemetry.onionnet.overview"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.overview" }),
    node({ id: "telemetry.onionnet.epoch", parentId: "telemetry.onionnet", order: 20, labelKey: "navigation.epoch", iconId: "activity", target: routeTarget("telemetry.onionnet.epoch"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.epoch" }),
    node({ id: "telemetry.onionnet.privacy", parentId: "telemetry.onionnet", order: 30, labelKey: "navigation.privacy", iconId: "shield", target: routeTarget("telemetry.onionnet.privacy"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.privacy" }),
    node({ id: "telemetry.onionnet.transport", parentId: "telemetry.onionnet", order: 40, labelKey: "navigation.transport", iconId: "network", target: routeTarget("telemetry.onionnet.transport"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.transport" }),
    node({ id: "telemetry.onionnet.queues", parentId: "telemetry.onionnet", order: 50, labelKey: "navigation.queues", iconId: "queue", target: routeTarget("telemetry.onionnet.queues"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.queues" }),
    node({ id: "telemetry.onionnet.probation", parentId: "telemetry.onionnet", order: 60, labelKey: "navigation.probation", iconId: "alert", target: routeTarget("telemetry.onionnet.probation"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.probation" }),
    node({ id: "telemetry.onionnet.ingress", parentId: "telemetry.onionnet", order: 70, labelKey: "navigation.ingress", iconId: "entry", target: routeTarget("telemetry.onionnet.ingress"), capabilityId: "telemetry.onionnet", helpTopicId: "telemetry.onionnet.ingress" }),
    node({ id: "telemetry.quic", parentId: "telemetry", order: 30, labelKey: "navigation.quic", iconId: "network", target: workspaceTarget("telemetry.quic.overview"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.overview" }),
    node({ id: "telemetry.quic.connections", parentId: "telemetry.quic", order: 20, labelKey: "quic.tabs.connections", iconId: "reticulum-link", target: routeTarget("telemetry.quic.connections"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.connections" }),
    node({ id: "telemetry.quic.paths", parentId: "telemetry.quic", order: 30, labelKey: "quic.tabs.paths", iconId: "reticulum-paths", target: routeTarget("telemetry.quic.paths"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.paths" }),
    node({ id: "telemetry.quic.streams", parentId: "telemetry.quic", order: 40, labelKey: "quic.tabs.streams", iconId: "queue", target: routeTarget("telemetry.quic.streams"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.streams" }),
    node({ id: "telemetry.quic.recovery", parentId: "telemetry.quic", order: 50, labelKey: "quic.tabs.recovery", iconId: "restore", target: routeTarget("telemetry.quic.recovery"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.recovery" }),
    node({ id: "telemetry.quic.security", parentId: "telemetry.quic", order: 60, labelKey: "quic.tabs.security", iconId: "shield", target: routeTarget("telemetry.quic.security"), capabilityId: "telemetry.quic", helpTopicId: "telemetry.quic.security" }),
    node({ id: "telemetry.aggregators", parentId: "telemetry", order: 40, labelKey: "navigation.aggregators", iconId: "aggregate", target: workspaceTarget("telemetry.aggregators.overview"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.overview" }),
    node({ id: "telemetry.aggregators.ingress", parentId: "telemetry.aggregators", order: 20, labelKey: "navigation.ingress", iconId: "entry", target: routeTarget("telemetry.aggregators.ingress"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.ingress" }),
    node({ id: "telemetry.aggregators.planning", parentId: "telemetry.aggregators", order: 30, labelKey: "navigation.planning", iconId: "advanced", target: routeTarget("telemetry.aggregators.planning"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.planning" }),
    node({ id: "telemetry.aggregators.placement", parentId: "telemetry.aggregators", order: 40, labelKey: "navigation.placement", iconId: "reticulum-interface", target: routeTarget("telemetry.aggregators.placement"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.placement" }),
    node({ id: "telemetry.aggregators.publication", parentId: "telemetry.aggregators", order: 50, labelKey: "navigation.publication", iconId: "send", target: routeTarget("telemetry.aggregators.publication"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.publication" }),
    node({ id: "telemetry.aggregators.recovery", parentId: "telemetry.aggregators", order: 60, labelKey: "navigation.recovery", iconId: "restore", target: routeTarget("telemetry.aggregators.recovery"), capabilityId: "telemetry.aggregators", helpTopicId: "telemetry.aggregators.recovery" }),
    node({ id: "telemetry.watchers", parentId: "telemetry", order: 50, labelKey: "navigation.watchers", iconId: "eye", target: workspaceTarget("telemetry.watchers.overview"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.overview" }),
    node({ id: "telemetry.watchers.alerts", parentId: "telemetry.watchers", order: 20, labelKey: "navigation.alerts", iconId: "alert", target: routeTarget("telemetry.watchers.alerts"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.alerts" }),
    node({ id: "telemetry.watchers.publication", parentId: "telemetry.watchers", order: 30, labelKey: "navigation.publicationChecks", iconId: "check", target: routeTarget("telemetry.watchers.publication"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.publication" }),
    node({ id: "telemetry.watchers.providers", parentId: "telemetry.watchers", order: 40, labelKey: "navigation.daProviders", iconId: "network", target: routeTarget("telemetry.watchers.providers"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.providers" }),
    node({ id: "telemetry.watchers.censorship", parentId: "telemetry.watchers", order: 50, labelKey: "navigation.censorship", iconId: "eye-off", target: routeTarget("telemetry.watchers.censorship"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.censorship" }),
    node({ id: "telemetry.watchers.evidence", parentId: "telemetry.watchers", order: 60, labelKey: "navigation.evidenceExport", iconId: "backup", target: routeTarget("telemetry.watchers.evidence"), capabilityId: "telemetry.watchers", presentationMode: "roadmap_preview", helpTopicId: "telemetry.watchers.evidence" }),
    node({ id: "telemetry.explorer", parentId: "telemetry", order: 60, labelKey: "navigation.explorer", iconId: "search", target: workspaceTarget("telemetry.explorer.overview"), capabilityId: "telemetry.explorer", presentationMode: "roadmap_preview", helpTopicId: "telemetry.explorer.overview" }),
    node({ id: "telemetry.explorer.search", parentId: "telemetry.explorer", order: 20, labelKey: "navigation.search", iconId: "search", target: routeTarget("telemetry.explorer.search"), capabilityId: "telemetry.explorer", presentationMode: "roadmap_preview", helpTopicId: "telemetry.explorer.search" }),
    node({ id: "telemetry.explorer.checkpoints", parentId: "telemetry.explorer", order: 30, labelKey: "navigation.checkpoints", iconId: "check", target: routeTarget("telemetry.explorer.checkpoints"), capabilityId: "telemetry.explorer", presentationMode: "roadmap_preview", helpTopicId: "telemetry.explorer.checkpoints" }),
    node({ id: "telemetry.explorer.batches", parentId: "telemetry.explorer", order: 40, labelKey: "navigation.batches", iconId: "queue", target: routeTarget("telemetry.explorer.batches"), capabilityId: "telemetry.explorer", presentationMode: "roadmap_preview", helpTopicId: "telemetry.explorer.batches" }),
    node({ id: "telemetry.explorer.evidence", parentId: "telemetry.explorer", order: 50, labelKey: "navigation.publicEvidence", iconId: "copy", target: routeTarget("telemetry.explorer.evidence"), capabilityId: "telemetry.explorer", presentationMode: "roadmap_preview", helpTopicId: "telemetry.explorer.evidence" }),

    node({ id: "dapps", order: 30, labelKey: "navigation.dapps", iconId: "spark", target: branchTarget(), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.discover" }),
    node({ id: "dapps.agents-budget", parentId: "dapps", order: 10, labelKey: "navigation.agentsBudget", iconId: "dapp-agents-budget", target: routeTarget("dapps.agents-budget"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.agents-budget" }),
    node({ id: "dapps.assets-locker", parentId: "dapps", order: 20, labelKey: "navigation.assetsLocker", iconId: "dapp-assets-locker", target: routeTarget("dapps.assets-locker"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.assets-locker" }),
    node({ id: "dapps.bounties", parentId: "dapps", order: 30, labelKey: "navigation.bounties", iconId: "dapp-bounties", target: routeTarget("dapps.bounties"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.bounties" }),
    node({ id: "dapps.create-asset", parentId: "dapps", order: 40, labelKey: "navigation.createAsset", iconId: "assets", target: routeTarget("dapps.create-asset"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.create-asset" }),
    node({ id: "dapps.create-permission", parentId: "dapps", order: 50, labelKey: "navigation.createPermission", iconId: "permission-list", target: routeTarget("dapps.create-permission"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.create-permission" }),
    node({ id: "dapps.create-voucher", parentId: "dapps", order: 60, labelKey: "navigation.createVoucher", iconId: "voucher-list", target: routeTarget("dapps.create-voucher"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.create-voucher" }),
    node({ id: "dapps.digital-goods", parentId: "dapps", order: 70, labelKey: "navigation.digitalGoods", iconId: "dapp-digital-goods", target: routeTarget("dapps.digital-goods"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.digital-goods" }),
    node({ id: "dapps.donation", parentId: "dapps", order: 80, labelKey: "navigation.donation", iconId: "dapp-donation", target: routeTarget("dapps.donation"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.donation" }),
    node({ id: "dapps.escrow", parentId: "dapps", order: 90, labelKey: "navigation.escrow", iconId: "dapp-escrow", target: routeTarget("dapps.escrow"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.escrow" }),
    node({ id: "dapps.pay", parentId: "dapps", order: 100, labelKey: "navigation.pay", iconId: "dapp-pay", target: routeTarget("dapps.pay"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.pay" }),
    node({ id: "dapps.payroll", parentId: "dapps", order: 110, labelKey: "navigation.payroll", iconId: "dapp-payroll", target: routeTarget("dapps.payroll"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.payroll" }),
    node({ id: "dapps.private-contract", parentId: "dapps", order: 120, labelKey: "navigation.privateContract", iconId: "dapp-private-contract", target: routeTarget("dapps.private-contract"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.private-contract" }),
    node({ id: "dapps.request", parentId: "dapps", order: 130, labelKey: "navigation.request", iconId: "dapp-request", target: routeTarget("dapps.request"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.request" }),
    node({ id: "dapps.service-credits", parentId: "dapps", order: 140, labelKey: "navigation.serviceCredits", iconId: "dapp-service-credits", target: routeTarget("dapps.service-credits"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.service-credits" }),
    node({ id: "dapps.subscription", parentId: "dapps", order: 150, labelKey: "navigation.subscription", iconId: "dapp-subscription", target: routeTarget("dapps.subscription"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.subscription" }),
    node({ id: "wallet.swap", parentId: "dapps", order: 160, labelKey: "navigation.swap", iconId: "swap", target: routeTarget("wallet.swap"), capabilityId: "wallet.swap", helpTopicId: "wallet.swap" }),
    node({ id: "dapps.tickets-passes", parentId: "dapps", order: 170, labelKey: "navigation.ticketsPasses", iconId: "dapp-tickets-passes", target: routeTarget("dapps.tickets-passes"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.tickets-passes" }),
    node({ id: "dapps.wbold-gateway", parentId: "dapps", order: 180, labelKey: "navigation.wboldGateway", iconId: "dapp-wbold-gateway", target: routeTarget("dapps.wbold-gateway"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.wbold-gateway" }),
    node({ id: "dapps.xchain-integration", parentId: "dapps", order: 190, labelKey: "navigation.xchainIntegration", iconId: "dapp-xchain-integration", target: routeTarget("dapps.xchain-integration"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.xchain-integration" }),
    node({ id: "dapps.discover", parentId: "dapps", order: 200, labelKey: "navigation.discover", iconId: "search", target: routeTarget("dapps.discover"), capabilityId: "dapps", presentationMode: "roadmap_preview", helpTopicId: "dapps.discover", sectionBreakBefore: true }),
    node({ id: "messenger", order: 40, labelKey: "navigation.messenger", iconId: "message", target: branchTarget(), capabilityId: "messenger", presentationMode: "roadmap_preview", helpTopicId: "messenger.inbox" }),
    node({ id: "messenger.inbox", parentId: "messenger", order: 10, labelKey: "navigation.inbox", iconId: "inbox", target: routeTarget("messenger.inbox"), capabilityId: "messenger", presentationMode: "roadmap_preview", helpTopicId: "messenger.inbox" }),
    node({ id: "messenger.sent", parentId: "messenger", order: 20, labelKey: "navigation.sent", iconId: "sent", target: routeTarget("messenger.sent"), capabilityId: "messenger", presentationMode: "roadmap_preview", helpTopicId: "messenger.sent" }),
    node({ id: "messenger.conversations", parentId: "messenger", order: 30, labelKey: "navigation.conversations", iconId: "message", target: routeTarget("messenger.conversations"), capabilityId: "messenger", presentationMode: "roadmap_preview", helpTopicId: "messenger.conversations" }),
    node({ id: "contacts.list", order: 55, labelKey: "navigation.contacts", iconId: "user", target: routeTarget("contacts.list"), capabilityId: "contacts", helpTopicId: "contacts.list" }),

    node({ id: "data-storage", order: 50, labelKey: "navigation.dataStorage", iconId: "storage", target: branchTarget(), helpTopicId: "data-storage.disk-usage" }),
    node({ id: "data-storage.disk-usage", parentId: "data-storage", order: 10, labelKey: "navigation.diskUsage", iconId: "bar-chart", target: routeTarget("data-storage.disk-usage"), helpTopicId: "data-storage.disk-usage" }),
    node({ id: "data-storage.network-usage", parentId: "data-storage", order: 20, labelKey: "navigation.networkUsage", iconId: "line-chart", target: routeTarget("data-storage.network-usage"), helpTopicId: "data-storage.network-usage" }),

    node({ id: "settings", order: 60, labelKey: "navigation.settings", iconId: "settings", target: branchTarget(), helpTopicId: "settings.general" }),
    node({ id: "settings.general", parentId: "settings", order: 10, labelKey: "navigation.general", iconId: "overview", target: routeTarget("settings.general"), helpTopicId: "settings.general" }),
    node({ id: "settings.network", parentId: "settings", order: 20, labelKey: "navigation.network", iconId: "network", target: workspaceTarget("settings.reticulum", "navigation.reticulum", "reticulum-node"), helpTopicId: "settings.reticulum" }),
    node({ id: "settings.onionnet", parentId: "settings.network", order: 20, labelKey: "navigation.onionnet", iconId: "shield", target: routeTarget("settings.onionnet"), helpTopicId: "settings.onionnet", isVisible: false }),
    node({ id: "settings.quic", parentId: "settings.network", order: 30, labelKey: "navigation.quic", iconId: "network", target: routeTarget("settings.quic"), helpTopicId: "settings.quic", isVisible: false }),
    node({ id: "settings.notifications", parentId: "settings", order: 30, labelKey: "navigation.notifications", iconId: "bell", target: routeTarget("settings.notifications"), helpTopicId: "settings.notifications" }),
    node({ id: "settings.appearance", parentId: "settings", order: 40, labelKey: "navigation.appearance", iconId: "eye", target: routeTarget("settings.appearance"), helpTopicId: "settings.appearance" }),
    node({ id: "help", order: 70, labelKey: "navigation.help", iconId: "question", target: helpTarget("help.root"), helpTopicId: "app" }),
    node({ id: "about", order: 75, labelKey: "navigation.about", iconId: "info", target: routeTarget("about"), helpTopicId: "about" }),
    node({ id: "logout", order: 80, labelKey: "navigation.logOut", iconId: "logout", target: actionTarget("logout") })
  ]);

  const nodesById = new Map(NAVIGATION_NODES.map((entry) => [entry.id, entry]));
  const nodesByRoute = new Map(NAVIGATION_NODES
    .filter((entry) => ["route", "workspace"].includes(entry.target.kind))
    .map((entry) => [entry.target.routeId, entry]));
  const profilesById = new Map(CAPABILITY_PROFILES.map((entry) => [entry.id, entry]));

  function navigationNode(nodeId) {
    return nodesById.get(nodeId) || null;
  }

  function navigationNodeForRoute(routeId) {
    return nodesByRoute.get(routeId) || null;
  }

  function navigationChildren(parentId = null, { includeHidden = false } = {}) {
    return NAVIGATION_NODES
      .filter((entry) => entry.parentId === parentId && (includeHidden || entry.isVisible))
      .sort((left, right) => left.order - right.order || left.id.localeCompare(right.id));
  }

  function workspaceLocalDestinations(workspaceId, { includeHidden = false } = {}) {
    const workspace = navigationNode(workspaceId);
    if (workspace?.target.kind !== "workspace") return freeze([]);
    return freeze([
      freeze({
        nodeId: workspace.id,
        routeId: workspace.target.routeId,
        labelKey: workspace.target.defaultLabelKey,
        iconId: workspace.target.defaultIconId
      }),
      ...navigationChildren(workspaceId, { includeHidden }).map((entry) => freeze({
        nodeId: entry.id,
        routeId: entry.target.routeId,
        labelKey: entry.labelKey,
        iconId: entry.iconId
      }))
    ]);
  }

  function ancestorBranchIdsForNode(nodeId) {
    const result = [];
    let current = navigationNode(nodeId);
    while (current?.parentId) {
      current = navigationNode(current.parentId);
      if (current?.target.kind === "branch") result.unshift(current.id);
    }
    return freeze(result);
  }

  function ancestorContainerIdsForNode(nodeId) {
    const result = [];
    let current = navigationNode(nodeId);
    while (current?.parentId) {
      current = navigationNode(current.parentId);
      if (["branch", "group", "workspace"].includes(current?.target.kind)) result.unshift(current.id);
    }
    return freeze(result);
  }

  function canonicalRouteFromLegacyNavigation(navigation) {
    const route = navigation || {};
    if (route.view === "wallet") return `wallet.${route.walletSection || "assets"}`;
    if (route.view === "wallet-send") return "wallet.send";
    if (route.view === "wallet-receive") return "wallet.receive";
    if (route.view === "wallet-import") return "wallet.import";
    if (route.view === "wallet-merge-split") return "wallet.merge-split";
    if (route.view === "activity") return "wallet.history";
    if (route.view === "swap") return "wallet.swap";
    if (route.view === "staking") return "wallet.staking.stake";
    if (route.view === "wallet-backup") return "wallet.backup";
    if (route.view === "wallet-settings") return `wallet.settings.${route.walletSettingsSection || "general"}`;
    if (route.view === "settings") {
      return `settings.${route.settingsSection || "general"}`;
    }
    if (route.view === "telemetry") {
      const source = route.telemetrySource || "onionnet";
      const tab = route[`${source}TelemetryTab`] || "overview";
      return `telemetry.${source}.${tab}`;
    }
    if (route.view === "data-storage") return `data-storage.${route.dataStorageSection || "disk-usage"}`;
    if (route.view === "about") return "about";
    return demo.PORT_CONTRACT.defaultRouteByNamespace.wallet;
  }

  function isWalletRoute(routeId) {
    return demo.PORT_CONTRACT.walletRoutes.includes(routeId);
  }

  function validateNavigationModel({
    nodes = NAVIGATION_NODES,
    profiles = CAPABILITY_PROFILES,
    iconNames = demo.ICON_NAMES || [],
    locales = root.Z00ZLocaleRegistry || [],
    contract = demo.PORT_CONTRACT
  } = {}) {
    const errors = [];
    const nodeIds = new Set();
    const routeIds = new Set();
    const profileIds = new Set();
    const byId = new Map();
    const profileById = new Map();

    for (const capability of profiles) {
      if (profileIds.has(capability.id)) errors.push(`duplicate capability ID: ${capability.id}`);
      profileIds.add(capability.id);
      profileById.set(capability.id, capability);
      if (!contract.maturity.includes(capability.maturity)) errors.push(`invalid maturity: ${capability.id}`);
      if (!contract.availability.includes(capability.availability)) errors.push(`invalid availability: ${capability.id}`);
      if (!contract.evidenceSources.includes(capability.evidenceSource)) errors.push(`invalid evidence source: ${capability.id}`);
      if (!contract.freshness.includes(capability.freshness)) errors.push(`invalid freshness: ${capability.id}`);
      if (!contract.presentationModes.includes(capability.presentationMode)) errors.push(`invalid presentation mode: ${capability.id}`);
    }

    for (const entry of nodes) {
      if (!entry?.id || nodeIds.has(entry.id)) errors.push(`duplicate node ID: ${entry?.id || "missing"}`);
      nodeIds.add(entry?.id);
      byId.set(entry?.id, entry);
      if (!/^[a-z][a-z0-9.-]*$/.test(entry?.id || "")) errors.push(`invalid node ID: ${entry?.id || "missing"}`);
      if (!/^[a-z][A-Za-z0-9.-]*$/.test(entry?.labelKey || "")) errors.push(`invalid label key: ${entry?.id || "missing"}`);
      if (entry?.iconTone !== "neutral") errors.push(`non-neutral navigation icon: ${entry?.id || "missing"}`);
      if (typeof entry?.isVisible !== "boolean") errors.push(`invalid visibility: ${entry?.id || "missing"}`);
      if (!iconNames.includes(entry?.iconId)) errors.push(`missing icon: ${entry?.id || "missing"}`);
      if (!contract.presentationModes.includes(entry?.presentationMode)) errors.push(`invalid node presentation: ${entry?.id || "missing"}`);
      if (entry?.capabilityId && !profileById.has(entry.capabilityId)) errors.push(`missing capability profile: ${entry.id}`);
      if (entry?.capabilityId && profileById.get(entry.capabilityId)?.presentationMode !== entry.presentationMode) {
        errors.push(`capability presentation mismatch: ${entry.id}`);
      }
      if (!entry?.target || !["branch", "group", "workspace", "route", "help", "action"].includes(entry.target.kind)) {
        errors.push(`invalid target: ${entry?.id || "missing"}`);
        continue;
      }
      if (["route", "workspace"].includes(entry.target.kind)) {
        if (!contract.routes.includes(entry.target.routeId)) errors.push(`unknown route: ${entry.id}`);
        if (routeIds.has(entry.target.routeId)) errors.push(`duplicate route node: ${entry.target.routeId}`);
        routeIds.add(entry.target.routeId);
        if (!contract.helpTopics.includes(entry.helpTopicId)) errors.push(`invalid route Help topic: ${entry.id}`);
      }
      if (entry.target.kind === "workspace") {
        if (!/^[a-z][A-Za-z0-9.-]*$/.test(entry.target.defaultLabelKey || "")) errors.push(`invalid workspace default label: ${entry.id}`);
        if (!iconNames.includes(entry.target.defaultIconId)) errors.push(`missing workspace default icon: ${entry.id}`);
      }
      if (entry.target.kind === "help") {
        if (!contract.helpRoutes.includes(entry.target.helpRouteId)) errors.push(`unknown Help route: ${entry.id}`);
        if (!contract.helpTopics.includes(entry.helpTopicId)) errors.push(`invalid Help topic: ${entry.id}`);
      }
      if (entry.target.kind === "action" && !contract.actions.includes(entry.target.actionId)) {
        errors.push(`unknown action: ${entry.id}`);
      }
    }

    for (const entry of nodes) {
      if (entry.parentId !== null && !byId.has(entry.parentId)) errors.push(`missing parent: ${entry.id}`);
      if (entry.target.kind === "branch" && entry.parentId !== null) errors.push(`nested accordion: ${entry.id}`);
      if (entry.target.kind === "group" && entry.parentId === null) errors.push(`root group: ${entry.id}`);
      const parentKind = entry.parentId === null ? null : byId.get(entry.parentId)?.target.kind;
      if (entry.parentId !== null && !["branch", "group", "workspace"].includes(parentKind)) errors.push(`non-container parent: ${entry.id}`);
      if (entry.target.kind === "workspace" && entry.parentId !== null && parentKind !== "branch") {
        errors.push(`workspace must be a root leaf or first-level branch leaf: ${entry.id}`);
      }
      if (parentKind === "workspace" && entry.target.kind !== "route") errors.push(`workspace child must be a local route: ${entry.id}`);
      const seen = new Set([entry.id]);
      let current = entry;
      let depth = 1;
      while (current.parentId !== null) {
        current = byId.get(current.parentId);
        if (!current) break;
        if (seen.has(current.id)) {
          errors.push(`cycle at node: ${entry.id}`);
          break;
        }
        seen.add(current.id);
        depth += 1;
      }
      if (depth > 3) errors.push(`maximum depth exceeded: ${entry.id}`);
    }

    const siblings = new Map();
    for (const entry of nodes) {
      const siblingKey = entry.parentId || "__root__";
      const group = siblings.get(siblingKey) || [];
      group.push(entry);
      siblings.set(siblingKey, group);
    }
    for (const entries of siblings.values()) {
      const iconIds = entries.map((entry) => entry.iconId);
      if (new Set(iconIds).size !== iconIds.length) errors.push(`sibling icons must differ: ${entries[0].parentId || "root"}`);
      const orders = entries.map((entry) => entry.order);
      if (new Set(orders).size !== orders.length) errors.push(`sibling order must differ: ${entries[0].parentId || "root"}`);
    }

    const expectedRoutes = new Set(contract.routes);
    for (const routeId of expectedRoutes) if (!routeIds.has(routeId)) errors.push(`missing route node: ${routeId}`);
    for (const routeId of routeIds) if (!expectedRoutes.has(routeId)) errors.push(`unexpected route node: ${routeId}`);
    const defaultNamespaces = Object.keys(contract.defaultRouteByNamespace);
    if (defaultNamespaces.length !== contract.routeNamespaces.length || defaultNamespaces.some((id) => !contract.routeNamespaces.includes(id))) {
      errors.push("default route namespace mismatch");
    }
    for (const routeId of Object.values(contract.defaultRouteByNamespace)) {
      if (!expectedRoutes.has(routeId)) errors.push(`default route is unknown: ${routeId}`);
    }
    if (JSON.stringify(contract.palettes) !== JSON.stringify(["z00z-default", "z00z-corporate"])) {
      errors.push("palette registry must contain exactly Z00Z Default and Z00Z Corporate");
    }
    const localeIds = locales.map((locale) => typeof locale === "string" ? locale : locale.id);
    if (JSON.stringify(localeIds) !== JSON.stringify(contract.locales)) errors.push("locale registry mismatch");

    return freeze({ valid: errors.length === 0, errors: freeze(errors) });
  }

  function assertNavigationModel(options) {
    const result = validateNavigationModel(options);
    if (!result.valid) throw new Error(`Invalid navigation model: ${result.errors.join("; ")}`);
    return result;
  }

  Object.assign(root.Z00ZDemo, {
    CAPABILITY_PROFILES,
    NAVIGATION_NODES,
    assertNavigationModel,
    capabilityProfile: (capabilityId) => profilesById.get(capabilityId) || null,
    canonicalRouteFromLegacyNavigation,
    isWalletRoute,
    navigationChildren,
    navigationNode,
    navigationNodeForRoute,
    workspaceLocalDestinations,
    ancestorBranchIdsForNode,
    ancestorContainerIdsForNode,
    validateNavigationModel
  });
})(typeof window === "undefined" ? globalThis : window);
