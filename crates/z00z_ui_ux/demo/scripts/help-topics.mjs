const DIALOG_DEFINITIONS = Object.freeze([
  Object.freeze({ id: "asset.details", group: "wallets", file: "assets-rights/asset-details", dialog: "asset-detail" }),
  Object.freeze({ id: "dapps.detail", group: "dapps", file: "detail", dialog: "dapps-detail" }),
  Object.freeze({ id: "dapps.permission-review", group: "dapps", file: "permission-review", dialog: "dapps-permission-review" }),
  Object.freeze({ id: "messenger.detail", group: "messenger", file: "detail", dialog: "messenger-detail" }),
  Object.freeze({ id: "messenger.request-review", group: "messenger", file: "request-review", dialog: "messenger-request-review" }),
  Object.freeze({ id: "contacts.detail", group: "contacts", file: "detail", dialog: "contacts-detail" }),
  Object.freeze({ id: "contacts.identity-review", group: "contacts", file: "identity-review", dialog: "contacts-identity-review" }),
  Object.freeze({ id: "telemetry.watchers.alert-detail", group: "telemetry", file: "watchers/alert-detail", dialog: "watchers-alert-detail" }),
  Object.freeze({ id: "telemetry.explorer.detail", group: "telemetry", file: "explorer/detail", dialog: "explorer-detail" })
]);

const ROOT_ARTICLE_DEFINITIONS = Object.freeze([
  Object.freeze({ id: "help.faq", file: "faq" }),
  Object.freeze({ id: "help.how-to", file: "how-to" }),
  Object.freeze({ id: "help.report-issues", file: "report-issues" }),
  Object.freeze({ id: "help.tips-and-tricks", file: "tips-and-tricks" }),
  Object.freeze({ id: "help.video-tutorials", file: "video-tutorials" })
]);

export const HELP_GROUP_DEFINITIONS = Object.freeze([
  Object.freeze({ id: "app", labelKey: "help.title", iconId: "question" }),
  Object.freeze({ id: "wallets", labelKey: "app.wallets", iconId: "wallet" }),
  Object.freeze({ id: "telemetry", labelKey: "navigation.telemetry", iconId: "network" }),
  Object.freeze({ id: "dapps", labelKey: "navigation.dapps", iconId: "spark" }),
  Object.freeze({ id: "messenger", labelKey: "navigation.messenger", iconId: "message" }),
  Object.freeze({ id: "data-storage", labelKey: "navigation.dataStorage", iconId: "storage" }),
  Object.freeze({ id: "contacts", labelKey: "navigation.contacts", iconId: "user" }),
  Object.freeze({ id: "settings", labelKey: "navigation.settings", iconId: "settings" })
]);

const DAPP_CONTEXT_TOPIC_ORDER = Object.freeze([
  "dapps.discover",
  "dapps.installed",
  "dapps.connections",
  "dapps.permissions",
  "wallet.swap",
  "wallet.exchange"
]);

function groupForRoute(routeId) {
  if (["wallet.swap", "wallet.exchange"].includes(routeId)) return "dapps";
  if (routeId.startsWith("wallet.")) return "wallets";
  if (routeId.startsWith("telemetry.")) return "telemetry";
  if (routeId.startsWith("dapps.")) return "dapps";
  if (routeId.startsWith("messenger.")) return "messenger";
  if (routeId.startsWith("contacts.")) return "contacts";
  if (routeId.startsWith("data-storage.")) return "data-storage";
  if (routeId.startsWith("settings.")) return "settings";
  if (routeId === "about") return "app";
  throw new Error(`Unsupported Help route namespace: ${routeId}`);
}

function fileForRoute(routeId) {
  if (routeId === "about") return "about";
  if (["wallet.assets", "wallet.vouchers", "wallet.permissions"].includes(routeId)) {
    return `assets-rights/${routeId.split(".").at(-1)}`;
  }
  if (routeId.startsWith("wallet.staking.")) return `staking/${routeId.split(".").at(-1)}`;
  if (routeId.startsWith("wallet.settings.")) return `settings/${routeId.split(".").at(-1)}`;
  if (routeId.startsWith("wallet.")) return routeId.split(".").at(-1);
  if (routeId.startsWith("telemetry.")) {
    const [, component, section] = routeId.split(".");
    return `${component}/${section}`;
  }
  if (routeId === "contacts.list") return "contacts";
  return routeId.split(".").at(-1);
}

export function helpTopicDefinitions(contract) {
  const groupOrder = new Map(HELP_GROUP_DEFINITIONS.map(({ id }, index) => [id, index]));
  const routeOrder = new Map(contract.routes.map((routeId, index) => [routeId, index]));
  const dappOrder = new Map(DAPP_CONTEXT_TOPIC_ORDER.map((routeId, index) => [routeId, index]));
  const routeTopics = contract.routes.map((routeId) => Object.freeze({
    id: routeId,
    group: groupForRoute(routeId),
    file: fileForRoute(routeId),
    ...(routeId === "about" ? { source: "root" } : {}),
    scope: "context",
    match: `activeRoute=${routeId}`
  })).sort((left, right) => {
    const groupDelta = groupOrder.get(left.group) - groupOrder.get(right.group);
    if (groupDelta) return groupDelta;
    if (left.group === "dapps") {
      return dappOrder.get(left.id) - dappOrder.get(right.id);
    }
    return routeOrder.get(left.id) - routeOrder.get(right.id);
  });
  return Object.freeze([
    Object.freeze({
      id: "app",
      group: "app",
      file: "app",
      scope: "global",
      match: "global"
    }),
    ...routeTopics,
    ...ROOT_ARTICLE_DEFINITIONS.map(({ id, file }) => Object.freeze({
      id,
      group: "app",
      file,
      source: "root",
      scope: "article",
      match: `article=${file}`
    })),
    ...DIALOG_DEFINITIONS.map(({ id, group, file, dialog }) => Object.freeze({
      id,
      group,
      file,
      scope: "dialog",
      match: `dialog=${dialog}`
    }))
  ]);
}

export function serializeHelpTopics(contract) {
  const topics = helpTopicDefinitions(contract);
  return `version: 1\ntopics:\n${topics.map((topic) => [
    `  - id: ${topic.id}`,
    `    group: ${topic.group}`,
    `    file: ${topic.file}`,
    ...(topic.source === "root" ? ["    source: root"] : []),
    `    scope: ${topic.scope}`,
    `    match: ${topic.match}`
  ].join("\n")).join("\n")}\n`;
}

export const DIALOG_HELP_TOPICS = DIALOG_DEFINITIONS;
