import "../port/contracts.js";
import "../port/navigation-model.js";

const demo = globalThis.Z00ZDemo;

if (!demo?.PORT_CONTRACT || !demo.navigationChildren || !demo.navigationNodeForRoute) {
  throw new Error("Demo navigation dependencies must load before the Help navigation contract.");
}

const DIALOG_PATHS = Object.freeze({
  "asset.details": { dialog: "asset-detail", path: ["wallet", "assets-rights", "asset-details"] },
  "dapps.detail": { dialog: "dapps-detail", path: ["dapps", "detail"] },
  "dapps.permission-review": { dialog: "dapps-permission-review", path: ["dapps", "permission-review"] },
  "messenger.detail": { dialog: "messenger-detail", path: ["messenger", "detail"] },
  "messenger.request-review": { dialog: "messenger-request-review", path: ["messenger", "request-review"] },
  "contacts.detail": { dialog: "contacts-detail", path: ["contacts", "detail"] },
  "contacts.identity-review": { dialog: "contacts-identity-review", path: ["contacts", "identity-review"] },
  "telemetry.watchers.alert-detail": { dialog: "watchers-alert-detail", path: ["telemetry", "watchers", "alert-detail"] },
  "telemetry.explorer.detail": { dialog: "explorer-detail", path: ["telemetry", "explorer", "detail"] },
});

const TITLE_SEGMENTS = Object.freeze({
  dapps: "dApps",
  "data-storage": "Data & Storage",
  onionnet: "OnionNet",
});

function segment(value) {
  return value.split(".").at(-1);
}

function titleSegment(value) {
  if (TITLE_SEGMENTS[value]) return TITLE_SEGMENTS[value];
  return value
    .split("-")
    .map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
    .join(" ");
}

export function helpTitle(topicId) {
  if (topicId === "app") return "App: Help";
  if (topicId === "about") return "App: About";

  const segments = topicId.split(".");
  if (segments.length < 2) throw new Error(`Help topic ${topicId} has no title namespace.`);
  return `${segments.slice(0, -1).map(titleSegment).join(" ")}: ${titleSegment(segments.at(-1))}`;
}

function nodeChain(node) {
  const chain = [];
  let current = node;

  while (current) {
    chain.unshift(current);
    current = current.parentId ? demo.navigationNode(current.parentId) : undefined;
  }

  return chain;
}

function pagePath(node) {
  const chain = nodeChain(node);

  if (chain.length === 1 && node.target.kind === "route") {
    const routeParts = node.id.split(".");
    return routeParts.length === 1 ? [node.id, "index"] : routeParts;
  }

  const parts = chain.flatMap((entry, index) => {
    if (entry.target.kind === "branch") return [entry.id];
    if (entry.target.kind === "workspace") return [segment(entry.id)];
    if (entry.target.kind === "route") return [segment(entry.id)];
    if (entry.target.kind === "help") return ["app", "index"];
    return index === chain.length - 1 ? [segment(entry.id)] : [];
  });

  if (node.target.kind === "workspace") parts.push("index");
  return parts;
}

function recordForRoute(routeId) {
  const node = demo.navigationNodeForRoute(routeId);

  if (!node?.helpTopicId) {
    throw new Error(`Route ${routeId} has no Help navigation node.`);
  }

  return Object.freeze({
    id: node.helpTopicId,
    labelKey: node.labelKey,
    nodeId: node.id,
    pagePath: Object.freeze(pagePath(node)),
    routeId,
    scope: "context",
  });
}

function dialogRecords() {
  return Object.entries(DIALOG_PATHS).map(([id, definition]) => Object.freeze({
    dialog: definition.dialog,
    id,
    labelKey: "help.title",
    nodeId: "",
    pagePath: Object.freeze(definition.path),
    routeId: "",
    scope: "dialog",
  }));
}

function appRecord() {
  return Object.freeze({
    id: "app",
    labelKey: "help.title",
    nodeId: "help",
    pagePath: Object.freeze(["app", "index"]),
    routeId: "",
    scope: "global",
  });
}

function assertRecords(records) {
  const topicIds = new Set();
  const pagePaths = new Set();

  for (const record of records) {
    const path = record.pagePath.join("/");
    if (topicIds.has(record.id)) throw new Error(`Duplicate Help topic ID: ${record.id}`);
    if (pagePaths.has(path)) throw new Error(`Duplicate Help page path: ${path}`);
    topicIds.add(record.id);
    pagePaths.add(path);
  }

  if (records.filter(({ scope }) => scope === "context").length !== demo.PORT_CONTRACT.routes.length) {
    throw new Error("Help navigation records do not cover every Demo route.");
  }
}

export function helpRecords() {
  const records = Object.freeze([
    appRecord(),
    ...demo.PORT_CONTRACT.routes.map(recordForRoute),
    ...dialogRecords(),
  ]);
  assertRecords(records);
  return records;
}

export function helpRecord(topicId) {
  return helpRecords().find((record) => record.id === topicId);
}

export function helpTree(parentId = undefined) {
  return demo.navigationChildren(parentId, { includeHidden: false })
    .filter((node) => node.target.kind !== "action")
    .map((node) => Object.freeze({
      ...node,
      children: helpTree(node.id),
      pagePath: node.helpTopicId ? helpRecord(node.helpTopicId)?.pagePath ?? [] : [],
    }));
}

export function pageFile(record) {
  return `${record.pagePath.join("/")}.md`;
}
