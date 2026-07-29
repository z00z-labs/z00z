import "../port/contracts.js";
import "../port/navigation-model.js";

const demo = globalThis.Z00ZDemo;

function parentPath(pagePath) {
  return pagePath.slice(0, -1).join("/");
}

function pageStem(pagePath) {
  return pagePath.at(-1);
}

function titleWithoutNamespace(title) {
  return title.replace(/^[^:]{1,48}:\s+/u, "");
}

function descendants(nodeId) {
  return demo.navigationChildren(nodeId, { includeHidden: false })
    .flatMap((node) => [node, ...descendants(node.id)]);
}

function freezeItem(item) {
  return Object.freeze({
    ...item,
    relatedTopicIds: Object.freeze([...(item.relatedTopicIds || [item.topicId])]),
    ...(item.children ? { children: Object.freeze(item.children.map(freezeItem)) } : {}),
  });
}

function directoryTitle(content, path, fallback) {
  return content.directories[path]?.title || fallback;
}

function pageForTopic(content, topicId) {
  return content.pages.find((page) => page.topicId === topicId);
}

function recordForNode(records, node) {
  return records.find((record) => record.id === node.helpTopicId);
}

function recordIconId(record) {
  const node = record.nodeId ? demo.navigationNode(record.nodeId) : null;
  if (node?.target.kind === "workspace") return node.target.defaultIconId;
  if (node?.iconId) return node.iconId;
  if (record.scope === "guide") return "shield";
  if (record.scope === "dialog") return "eye";
  return "info";
}

function directRecords(content, records, directoryPath) {
  return content.directories[directoryPath]?.entries
    .map((entryName) => records.find((record) => (
      parentPath(record.pagePath) === directoryPath
      && pageStem(record.pagePath) === entryName
    )))
    .filter(Boolean) || [];
}

function pageItem(content, record, directoryPath) {
  const page = pageForTopic(content, record.id);
  return {
    iconId: recordIconId(record),
    id: record.nodeId || (record.id.startsWith("help.") ? record.id : `help.${record.id}`),
    relatedTopicIds: [record.id],
    title: titleWithoutNamespace(page.title),
    topicId: record.id,
    type: "article",
  };
}

function directPageItems(content, records, directoryPath) {
  return directRecords(content, records, directoryPath)
    .map((record) => pageItem(content, record, directoryPath));
}

function relatedTopicsForPath(records, path) {
  return records
    .filter((record) => record.pagePath.join("/").startsWith(`${path}/`))
    .map((record) => record.id);
}

function usesDirectoryIdentity(node, record) {
  const directoryPath = parentPath(record.pagePath);
  return node.target.kind === "workspace"
    || node.parentId === null
    || (pageStem(record.pagePath) === "index" && directoryPath !== node.parentId);
}

function nodeItem(content, records, node) {
  const record = recordForNode(records, node);
  if (!record) throw new Error(`Help content is missing the App navigation topic ${node.helpTopicId}.`);
  const page = pageForTopic(content, record.id);
  if (!page) throw new Error(`Help content is missing ${record.id}.`);
  const path = parentPath(record.pagePath);
  const directoryIdentity = usesDirectoryIdentity(node, record);
  return {
    ...(directoryIdentity ? { contextId: node.id, directoryPath: path } : {}),
    iconId: node.iconId,
    id: node.id,
    relatedTopicIds: directoryIdentity
      ? relatedTopicsForPath(records, path)
      : [record.id],
    title: directoryIdentity
      ? directoryTitle(content, path, titleWithoutNamespace(page.title))
      : titleWithoutNamespace(page.title),
    topicId: record.id,
    type: "article",
  };
}

function directoryItem(content, records, directoryPath) {
  const pages = directPageItems(content, records, directoryPath);
  if (!pages.length) return null;
  const recordsInDirectory = directRecords(content, records, directoryPath);
  const index = recordsInDirectory.findIndex((record) => pageStem(record.pagePath) === "index");
  const landing = pages[index < 0 ? 0 : index];
  const id = `content.${directoryPath.replaceAll("/", ".")}`;
  return {
    contextId: id,
    directoryPath,
    iconId: content.directories[directoryPath]?.iconId || landing.iconId,
    id,
    relatedTopicIds: pages.flatMap(({ relatedTopicIds }) => relatedTopicIds),
    title: directoryTitle(content, directoryPath, landing.title),
    topicId: landing.topicId,
    type: "article",
  };
}

function additionalItems(content, records, directoryPath, excludedDirectories = new Set()) {
  return content.directories[directoryPath]?.entries.flatMap((entryName) => {
    const childDirectory = directoryPath ? `${directoryPath}/${entryName}` : entryName;
    if (content.directories[childDirectory]) {
      if (excludedDirectories.has(childDirectory)) return [];
      return directoryItem(content, records, childDirectory) || [];
    }
    const record = directRecords(content, records, directoryPath)
      .find((candidate) => pageStem(candidate.pagePath) === entryName);
    if (!record || record.nodeId || !["article", "guide"].includes(record.scope)) return [];
    return pageItem(content, record, directoryPath);
  }) || [];
}

function branchItem(content, records, node) {
  const appChildNodes = demo.navigationChildren(node.id, { includeHidden: false });
  const appChildren = appChildNodes.map((child) => nodeItem(content, records, child));
  const appDirectories = new Set(appChildNodes.flatMap((child) => {
    const record = recordForNode(records, child);
    return record && usesDirectoryIdentity(child, record)
      ? [parentPath(record.pagePath)]
      : [];
  }));
  const extras = additionalItems(content, records, node.id, appDirectories);
  const relatedTopicIds = [
    ...descendants(node.id).map(({ helpTopicId }) => helpTopicId).filter(Boolean),
    ...extras.flatMap(({ relatedTopicIds }) => relatedTopicIds),
  ];
  return {
    children: [...appChildren, ...extras],
    iconId: node.iconId,
    id: node.id,
    relatedTopicIds,
    title: directoryTitle(content, node.id, node.id),
    type: "section",
  };
}

function guidesItem(content, records) {
  const children = additionalItems(content, records, "guides");
  if (!children.length) return null;
  return {
    children,
    iconId: content.directories.guides?.iconId || "question",
    id: "guides",
    relatedTopicIds: children.flatMap(({ relatedTopicIds }) => relatedTopicIds),
    title: directoryTitle(content, "guides", "Help Guides"),
    type: "section",
  };
}

function buildContextArticles(content, records, items) {
  const groupedItems = [];
  const visit = (item) => {
    if (item.contextId && item.directoryPath) groupedItems.push(item);
    item.children?.forEach(visit);
  };
  items.forEach(visit);
  return Object.fromEntries(groupedItems
    .map((item) => [
      item.contextId,
      Object.freeze(directPageItems(content, records, item.directoryPath).map(freezeItem)),
    ])
    .filter(([, contextItems]) => contextItems.length > 1));
}

export function buildPrimaryNavigation(content, records) {
  const rootNodes = demo.navigationChildren()
    .filter((node) => !["help", "logout"].includes(node.id));
  const items = rootNodes.map((node) => (
    node.target.kind === "branch"
      ? branchItem(content, records, node)
      : nodeItem(content, records, node)
  ));
  const guides = guidesItem(content, records);
  const aboutIndex = items.findIndex(({ id }) => id === "about");
  if (guides) items.splice(aboutIndex < 0 ? items.length : aboutIndex, 0, guides);
  return Object.freeze({
    contexts: Object.freeze(buildContextArticles(content, records, items)),
    directories: content.directories,
    homeTopicId: content.homeTopicId,
    items: Object.freeze(items.map(freezeItem)),
    title: content.title,
  });
}
