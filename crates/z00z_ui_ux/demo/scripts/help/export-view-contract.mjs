import { helpRecords, pageFile } from "./navigation-contract.mjs";

const demo = globalThis.Z00ZDemo;

function navigationPath(record) {
  const node = record.routeId ? demo.navigationNodeForRoute(record.routeId) : undefined;
  const nodes = [];
  let current = node;

  while (current) {
    nodes.unshift({ id: current.id, labelKey: current.labelKey, target: current.target.kind });
    current = current.parentId ? demo.navigationNode(current.parentId) : undefined;
  }
  return nodes;
}

function assetPath(id) {
  return `help/assets/en/${id.replaceAll(".", "-")}.png`;
}

export function viewContract() {
  return Object.freeze({
    sourceLocale: "en",
    version: 1,
    views: Object.freeze(helpRecords().map((record) => Object.freeze({
      dialog: record.dialog || "",
      id: record.id,
      navigationPath: Object.freeze(navigationPath(record)),
      pagePath: pageFile(record),
      routeId: record.routeId,
      scope: record.scope,
      screenshot: assetPath(record.id),
    }))),
  });
}

if (import.meta.url === `file://${process.argv[1]}`) {
  process.stdout.write(`${JSON.stringify(viewContract(), null, 2)}\n`);
}
