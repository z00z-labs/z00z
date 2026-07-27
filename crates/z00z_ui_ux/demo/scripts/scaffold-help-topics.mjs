import { access, mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";
import { helpDocumentPath } from "./help-source.mjs";
import { helpTopicDefinitions } from "./help-topics.mjs";
import { serializeHelpMarkdown } from "./sync-help.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const contractPath = resolve(demoRoot, "scripts/port/contracts.js");
const sandbox = { URLSearchParams };
sandbox.globalThis = sandbox;
vm.runInNewContext(await readFile(contractPath, "utf8"), sandbox, { filename: contractPath });

const LABELS = Object.freeze({
  assets: "Assets",
  vouchers: "Vouchers",
  permissions: "Permissions",
  quarantine: "Quarantine",
  send: "Send",
  receive: "Receive",
  history: "History",
  swap: "Swap",
  exchange: "Exchange",
  staking: "Staking",
  stake: "Stake",
  unstake: "Unstake",
  backup: "Backup",
  settings: "Wallet settings",
  general: "General",
  security: "Security",
  policies: "Policies",
  advanced: "Advanced",
  reticulum: "Reticulum",
  onionnet: "OnionNet",
  aggregators: "Aggregators",
  watchers: "Watchers",
  explorer: "Explorer",
  overview: "Overview",
  node: "Node",
  interfaces: "Interfaces",
  radio: "Radio",
  entrypoints: "Entrypoints",
  paths: "Paths",
  probes: "Probes",
  links: "Links",
  epoch: "Epoch",
  privacy: "Privacy",
  transport: "Transport",
  queues: "Queues",
  probation: "Probation",
  ingress: "Ingress",
  planning: "Planning",
  placement: "Placement",
  publication: "Publication",
  recovery: "Recovery",
  alerts: "Alerts",
  providers: "DA providers",
  censorship: "Censorship signals",
  evidence: "Public evidence",
  search: "Search",
  checkpoints: "Checkpoints",
  batches: "Batches",
  discover: "Discover",
  installed: "Installed",
  connections: "Connections",
  activity: "Activity",
  inbox: "Inbox",
  sent: "Sent",
  requests: "Requests",
  conversations: "Conversations",
  outbox: "Outbox",
  receipts: "Receipts",
  notifications: "Notifications",
  "disk-usage": "Disk usage",
  "network-usage": "Network usage",
  about: "About",
  list: "Contacts",
  detail: "Details",
  "permission-review": "Permission review",
  "request-review": "Request review",
  "identity-review": "Identity review",
  "alert-detail": "Alert details"
});

function routeTitle(topicId) {
  const parts = topicId.split(".");
  if (topicId === "about") return "About Z00Z";
  if (topicId === "asset.details") return "Asset details";
  if (parts[0] === "telemetry") {
    return `${LABELS[parts[1]]} ${LABELS[parts[2]].toLocaleLowerCase("en")}`;
  }
  if (parts[0] === "wallet" && parts[1] === "settings") {
    return `Wallet settings — ${LABELS[parts[2]]}`;
  }
  if (parts[0] === "dapps" && parts[1] === "permission-review") return "dApp permission review";
  if (parts[0] === "dapps" && parts[1] === "detail") return "dApp details";
  if (parts[0] === "messenger" && parts[1] === "detail") return "Messenger details";
  if (parts[0] === "messenger" && parts[1] === "request-review") return "Messenger request review";
  if (parts[0] === "contacts" && parts[1] === "detail") return "Contact details";
  if (parts[0] === "contacts" && parts[1] === "identity-review") return "Contact identity review";
  return LABELS[parts.at(-1)] || parts.at(-1);
}

function topicCopy(topic) {
  const title = routeTitle(topic.id);
  if (topic.id.startsWith("wallet.staking.")) {
    return {
      summary: `${title} explains the compatibility-only staking recipe and the authority still required from the native wallet.`,
      use: [
        `Use ${title} to prepare an amount only after the native wallet provides a verified staking position and terms.`,
        "Review validator, lock-up, unlock, fee, and settlement terms before any authorization."
      ],
      safety: [
        "The demo does not invent validators, delegated balances, rewards, unlock periods, or settlement state.",
        "Stake and Unstake remain unavailable until an authoritative wallet adapter supplies terms and reconciliation."
      ]
    };
  }
  if (topic.id === "wallet.quarantine") {
    return {
      summary: "Review wallet objects that require explicit local inspection before they can be used.",
      use: [
        "Inspect the stated reason, source, and local status before taking any recovery action.",
        "An unavailable action remains blocked until the native wallet reports a safe next step."
      ],
      safety: [
        "Quarantine never proves that an object is safe; authority remains with the native wallet policy.",
        "Secrets, raw signed packages, and private transport data never enter Help."
      ]
    };
  }
  if (topic.id.startsWith("telemetry.watchers.")) {
    return {
      summary: `${title} explains the read-only Watchers roadmap preview and its public evidence boundary.`,
      use: [
        `Use ${title} to inspect deterministic publication-health evidence without changing network state.`,
        "Unavailable, stale, malformed, and error states remain explicit and fail closed."
      ],
      safety: [
        "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
        "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
      ]
    };
  }
  if (topic.id.startsWith("telemetry.explorer.")) {
    return {
      summary: `${title} explains the privacy-bounded Explorer roadmap preview for supported public identifiers.`,
      use: [
        `Use ${title} only with the supported public checkpoint, batch, alert, or evidence identifiers.`,
        "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
      ],
      safety: [
        "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
        "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
      ]
    };
  }
  if (topic.id.startsWith("dapps.")) {
    return {
      summary: `${title} explains the bounded local dApps roadmap preview and its permission boundary.`,
      use: [
        `Use ${title} to inspect deterministic local descriptors, scoped intents, and explicit outcomes.`,
        "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
      ],
      safety: [
        "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
        "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
      ]
    };
  }
  if (topic.id.startsWith("messenger.")) {
    return {
      summary: `${title} explains the private request-coordination roadmap preview and its Wallet handoff.`,
      use: [
        `Use ${title} to inspect deterministic local messages, requests, delivery states, expiry, and recovery states.`,
        "Accepting a request creates a Wallet review intent; it does not settle or mutate wallet state."
      ],
      safety: [
        "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
        "Opening, deleting, blocking, or reporting content never changes Wallet settlement state."
      ]
    };
  }
  if (topic.id.startsWith("contacts.")) {
    return {
      summary: `${title} explains local contact labels, receiver cards, and explicit identity-change review.`,
      use: [
        `Use ${title} to inspect local contact data, expiry, revocation, and identity-change evidence.`,
        "A saved label is not proof of identity or trust; changed receiver data requires explicit review."
      ],
      safety: [
        "Contacts remain local and are never uploaded or published as an address or presence graph.",
        "Removing a local contact cannot revoke external credentials or change Wallet settlement."
      ]
    };
  }
  if (topic.id === "settings.notifications") {
    return {
      summary: "Choose local notification, vibration, and ringtone preferences for this device.",
      use: [
        "Use the master notification control before choosing a vibration policy or ringtone.",
        "Vibration and ringtone choices remain disabled when notifications are off."
      ],
      safety: [
        "These are local demo preferences and do not request operating-system permission.",
        "The packaged application must fail clearly when sound or haptic capability is unavailable."
      ]
    };
  }
  if (topic.id.startsWith("data-storage.")) {
    return {
      summary: `${title} explains privacy-bounded aggregate storage and network counters.`,
      use: [
        `Use ${title} to understand local resource use without opening private wallet records.`,
        "Displayed totals are deterministic fixtures and never represent a live device scan."
      ],
      safety: [
        "Contacts, destinations, messages, wallet activity, secrets, and arbitrary paths are excluded.",
        "A packaged app must expose aggregate counters only through a bounded native capability."
      ]
    };
  }
  if (topic.id === "about") {
    return {
      summary: "Review the Z00Z demo version, purpose, palette, and update channel.",
      use: [
        "Use Check for updates to verify the current demo metadata for this session.",
        "The JavaScript demo is the UX target for a future Rust and Tauri application."
      ],
      safety: [
        "The demo does not download or install an update.",
        "A packaged application must verify a signed release manifest before offering an update."
      ]
    };
  }
  throw new Error(`No English Help scaffold copy is defined for ${topic.id}`);
}

let created = 0;
for (const topic of helpTopicDefinitions(sandbox.Z00ZDemo.PORT_CONTRACT)) {
  if (topic.id === "app") continue;
  const outputPath = helpDocumentPath(demoRoot, "en", topic);
  try {
    await access(outputPath);
    continue;
  } catch {
    // Missing canonical English files are scaffolded; existing reviewed copy is preserved.
  }
  const copy = topicCopy(topic);
  const document = {
    id: topic.id,
    title: routeTitle(topic.id),
    summary: copy.summary,
    scope: topic.scope,
    sections: [
      {
        title: "Use this view",
        target: "current-view",
        blocks: [{ type: "list", items: copy.use }]
      },
      {
        title: "Local and safe behavior",
        target: "",
        blocks: [{ type: "list", items: copy.safety }]
      }
    ]
  };
  await mkdir(dirname(outputPath), { recursive: true });
  await writeFile(outputPath, serializeHelpMarkdown(document), "utf8");
  created += 1;
}

console.log(`Scaffolded ${created} missing English Help topic${created === 1 ? "" : "s"}; existing files were preserved.`);
