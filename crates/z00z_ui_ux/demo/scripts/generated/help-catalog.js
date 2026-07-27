"use strict";
((root) => {
  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };
  root.Z00ZHelpCatalog = deepFreeze({
  "version": 1,
  "locales": [
    "en",
    "ru",
    "fr",
    "de",
    "es",
    "pt",
    "ko",
    "tr",
    "ja",
    "zh-Hans"
  ],
  "groups": [
    {
      "id": "app",
      "labelKey": "help.title",
      "iconId": "question"
    },
    {
      "id": "wallets",
      "labelKey": "app.wallets",
      "iconId": "wallet"
    },
    {
      "id": "telemetry",
      "labelKey": "navigation.telemetry",
      "iconId": "network"
    },
    {
      "id": "dapps",
      "labelKey": "navigation.dapps",
      "iconId": "spark"
    },
    {
      "id": "messenger",
      "labelKey": "navigation.messenger",
      "iconId": "message"
    },
    {
      "id": "data-storage",
      "labelKey": "navigation.dataStorage",
      "iconId": "storage"
    },
    {
      "id": "contacts",
      "labelKey": "navigation.contacts",
      "iconId": "user"
    },
    {
      "id": "settings",
      "labelKey": "navigation.settings",
      "iconId": "settings"
    }
  ],
  "topics": [
    {
      "id": "app",
      "group": "app",
      "file": "app",
      "scope": "global",
      "match": {
        "global": "true"
      },
      "source": "group"
    },
    {
      "id": "about",
      "group": "app",
      "file": "about",
      "source": "root",
      "scope": "context",
      "match": {
        "activeRoute": "about"
      }
    },
    {
      "id": "wallet.assets",
      "group": "wallets",
      "file": "assets-rights/assets",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.assets"
      },
      "source": "group"
    },
    {
      "id": "wallet.vouchers",
      "group": "wallets",
      "file": "assets-rights/vouchers",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.vouchers"
      },
      "source": "group"
    },
    {
      "id": "wallet.permissions",
      "group": "wallets",
      "file": "assets-rights/permissions",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.permissions"
      },
      "source": "group"
    },
    {
      "id": "wallet.quarantine",
      "group": "wallets",
      "file": "quarantine",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.quarantine"
      },
      "source": "group"
    },
    {
      "id": "wallet.send",
      "group": "wallets",
      "file": "send",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.send"
      },
      "source": "group"
    },
    {
      "id": "wallet.receive",
      "group": "wallets",
      "file": "receive",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.receive"
      },
      "source": "group"
    },
    {
      "id": "wallet.history",
      "group": "wallets",
      "file": "history",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.history"
      },
      "source": "group"
    },
    {
      "id": "wallet.staking.stake",
      "group": "wallets",
      "file": "staking/stake",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.staking.stake"
      },
      "source": "group"
    },
    {
      "id": "wallet.staking.unstake",
      "group": "wallets",
      "file": "staking/unstake",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.staking.unstake"
      },
      "source": "group"
    },
    {
      "id": "wallet.backup",
      "group": "wallets",
      "file": "backup",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.backup"
      },
      "source": "group"
    },
    {
      "id": "wallet.settings.general",
      "group": "wallets",
      "file": "settings/general",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.settings.general"
      },
      "source": "group"
    },
    {
      "id": "wallet.settings.security",
      "group": "wallets",
      "file": "settings/security",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.settings.security"
      },
      "source": "group"
    },
    {
      "id": "wallet.settings.backup",
      "group": "wallets",
      "file": "settings/backup",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.settings.backup"
      },
      "source": "group"
    },
    {
      "id": "wallet.settings.policies",
      "group": "wallets",
      "file": "settings/policies",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.settings.policies"
      },
      "source": "group"
    },
    {
      "id": "wallet.settings.advanced",
      "group": "wallets",
      "file": "settings/advanced",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.settings.advanced"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.overview",
      "group": "telemetry",
      "file": "reticulum/overview",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.overview"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.node",
      "group": "telemetry",
      "file": "reticulum/node",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.node"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.interfaces",
      "group": "telemetry",
      "file": "reticulum/interfaces",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.interfaces"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.radio",
      "group": "telemetry",
      "file": "reticulum/radio",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.radio"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.entrypoints",
      "group": "telemetry",
      "file": "reticulum/entrypoints",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.entrypoints"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.paths",
      "group": "telemetry",
      "file": "reticulum/paths",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.paths"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.probes",
      "group": "telemetry",
      "file": "reticulum/probes",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.probes"
      },
      "source": "group"
    },
    {
      "id": "telemetry.reticulum.links",
      "group": "telemetry",
      "file": "reticulum/links",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.reticulum.links"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.overview",
      "group": "telemetry",
      "file": "onionnet/overview",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.overview"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.epoch",
      "group": "telemetry",
      "file": "onionnet/epoch",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.epoch"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.privacy",
      "group": "telemetry",
      "file": "onionnet/privacy",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.privacy"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.transport",
      "group": "telemetry",
      "file": "onionnet/transport",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.transport"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.queues",
      "group": "telemetry",
      "file": "onionnet/queues",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.queues"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.probation",
      "group": "telemetry",
      "file": "onionnet/probation",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.probation"
      },
      "source": "group"
    },
    {
      "id": "telemetry.onionnet.ingress",
      "group": "telemetry",
      "file": "onionnet/ingress",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.onionnet.ingress"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.overview",
      "group": "telemetry",
      "file": "aggregators/overview",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.overview"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.ingress",
      "group": "telemetry",
      "file": "aggregators/ingress",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.ingress"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.planning",
      "group": "telemetry",
      "file": "aggregators/planning",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.planning"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.placement",
      "group": "telemetry",
      "file": "aggregators/placement",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.placement"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.publication",
      "group": "telemetry",
      "file": "aggregators/publication",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.publication"
      },
      "source": "group"
    },
    {
      "id": "telemetry.aggregators.recovery",
      "group": "telemetry",
      "file": "aggregators/recovery",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.aggregators.recovery"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.overview",
      "group": "telemetry",
      "file": "watchers/overview",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.overview"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.alerts",
      "group": "telemetry",
      "file": "watchers/alerts",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.alerts"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.publication",
      "group": "telemetry",
      "file": "watchers/publication",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.publication"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.providers",
      "group": "telemetry",
      "file": "watchers/providers",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.providers"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.censorship",
      "group": "telemetry",
      "file": "watchers/censorship",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.censorship"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.evidence",
      "group": "telemetry",
      "file": "watchers/evidence",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.watchers.evidence"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.overview",
      "group": "telemetry",
      "file": "explorer/overview",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.explorer.overview"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.search",
      "group": "telemetry",
      "file": "explorer/search",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.explorer.search"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.checkpoints",
      "group": "telemetry",
      "file": "explorer/checkpoints",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.explorer.checkpoints"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.batches",
      "group": "telemetry",
      "file": "explorer/batches",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.explorer.batches"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.evidence",
      "group": "telemetry",
      "file": "explorer/evidence",
      "scope": "context",
      "match": {
        "activeRoute": "telemetry.explorer.evidence"
      },
      "source": "group"
    },
    {
      "id": "dapps.discover",
      "group": "dapps",
      "file": "discover",
      "scope": "context",
      "match": {
        "activeRoute": "dapps.discover"
      },
      "source": "group"
    },
    {
      "id": "dapps.installed",
      "group": "dapps",
      "file": "installed",
      "scope": "context",
      "match": {
        "activeRoute": "dapps.installed"
      },
      "source": "group"
    },
    {
      "id": "dapps.connections",
      "group": "dapps",
      "file": "connections",
      "scope": "context",
      "match": {
        "activeRoute": "dapps.connections"
      },
      "source": "group"
    },
    {
      "id": "dapps.permissions",
      "group": "dapps",
      "file": "permissions",
      "scope": "context",
      "match": {
        "activeRoute": "dapps.permissions"
      },
      "source": "group"
    },
    {
      "id": "wallet.swap",
      "group": "dapps",
      "file": "swap",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.swap"
      },
      "source": "group"
    },
    {
      "id": "wallet.exchange",
      "group": "dapps",
      "file": "exchange",
      "scope": "context",
      "match": {
        "activeRoute": "wallet.exchange"
      },
      "source": "group"
    },
    {
      "id": "messenger.inbox",
      "group": "messenger",
      "file": "inbox",
      "scope": "context",
      "match": {
        "activeRoute": "messenger.inbox"
      },
      "source": "group"
    },
    {
      "id": "messenger.sent",
      "group": "messenger",
      "file": "sent",
      "scope": "context",
      "match": {
        "activeRoute": "messenger.sent"
      },
      "source": "group"
    },
    {
      "id": "messenger.conversations",
      "group": "messenger",
      "file": "conversations",
      "scope": "context",
      "match": {
        "activeRoute": "messenger.conversations"
      },
      "source": "group"
    },
    {
      "id": "data-storage.disk-usage",
      "group": "data-storage",
      "file": "disk-usage",
      "scope": "context",
      "match": {
        "activeRoute": "data-storage.disk-usage"
      },
      "source": "group"
    },
    {
      "id": "data-storage.network-usage",
      "group": "data-storage",
      "file": "network-usage",
      "scope": "context",
      "match": {
        "activeRoute": "data-storage.network-usage"
      },
      "source": "group"
    },
    {
      "id": "contacts.list",
      "group": "contacts",
      "file": "contacts",
      "scope": "context",
      "match": {
        "activeRoute": "contacts.list"
      },
      "source": "group"
    },
    {
      "id": "settings.general",
      "group": "settings",
      "file": "general",
      "scope": "context",
      "match": {
        "activeRoute": "settings.general"
      },
      "source": "group"
    },
    {
      "id": "settings.notifications",
      "group": "settings",
      "file": "notifications",
      "scope": "context",
      "match": {
        "activeRoute": "settings.notifications"
      },
      "source": "group"
    },
    {
      "id": "settings.appearance",
      "group": "settings",
      "file": "appearance",
      "scope": "context",
      "match": {
        "activeRoute": "settings.appearance"
      },
      "source": "group"
    },
    {
      "id": "help.faq",
      "group": "app",
      "file": "faq",
      "source": "root",
      "scope": "article",
      "match": {
        "article": "faq"
      }
    },
    {
      "id": "help.how-to",
      "group": "app",
      "file": "how-to",
      "source": "root",
      "scope": "article",
      "match": {
        "article": "how-to"
      }
    },
    {
      "id": "help.report-issues",
      "group": "app",
      "file": "report-issues",
      "source": "root",
      "scope": "article",
      "match": {
        "article": "report-issues"
      }
    },
    {
      "id": "help.tips-and-tricks",
      "group": "app",
      "file": "tips-and-tricks",
      "source": "root",
      "scope": "article",
      "match": {
        "article": "tips-and-tricks"
      }
    },
    {
      "id": "help.video-tutorials",
      "group": "app",
      "file": "video-tutorials",
      "source": "root",
      "scope": "article",
      "match": {
        "article": "video-tutorials"
      }
    },
    {
      "id": "asset.details",
      "group": "wallets",
      "file": "assets-rights/asset-details",
      "scope": "dialog",
      "match": {
        "dialog": "asset-detail"
      },
      "source": "group"
    },
    {
      "id": "dapps.detail",
      "group": "dapps",
      "file": "detail",
      "scope": "dialog",
      "match": {
        "dialog": "dapps-detail"
      },
      "source": "group"
    },
    {
      "id": "dapps.permission-review",
      "group": "dapps",
      "file": "permission-review",
      "scope": "dialog",
      "match": {
        "dialog": "dapps-permission-review"
      },
      "source": "group"
    },
    {
      "id": "messenger.detail",
      "group": "messenger",
      "file": "detail",
      "scope": "dialog",
      "match": {
        "dialog": "messenger-detail"
      },
      "source": "group"
    },
    {
      "id": "messenger.request-review",
      "group": "messenger",
      "file": "request-review",
      "scope": "dialog",
      "match": {
        "dialog": "messenger-request-review"
      },
      "source": "group"
    },
    {
      "id": "contacts.detail",
      "group": "contacts",
      "file": "detail",
      "scope": "dialog",
      "match": {
        "dialog": "contacts-detail"
      },
      "source": "group"
    },
    {
      "id": "contacts.identity-review",
      "group": "contacts",
      "file": "identity-review",
      "scope": "dialog",
      "match": {
        "dialog": "contacts-identity-review"
      },
      "source": "group"
    },
    {
      "id": "telemetry.watchers.alert-detail",
      "group": "telemetry",
      "file": "watchers/alert-detail",
      "scope": "dialog",
      "match": {
        "dialog": "watchers-alert-detail"
      },
      "source": "group"
    },
    {
      "id": "telemetry.explorer.detail",
      "group": "telemetry",
      "file": "explorer/detail",
      "scope": "dialog",
      "match": {
        "dialog": "explorer-detail"
      },
      "source": "group"
    }
  ],
  "catalogues": {
    "en": {
      "app": {
        "id": "app",
        "title": "Application help",
        "summary": "Local application help explains this view and remains available offline.",
        "scope": "global",
        "sections": [
          {
            "title": "Use this help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open global Help for application navigation and offline behavior; use the question action inside a view for its controls.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          },
          {
            "title": "Test Text",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "test",
                  "test"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "About Z00Z",
        "summary": "Review the Z00Z demo version, purpose, palette, and update channel.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Check for updates to verify the current demo metadata for this session.",
                  "The JavaScript demo is the UX target for a future Rust and Tauri application."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo does not download or install an update.",
                  "A packaged application must verify a signed release manifest before offering an update."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Assets",
        "summary": "Browse the selected wallet’s coins, tokens, and NFTs with their local balances and market-data status.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use All, Coins, Tokens, or NFTs to narrow the selected wallet’s asset list.",
                  "Balance is wallet-owned. Value and Price stay Unavailable until a trusted market feed is connected."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Selecting a row opens read-only asset metadata; Send and Receive remain separate wallet actions.",
                  "Asset icons and this Help are packaged with the application and work offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Vouchers",
        "summary": "Vouchers explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filter vouchers by lifecycle, open a row for its terms, or create one when the wallet has none.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Permissions",
        "summary": "Permissions explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filter zero-value rights by Held, Delegated, or Used and open any row to inspect its bounded authority.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Quarantine",
        "summary": "Review wallet objects that require explicit local inspection before they can be used.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Inspect the stated reason, source, and local status before taking any recovery action.",
                  "An unavailable action remains blocked until the native wallet reports a safe next step."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Quarantine never proves that an object is safe; authority remains with the native wallet policy.",
                  "Secrets, raw signed packages, and private transport data never enter Help."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Send",
        "summary": "Send explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choose Assets, Vouchers, or Permissions first. Assets carry value, vouchers carry policy-bound conditional value, and permissions carry bounded zero-value authority.",
                  "Review the receiver plus the selected family’s balance or policy, expiry, remaining uses, scope, and delegation limits before authorizing once."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Receive",
        "summary": "Receive explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Show the selected wallet’s Receiver Card and copy its abbreviated receiver when sharing it out of band.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "History",
        "summary": "History explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filter wallet events by object family and open a row for its receipt and technical lifecycle.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Stake",
        "summary": "Stake explains the compatibility-only staking recipe and the authority still required from the native wallet.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Stake to prepare an amount only after the native wallet provides a verified staking position and terms.",
                  "Review validator, lock-up, unlock, fee, and settlement terms before any authorization."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo does not invent validators, delegated balances, rewards, unlock periods, or settlement state.",
                  "Stake and Unstake remain unavailable until an authoritative wallet adapter supplies terms and reconciliation."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Unstake",
        "summary": "Unstake explains the compatibility-only staking recipe and the authority still required from the native wallet.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Unstake to prepare an amount only after the native wallet provides a verified staking position and terms.",
                  "Review validator, lock-up, unlock, fee, and settlement terms before any authorization."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo does not invent validators, delegated balances, rewards, unlock periods, or settlement state.",
                  "Stake and Unstake remain unavailable until an authoritative wallet adapter supplies terms and reconciliation."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Wallet backup",
        "summary": "Wallet backup explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Check the latest local backup, integrity, and destination before creating a fresh encrypted backup.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Wallet general settings",
        "summary": "Wallet general settings explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Rename only the selected wallet; its wallet ID and creation-time chain remain read-only.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Wallet security",
        "summary": "Wallet security explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Set the inactivity lock, lock immediately, or change the selected wallet password.",
                  "Recovery-phrase access and master-key rotation require re-authentication and explicit confirmation; verify a backup before rotation."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Wallet backup settings",
        "summary": "Wallet backup settings explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Automatic backup, interval, create, and restore controls apply only to the selected wallet.",
                  "Restore validates integrity before replacement. Seed-only recovery does not restore labels, local history, receiver context, or disclosure artifacts."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Wallet policies",
        "summary": "Wallet policies explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review the profile, local spend rules, locked protocol rules, and compliance availability for this wallet.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Advanced wallet settings",
        "summary": "Advanced wallet settings explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Validate and apply the selected wallet’s safe local YAML draft; secrets and filesystem paths are excluded.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum overview",
        "summary": "Reticulum overview presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum overview evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulum node",
        "summary": "Reticulum node presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum node evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulum interfaces",
        "summary": "Reticulum interfaces presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum interfaces evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum radio",
        "summary": "Reticulum radio presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum radio evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulum entry points",
        "summary": "Reticulum entry points presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum entry points evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulum paths",
        "summary": "Reticulum paths presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum paths evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulum probes",
        "summary": "Reticulum probes presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum probes evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulum links",
        "summary": "Reticulum links presents read-only carrier evidence from the registered local Reticulum bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review reticulum links evidence supplied by the registered local bridge; this view cannot change Reticulum.",
                  "Unavailable means that no fresh local snapshot exists; addresses, destinations, routes, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet overview",
        "summary": "OnionNet overview presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet overview aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNet epoch",
        "summary": "OnionNet epoch presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet epoch aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNet privacy",
        "summary": "OnionNet privacy presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet privacy aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNet transport",
        "summary": "OnionNet transport presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet transport aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNet queues and replay",
        "summary": "OnionNet queues and replay presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet queues and replay aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet probation",
        "summary": "OnionNet probation presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet probation aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNet ingress",
        "summary": "OnionNet ingress presents privacy-safe OnionNet telemetry aggregates without exposing routes or sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review onionnet ingress aggregates supplied by the registered local bridge; this view cannot change OnionNet.",
                  "Unavailable means that no fresh local snapshot exists; routes, endpoints, session identifiers, and payloads remain hidden."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Aggregators overview",
        "summary": "Aggregators overview presents read-only publication and placement evidence from the registered local bridge.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review publication, placement, validation, and lifecycle evidence supplied by the registered local bridge.",
                  "Unavailable means that no fresh local snapshot exists; the demo does not invent network state."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Aggregator ingress",
        "summary": "Ingress explains how the runtime admits a transaction or claim payload as a digest-bound work item.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Check the `WorkPayload` to `WorkItem` or `RejectRecord` contract.",
                  "Unavailable means no fresh admission snapshot exists; it does not mean accepted or rejected."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed boundary",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Object-package binding changes the admission digest and intake identity.",
                  "Raw payloads, receivers, memos, and wallet-local routes never enter Help."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Aggregator planning",
        "summary": "Planning explains deterministic batch and shard-route binding without claiming settlement authority.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review planner mode, route generation, intake count, operation count, and digest ownership.",
                  "Unavailable means no verified `BatchPlanned` snapshot is connected."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed boundary",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Planner configuration, generation, route-table digest, and recomputed plan must agree.",
                  "Planning never finalizes settlement, publication, or storage truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Aggregator placement",
        "summary": "Placement explains the runtime-owned shard generation, primary owner, secondary readiness, and journal lineage view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review the `ShardPlacementView` contract without inferring global topology.",
                  "Unavailable means no current placement-table observation is connected."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed boundary",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The placement table must own the exact shard and routing generation.",
                  "Aggregator IDs are operational data; endpoints and wallet identities stay hidden."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Aggregator publication",
        "summary": "Publication explains how an ordered batch is bound to checkpoint, quorum, data-availability, and lifecycle evidence.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Follow `PublicationRequest` to `PublishedBatch` and `PublicationRecord`.",
                  "Unavailable means no verified publication or readiness bundle is connected."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed boundary",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Partial or mismatched provider, height, manifest, payload, statement, or evidence data is rejected.",
                  "Storage owns checkpoint roots, proofs, and lifecycle truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Aggregator recovery",
        "summary": "Recovery explains restart and secondary-takeover checks against committed route, generation, primary, and journal lineage.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review the `ShardRecoveryRecord`, recovery intent, durable state, and execution-ticket contract.",
                  "Unavailable means no committed recovery snapshot is connected."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed boundary",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wrong generation, primary, shard, batch, route, or lineage is rejected.",
                  "The renderer cannot initiate failover or mutate storage recovery truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Watchers overview",
        "summary": "Watchers overview explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers overview to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Watchers alerts",
        "summary": "Watchers alerts explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers alerts to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Watchers publication",
        "summary": "Watchers publication explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers publication to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Watchers da providers",
        "summary": "Watchers da providers explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers da providers to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Watchers censorship signals",
        "summary": "Watchers censorship signals explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers censorship signals to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Watchers public evidence",
        "summary": "Watchers public evidence explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers public evidence to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Explorer overview",
        "summary": "Explorer overview explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer overview only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Explorer search",
        "summary": "Explorer search explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer search only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Explorer checkpoints",
        "summary": "Explorer checkpoints explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer checkpoints only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Explorer batches",
        "summary": "Explorer batches explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer batches only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Explorer public evidence",
        "summary": "Explorer public evidence explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer public evidence only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Discover",
        "summary": "Discover explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Discover to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Installed",
        "summary": "Installed explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Installed to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Connections",
        "summary": "Connections explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Connections to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Permissions",
        "summary": "Permissions explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Permissions to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Swap",
        "summary": "Swap explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choose a held source asset, amount, and compatible target asset, then inspect the preview before submission.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Exchange",
        "summary": "Exchange explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choose Hyperliquid Spot for an order-book request or NEAR Intents for a solver-driven cross-chain request, then enter only the fields required by that execution model.",
                  "Review pair or route, recipient/refund controls, slippage and deadline. Quote, output, fees, deposit address, and execution status stay unavailable until a verified connector supplies them."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Inbox",
        "summary": "Inbox explains the private request-coordination roadmap preview and its Wallet handoff.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Inbox to inspect deterministic local messages, requests, receipts, expiry, and recovery states.",
                  "Accepting a request creates a Wallet review intent; it does not settle or mutate wallet state."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
                  "Opening, deleting, blocking, or reporting content never changes Wallet settlement state."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Sent",
        "summary": "Sent shows local delivery states while keeping them strictly separate from Wallet settlement.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Sent to inspect deterministic local messages, delivery states, expiry, and recovery states.",
                  "A sent or acknowledged message does not prove delivery, ownership, or Wallet settlement."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
                  "Retrying, expiring, or acknowledging content never changes Wallet settlement state."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Conversations",
        "summary": "Conversations explains the private request-coordination roadmap preview and its Wallet handoff.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Conversations to inspect deterministic local messages, requests, receipts, expiry, and recovery states.",
                  "Accepting a request creates a Wallet review intent; it does not settle or mutate wallet state."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
                  "Opening, deleting, blocking, or reporting content never changes Wallet settlement state."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Disk usage",
        "summary": "Disk usage explains privacy-bounded aggregate storage and network counters.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Disk usage to understand local resource use without opening private wallet records.",
                  "Displayed totals are deterministic fixtures and never represent a live device scan."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts, destinations, messages, wallet activity, secrets, and arbitrary paths are excluded.",
                  "A packaged app must expose aggregate counters only through a bounded native capability."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Network usage",
        "summary": "Network usage explains privacy-bounded aggregate storage and network counters.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Network usage to understand local resource use without opening private wallet records.",
                  "Displayed totals are deterministic fixtures and never represent a live device scan."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts, destinations, messages, wallet activity, secrets, and arbitrary paths are excluded.",
                  "A packaged app must expose aggregate counters only through a bounded native capability."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Contacts",
        "summary": "Contacts explains local contact labels, receiver cards, and explicit identity-change review.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Contacts to inspect local contact data, expiry, revocation, and identity-change evidence.",
                  "A saved label is not proof of identity or trust; changed receiver data requires explicit review."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts remain local and are never uploaded or published as an address or presence graph.",
                  "Removing a local contact cannot revoke external credentials or change Wallet settlement."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Application preferences",
        "summary": "Application preferences explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choose the application language, regional format, display time zone, and notification preference.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Notifications",
        "summary": "Choose local notification, vibration, and ringtone preferences for this device.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use the master notification control before choosing a vibration policy or ringtone.",
                  "Vibration and ringtone choices remain disabled when notifications are off."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "These are local demo preferences and do not request operating-system permission.",
                  "The packaged application must fail clearly when sound or haptic capability is unavailable."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Appearance",
        "summary": "Appearance explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Switch Dark or Light mode, choose a palette, and select the local YAML highlighting theme.",
                  "Unavailable, read-only, and pending states are shown explicitly."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet secrets and private transport data never enter Help.",
                  "This Help is packaged with the application and works offline."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Asset details",
        "summary": "Inspect the selected asset’s identity, issuer, supply, and local classification.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Name and ticker identify the asset; Owner and Asset ID identify its declared source.",
                  "Current and maximum supply remain Unavailable when the wallet has no authoritative local source."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "These fields are read-only and do not prove market value, ownership, or protocol trust.",
                  "The asset icon, metadata, and this Help are packaged locally and work offline."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApp details",
        "summary": "dApp details explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use dApp details to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApp permission review",
        "summary": "dApp permission review explains the bounded local dApps roadmap preview and its permission boundary.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use dApp permission review to inspect deterministic local descriptors, scoped intents, and explicit outcomes.",
                  "Review scope, uses, expiry, value, fee, disclosure, and revoke behavior before accepting an intent."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps is a roadmap preview: no remote app code, arbitrary URL, or generic signing request is executed.",
                  "Accepted intents are revalidated by the Wallet; this view cannot mutate wallet objects."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Messenger details",
        "summary": "Messenger details explains the private request-coordination roadmap preview and its Wallet handoff.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Messenger details to inspect deterministic local messages, requests, receipts, expiry, and recovery states.",
                  "Accepting a request creates a Wallet review intent; it does not settle or mutate wallet state."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
                  "Opening, deleting, blocking, or reporting content never changes Wallet settlement state."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Messenger request review",
        "summary": "Messenger request review explains the private request-coordination roadmap preview and its Wallet handoff.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Messenger request review to inspect deterministic local messages, requests, receipts, expiry, and recovery states.",
                  "Accepting a request creates a Wallet review intent; it does not settle or mutate wallet state."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger is a roadmap preview for short-lived relay coordination, not permanent on-chain chat.",
                  "Opening, deleting, blocking, or reporting content never changes Wallet settlement state."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Contact details",
        "summary": "Contact details explains local contact labels, receiver cards, and explicit identity-change review.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Contact details to inspect local contact data, expiry, revocation, and identity-change evidence.",
                  "A saved label is not proof of identity or trust; changed receiver data requires explicit review."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts remain local and are never uploaded or published as an address or presence graph.",
                  "Removing a local contact cannot revoke external credentials or change Wallet settlement."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Contact identity review",
        "summary": "Contact identity review explains local contact labels, receiver cards, and explicit identity-change review.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Contact identity review to inspect local contact data, expiry, revocation, and identity-change evidence.",
                  "A saved label is not proof of identity or trust; changed receiver data requires explicit review."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts remain local and are never uploaded or published as an address or presence graph.",
                  "Removing a local contact cannot revoke external credentials or change Wallet settlement."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Watchers alert details",
        "summary": "Watchers alert details explains the read-only Watchers roadmap preview and its public evidence boundary.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Watchers alert details to inspect deterministic publication-health evidence without changing network state.",
                  "Unavailable, stale, malformed, and error states remain explicit and fail closed."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers is a roadmap preview backed by local fixtures, not a shipped protocol capability.",
                  "Wallet labels, counterparties, route paths, messages, and secret material are never exposed."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Explorer details",
        "summary": "Explorer details explains the privacy-bounded Explorer roadmap preview for supported public identifiers.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Explorer details only with the supported public checkpoint, batch, alert, or evidence identifiers.",
                  "Unknown, private, malformed, or unavailable identifiers fail closed without a wallet lookup."
                ]
              }
            ]
          },
          {
            "title": "Local and safe behavior",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer is a roadmap preview backed by local fixtures, not a public wallet-data service.",
                  "Wallet-local balances, contacts, messages, memos, routes, and secret material never enter Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "ru": {
      "app": {
        "id": "app",
        "title": "Справка приложения",
        "summary": "Локальная справка объясняет этот экран и работает без интернета.",
        "scope": "global",
        "sections": [
          {
            "title": "Как использовать справку",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Откройте общую справку о навигации и работе без интернета; знак вопроса внутри экрана объясняет именно его элементы.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          },
          {
            "title": "Тестовый текст",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Тест",
                  "Тест"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "О приложении",
        "summary": "О приложении: версия, назначение и канал обновлений Z00Z.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте текущую версию демо для этой сессии.",
                  "JavaScript-демо задаёт UX-цель для Rust и Tauri."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Демо не скачивает и не устанавливает обновления.",
                  "Готовое приложение должно проверять подписанный манифест выпуска."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Активы",
        "summary": "Монеты, токены и NFT выбранного кошелька с локальными балансами и состоянием рыночных данных.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Фильтры «Все», «Монеты», «Токены» и NFT сужают список активов выбранного кошелька.",
                  "Баланс принадлежит кошельку. «Стоимость» и «Цена» остаются недоступными, пока не подключён доверенный источник котировок."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Нажатие строки открывает read-only сведения об активе; отправка и получение остаются отдельными действиями кошелька.",
                  "Иконки активов и эта справка встроены в приложение и работают без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Ваучеры",
        "summary": "Ваучеры: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Фильтруйте ваучеры по состоянию, открывайте строку для просмотра условий или создайте первый ваучер.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Разрешения",
        "summary": "Разрешения: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Фильтруйте права с нулевой стоимостью по Held, Delegated и Used и открывайте строку для проверки ограниченных полномочий.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Карантин",
        "summary": "Карантин: локальная справка для объектов, требующих явной проверки кошельком.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте указанную причину, источник и локальный статус перед любым действием.",
                  "Недоступное действие остаётся заблокированным, пока нативный кошелёк не сообщит безопасный следующий шаг."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Окончательное решение принимает политика нативного кошелька, а не этот экран.",
                  "Секреты и приватные транспортные данные не попадают в справку."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Отправка",
        "summary": "Отправка: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Сначала выберите Активы, Ваучеры или Полномочия. Это стоимость, условная стоимость по правилам или ограниченное право с нулевой стоимостью.",
                  "Перед однократной авторизацией проверьте получателя, а также баланс либо правила, срок, оставшиеся использования, область и делегирование."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Получение",
        "summary": "Получение: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Покажите Receiver Card выбранного кошелька и скопируйте сокращённый адрес для передачи по отдельному каналу.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "История",
        "summary": "История: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Фильтруйте события кошелька по типу объекта и открывайте строку для квитанции и технического жизненного цикла.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Стейкинг",
        "summary": "Стейкинг: локальная справка для объектов, требующих явной проверки кошельком.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте указанную причину, источник и локальный статус перед любым действием.",
                  "Недоступное действие остаётся заблокированным, пока нативный кошелёк не сообщит безопасный следующий шаг."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Окончательное решение принимает политика нативного кошелька, а не этот экран.",
                  "Секреты и приватные транспортные данные не попадают в справку."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Вывести из стейкинга",
        "summary": "Вывести из стейкинга: локальная справка для объектов, требующих явной проверки кошельком.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте указанную причину, источник и локальный статус перед любым действием.",
                  "Недоступное действие остаётся заблокированным, пока нативный кошелёк не сообщит безопасный следующий шаг."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Окончательное решение принимает политика нативного кошелька, а не этот экран.",
                  "Секреты и приватные транспортные данные не попадают в справку."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Резервная копия кошелька",
        "summary": "Резервная копия кошелька: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте дату, целостность и назначение последней локальной копии перед созданием новой зашифрованной копии.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Общие настройки кошелька",
        "summary": "Общие настройки кошелька: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Переименовать можно только выбранный кошелёк; его Wallet ID и выбранная при создании сеть доступны только для чтения.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Безопасность кошелька",
        "summary": "Безопасность кошелька: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Настройте блокировку по бездействию, заблокируйте приложение сразу или смените пароль выбранного кошелька.",
                  "Просмотр seed-фразы и ротация мастер-ключа требуют повторной аутентификации и явного подтверждения; перед ротацией проверьте резервную копию."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Настройки резервного копирования",
        "summary": "Настройки резервного копирования: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Автоматическое копирование, интервал, создание и восстановление относятся только к выбранному кошельку.",
                  "Перед заменой данных проверяется целостность. Восстановление только по seed не возвращает метки, локальную историю, контекст получателя и артефакты раскрытия."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Политики кошелька",
        "summary": "Политики кошелька: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте профиль, локальные лимиты трат, заблокированные правила протокола и доступность compliance для этого кошелька.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Расширенные настройки кошелька",
        "summary": "Расширенные настройки кошелька: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте и примените безопасный локальный YAML выбранного кошелька; секреты и пути файлов исключены.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Обзор Reticulum",
        "summary": "Обзор Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Обзор Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Узел Reticulum",
        "summary": "Узел Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Узел Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Интерфейсы Reticulum",
        "summary": "Интерфейсы Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Интерфейсы Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Радио Reticulum",
        "summary": "Радио Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Радио Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Точки входа Reticulum",
        "summary": "Точки входа Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Точки входа Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Маршруты Reticulum",
        "summary": "Маршруты Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Маршруты Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Пробы Reticulum",
        "summary": "Пробы Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Пробы Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Связи Reticulum",
        "summary": "Связи Reticulum: данные carrier-телеметрии только для чтения от зарегистрированного локального моста Reticulum.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные раздела «Связи Reticulum», полученные от локального моста; экран не изменяет Reticulum.",
                  "Недоступно означает отсутствие свежего локального снимка; адреса, назначения, маршруты и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "Обзор OnionNet",
        "summary": "Обзор OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Обзор OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "Эпоха OnionNet",
        "summary": "Эпоха OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Эпоха OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "Конфиденциальность OnionNet",
        "summary": "Конфиденциальность OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Конфиденциальность OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "Транспорт OnionNet",
        "summary": "Транспорт OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Транспорт OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "Очереди и повтор OnionNet",
        "summary": "Очереди и повтор OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Очереди и повтор OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "Проверка OnionNet",
        "summary": "Проверка OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Проверка OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "Входящий трафик OnionNet",
        "summary": "Входящий трафик OnionNet: безопасные агрегаты телеметрии OnionNet без раскрытия маршрутов и сессий.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте агрегаты раздела «Входящий трафик OnionNet», полученные от локального моста; экран не изменяет OnionNet.",
                  "Недоступно означает отсутствие свежего локального снимка; маршруты, endpoints, идентификаторы сессий и payload скрыты."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Обзор агрегаторов",
        "summary": "Обзор агрегаторов: данные публикации и размещения только для чтения от зарегистрированного локального моста.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте данные публикации, размещения, валидации и жизненного цикла от локального моста.",
                  "Недоступно означает отсутствие свежего локального снимка; demo не выдумывает состояние сети."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Вход агрегатора",
        "summary": "Экран объясняет, как runtime принимает транзакцию или claim как рабочий элемент, связанный с digest.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте контракт `WorkPayload` → `WorkItem` или `RejectRecord`.",
                  "Недоступно означает отсутствие свежего снимка admission, а не принятие или отказ."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed граница",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Привязка object package изменяет admission digest и intake identity.",
                  "Raw payload, получатели, memo и локальные маршруты кошелька не попадают в Help."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Планирование агрегатора",
        "summary": "Экран объясняет детерминированную привязку batch и shard route без заявления settlement authority.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте planner mode, generation маршрута, число intake и операций, а также владельцев digest.",
                  "Недоступно означает, что проверенный снимок `BatchPlanned` не подключён."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed граница",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Конфигурация, generation, route-table digest и пересчитанный plan должны совпадать.",
                  "Планирование не финализирует settlement, publication или storage truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Размещение агрегатора",
        "summary": "Экран объясняет shard generation, primary owner, готовность secondary и journal lineage, которыми владеет runtime.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте контракт `ShardPlacementView`, не делая выводов о глобальной топологии.",
                  "Недоступно означает отсутствие текущего наблюдения placement table."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed граница",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Placement table должна владеть точным shard и routing generation.",
                  "Aggregator ID — операционные данные; endpoints и identity кошелька скрыты."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Публикация агрегатора",
        "summary": "Экран объясняет привязку ordered batch к checkpoint, quorum, data availability и lifecycle evidence.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Следуйте цепочке `PublicationRequest` → `PublishedBatch` → `PublicationRecord`.",
                  "Недоступно означает отсутствие проверенной публикации или readiness bundle."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed граница",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Неполные или несовпадающие provider, height, manifest, payload, statement и evidence отклоняются.",
                  "Storage владеет checkpoint roots, proofs и lifecycle truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Восстановление агрегатора",
        "summary": "Экран объясняет проверки restart и secondary takeover по committed route, generation, primary и journal lineage.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверяйте `ShardRecoveryRecord`, recovery intent, durable state и execution ticket.",
                  "Недоступно означает отсутствие подключённого committed recovery snapshot."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed граница",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Неверные generation, primary, shard, batch, route или lineage отклоняются.",
                  "Renderer не может запускать failover или изменять storage recovery truth."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Наблюдатели — Обзор",
        "summary": "Наблюдатели — Обзор: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Наблюдатели — Оповещения",
        "summary": "Наблюдатели — Оповещения: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Наблюдатели — Публикация",
        "summary": "Наблюдатели — Публикация: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Наблюдатели — DA-провайдеры",
        "summary": "Наблюдатели — DA-провайдеры: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Наблюдатели — Сигналы цензуры",
        "summary": "Наблюдатели — Сигналы цензуры: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Наблюдатели — Экспорт доказательств",
        "summary": "Наблюдатели — Экспорт доказательств: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Обозреватель — Обзор",
        "summary": "Обозреватель — Обзор: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Обозреватель — Поиск",
        "summary": "Обозреватель — Поиск: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Обозреватель — Контрольные точки",
        "summary": "Обозреватель — Контрольные точки: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Обозреватель — Пакеты",
        "summary": "Обозреватель — Пакеты: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Обозреватель — Публичные доказательства",
        "summary": "Обозреватель — Публичные доказательства: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Обзор",
        "summary": "Обзор: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Установленные",
        "summary": "Установленные: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Подключения",
        "summary": "Подключения: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Полномочия",
        "summary": "Полномочия: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Обмен",
        "summary": "Обмен: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Выберите имеющийся исходный актив, сумму и совместимый целевой актив, затем проверьте предварительный расчёт.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Биржа",
        "summary": "Биржа: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Выберите Hyperliquid Spot для книги ордеров или NEAR Intents для кроссчейн-запроса через solver, затем заполните поля выбранной модели.",
                  "Проверьте пару либо маршрут, получателя/возврат, проскальзывание и срок. Курс, выход, комиссии, депозитный адрес и статус недоступны без проверенного коннектора."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Входящие",
        "summary": "Входящие: справка о приватном демо координации запросов и передаче в Кошелёк.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.",
                  "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.",
                  "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Отправленные",
        "summary": "Отправленные: справка о приватном демо координации запросов и передаче в Кошелёк.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.",
                  "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.",
                  "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Диалоги",
        "summary": "Диалоги: справка о приватном демо координации запросов и передаче в Кошелёк.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.",
                  "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.",
                  "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Использование диска",
        "summary": "Использование диска: агрегированные локальные показатели без приватных данных.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте экран для оценки ресурсов без открытия записей кошелька.",
                  "Показанные значения являются детерминированными демо-данными."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Контакты, сообщения, маршруты, операции и секреты исключены.",
                  "Готовое приложение должно получать только агрегаты через ограниченную нативную возможность."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Использование сети",
        "summary": "Использование сети: агрегированные локальные показатели без приватных данных.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте экран для оценки ресурсов без открытия записей кошелька.",
                  "Показанные значения являются детерминированными демо-данными."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Контакты, сообщения, маршруты, операции и секреты исключены.",
                  "Готовое приложение должно получать только агрегаты через ограниченную нативную возможность."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Контакты",
        "summary": "Контакты: справка о локальных метках контактов, карточках получателя и проверке изменения личности.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные данные контакта, срок, отзыв и доказательства изменения личности.",
                  "Сохранённая метка не доказывает личность или доверие; изменённые данные требуют явной проверки."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Контакты остаются локальными и не публикуются как граф адресов или присутствия.",
                  "Удаление локального контакта не отзывает внешние полномочия и не изменяет расчёты Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Настройки приложения",
        "summary": "Настройки приложения: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Выберите язык приложения, региональный формат, часовой пояс отображения и режим уведомлений.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Уведомления",
        "summary": "Уведомления: локальные настройки уведомлений, вибрации и мелодии.",
        "scope": "context",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Сначала включите уведомления, затем выберите режим вибрации и мелодию.",
                  "При отключённых уведомлениях зависимые параметры недоступны."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Демо не запрашивает системные разрешения.",
                  "Готовое приложение должно явно сообщать, если звук или вибрация недоступны."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Оформление",
        "summary": "Оформление: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Переключите тёмный или светлый режим, выберите палитру и локальную тему подсветки YAML.",
                  "Недоступные, read-only и ожидающие состояния обозначаются явно."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Секреты кошелька и приватные транспортные данные не попадают в справку.",
                  "Справка встроена в приложение и работает без интернета."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Сведения об активе",
        "summary": "Идентификаторы, эмитент, предложение и локальная классификация выбранного актива.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Название и тикер обозначают актив; владелец и Asset ID указывают его заявленный источник.",
                  "Текущее и максимальное предложение остаются недоступными без авторитетного локального источника."
                ]
              }
            ]
          },
          {
            "title": "Локальная и безопасная работа",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Поля доступны только для чтения и не подтверждают рыночную стоимость, владение или доверие к протоколу.",
                  "Иконка, метаданные и справка хранятся локально и работают без интернета."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps — сведения",
        "summary": "dApps — сведения: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps — проверка разрешения",
        "summary": "dApps — проверка разрешения: справка об ограниченном локальном демо dApps и его разрешениях.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные описания, ограниченные намерения и явный результат операции.",
                  "Перед принятием проверьте область, число использований, срок, сумму, комиссию, раскрытие и отзыв."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps — интерактивная дорожная карта: удалённый код, произвольные URL и универсальная подпись не выполняются.",
                  "Принятое намерение заново проверяет Кошелёк; этот экран не изменяет объекты кошелька."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Мессенджер — сведения",
        "summary": "Мессенджер — сведения: справка о приватном демо координации запросов и передаче в Кошелёк.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.",
                  "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.",
                  "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Мессенджер — проверка запроса",
        "summary": "Мессенджер — проверка запроса: справка о приватном демо координации запросов и передаче в Кошелёк.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные сообщения, запросы, квитанции, срок действия и состояния восстановления.",
                  "Принятие запроса создаёт намерение для проверки в Кошельке, но ничего не рассчитывает и не изменяет."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger — интерактивная дорожная карта краткоживущей ретрансляции, а не постоянный чат в цепочке.",
                  "Открытие, удаление, блокировка или жалоба не меняют состояние расчётов Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Контакты — сведения",
        "summary": "Контакты — сведения: справка о локальных метках контактов, карточках получателя и проверке изменения личности.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные данные контакта, срок, отзыв и доказательства изменения личности.",
                  "Сохранённая метка не доказывает личность или доверие; изменённые данные требуют явной проверки."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Контакты остаются локальными и не публикуются как граф адресов или присутствия.",
                  "Удаление локального контакта не отзывает внешние полномочия и не изменяет расчёты Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Контакты — проверка личности",
        "summary": "Контакты — проверка личности: справка о локальных метках контактов, карточках получателя и проверке изменения личности.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте локальные данные контакта, срок, отзыв и доказательства изменения личности.",
                  "Сохранённая метка не доказывает личность или доверие; изменённые данные требуют явной проверки."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Контакты остаются локальными и не публикуются как граф адресов или присутствия.",
                  "Удаление локального контакта не отзывает внешние полномочия и не изменяет расчёты Кошелька."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Наблюдатели — сведения об оповещении",
        "summary": "Наблюдатели — сведения об оповещении: справка о доступном только для чтения демо будущей функции Watchers и границе публичных доказательств.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте этот экран для проверки детерминированных данных о публикации без изменения состояния сети.",
                  "Недоступные, устаревшие, повреждённые состояния и ошибки показываются явно и безопасно блокируют действие."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers — интерактивная дорожная карта на локальных фикстурах, а не готовая возможность протокола.",
                  "Метки кошелька, контрагенты, маршруты, сообщения и секреты не раскрываются."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Обозреватель — сведения",
        "summary": "Обозреватель — сведения: справка о приватном демо Explorer для поддерживаемых публичных идентификаторов.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Использование этого экрана",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Используйте только поддерживаемые публичные идентификаторы контрольных точек, пакетов, оповещений и доказательств.",
                  "Неизвестные, приватные, повреждённые или недоступные идентификаторы блокируются без обращения к кошельку."
                ]
              }
            ]
          },
          {
            "title": "Локальное и безопасное поведение",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer — интерактивная дорожная карта на локальных фикстурах, а не сервис данных кошелька.",
                  "Локальные балансы, контакты, сообщения, заметки, маршруты и секреты не передаются в Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "fr": {
      "app": {
        "id": "app",
        "title": "Aide de l’application",
        "summary": "L’aide locale explique cette vue et reste disponible hors ligne.",
        "scope": "global",
        "sections": [
          {
            "title": "Utiliser cette aide",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ouvrez l’aide globale pour la navigation et le mode hors ligne ; le bouton question d’une vue explique ses propres commandes.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          },
          {
            "title": "Texte de test",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Test",
                  "Test"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "À propos",
        "summary": "À propos : version, objectif et canal de mise à jour Z00Z.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez la version de démonstration de cette session.",
                  "La démo JavaScript définit la cible UX pour Rust et Tauri."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La démo ne télécharge ni n’installe de mise à jour.",
                  "L’application doit vérifier un manifeste de version signé."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Actifs",
        "summary": "Parcourez les pièces, jetons et NFT du portefeuille sélectionné avec leurs soldes locaux et l’état des données de marché.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Utilisez Tous, Pièces, Jetons ou NFT pour filtrer les actifs du portefeuille sélectionné.",
                  "Le solde appartient au portefeuille. Valeur et Prix restent indisponibles sans source de marché fiable."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Une ligne ouvre les métadonnées en lecture seule ; Envoyer et Recevoir restent des actions séparées.",
                  "Les icônes et cette aide sont intégrées à l’application et fonctionnent hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Bons",
        "summary": "Bons explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtrez les bons par cycle de vie, ouvrez une ligne pour ses conditions ou créez le premier bon.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Autorisations",
        "summary": "Autorisations explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtrez les droits de valeur nulle par Détenu, Délégué ou Utilisé et ouvrez une ligne pour son autorité limitée.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Quarantaine",
        "summary": "Quarantaine : aide locale pour les objets qui exigent un examen explicite du portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le motif, la source et l’état local indiqués avant toute action.",
                  "Une action indisponible reste bloquée jusqu’à ce que le portefeuille natif fournisse une étape sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La décision finale appartient à la politique du portefeuille natif, pas à cette vue.",
                  "Les secrets et les données de transport privées n’entrent jamais dans l’aide."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Envoyer",
        "summary": "Envoyer explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choisissez d’abord Actifs, Bons ou Permissions : valeur, valeur conditionnelle régie par une politique, ou autorité limitée de valeur nulle.",
                  "Avant l’autorisation unique, vérifiez le destinataire ainsi que le solde ou la politique, l’expiration, les usages, la portée et la délégation."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Recevoir",
        "summary": "Recevoir explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Affichez la carte de réception du portefeuille et copiez son destinataire abrégé pour un partage hors bande.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "Historique",
        "summary": "Historique explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtrez les événements par famille d’objet et ouvrez une ligne pour son reçu et son cycle technique.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Staker",
        "summary": "Staker : aide locale pour les objets qui exigent un examen explicite du portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le motif, la source et l’état local indiqués avant toute action.",
                  "Une action indisponible reste bloquée jusqu’à ce que le portefeuille natif fournisse une étape sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La décision finale appartient à la politique du portefeuille natif, pas à cette vue.",
                  "Les secrets et les données de transport privées n’entrent jamais dans l’aide."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Retirer du staking",
        "summary": "Retirer du staking : aide locale pour les objets qui exigent un examen explicite du portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le motif, la source et l’état local indiqués avant toute action.",
                  "Une action indisponible reste bloquée jusqu’à ce que le portefeuille natif fournisse une étape sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La décision finale appartient à la politique du portefeuille natif, pas à cette vue.",
                  "Les secrets et les données de transport privées n’entrent jamais dans l’aide."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Sauvegarde du portefeuille",
        "summary": "Sauvegarde du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez la date, l’intégrité et la destination de la dernière sauvegarde avant d’en créer une nouvelle chiffrée.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Paramètres généraux du portefeuille",
        "summary": "Paramètres généraux du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Renommez uniquement le portefeuille sélectionné ; son ID et sa chaîne de création restent en lecture seule.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Sécurité du portefeuille",
        "summary": "Sécurité du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Réglez le verrouillage d’inactivité, verrouillez immédiatement ou modifiez le mot de passe du portefeuille sélectionné.",
                  "L’accès à la phrase de récupération et la rotation de la clé principale exigent une nouvelle authentification et une confirmation explicite ; vérifiez une sauvegarde avant la rotation."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Sauvegarde du portefeuille",
        "summary": "Sauvegarde du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La sauvegarde automatique, l’intervalle, la création et la restauration concernent uniquement le portefeuille sélectionné.",
                  "La restauration valide l’intégrité avant le remplacement. Une récupération par phrase seule ne restaure pas les libellés, l’historique local, le contexte du destinataire ni les artefacts de divulgation."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Politiques du portefeuille",
        "summary": "Politiques du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez le profil, les limites locales, les règles de protocole verrouillées et la disponibilité de conformité.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Paramètres avancés du portefeuille",
        "summary": "Paramètres avancés du portefeuille explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Validez et appliquez le brouillon YAML local sûr du portefeuille ; secrets et chemins sont exclus.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Vue d’ensemble Reticulum",
        "summary": "Vue d’ensemble Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Vue d’ensemble Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Nœud Reticulum",
        "summary": "Nœud Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Nœud Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Interfaces Reticulum",
        "summary": "Interfaces Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Interfaces Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Radio Reticulum",
        "summary": "Radio Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Radio Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Points d’entrée Reticulum",
        "summary": "Points d’entrée Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Points d’entrée Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Chemins Reticulum",
        "summary": "Chemins Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Chemins Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Sondes Reticulum",
        "summary": "Sondes Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Sondes Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Liens Reticulum",
        "summary": "Liens Reticulum présente les preuves de transport en lecture seule du pont Reticulum local enregistré.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves « Liens Reticulum » fournies par le pont local ; cette vue ne modifie pas Reticulum.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; adresses, destinations, routes et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "Vue d’ensemble OnionNet",
        "summary": "Vue d’ensemble OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Vue d’ensemble OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "Époque OnionNet",
        "summary": "Époque OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Époque OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "Confidentialité OnionNet",
        "summary": "Confidentialité OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Confidentialité OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "Transport OnionNet",
        "summary": "Transport OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Transport OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "Files et rejeu OnionNet",
        "summary": "Files et rejeu OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Files et rejeu OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "Probation OnionNet",
        "summary": "Probation OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Probation OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "Entrée OnionNet",
        "summary": "Entrée OnionNet présente des agrégats OnionNet respectueux de la confidentialité sans révéler routes ni sessions.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les agrégats « Entrée OnionNet » du pont local ; cette vue ne modifie pas OnionNet.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; routes, points de terminaison, sessions et contenus restent masqués."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Vue d’ensemble des agrégateurs",
        "summary": "Vue d’ensemble des agrégateurs présente les preuves de publication et de placement en lecture seule du pont local.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les preuves de publication, placement, validation et cycle de vie du pont local.",
                  "Indisponible signifie qu’aucun instantané local récent n’existe ; la démo n’invente pas l’état du réseau."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Entrée de l’agrégateur",
        "summary": "Cette vue explique comment le runtime admet une transaction ou une réclamation comme travail lié à un digest.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le contrat `WorkPayload` vers `WorkItem` ou `RejectRecord`.",
                  "Indisponible signifie qu’aucun instantané d’admission récent n’existe, pas que le travail est accepté ou rejeté."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La liaison d’un object package modifie le digest d’admission et l’identité d’entrée.",
                  "Payloads bruts, destinataires, mémos et routes locales du portefeuille restent hors de l’aide."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Planification de l’agrégateur",
        "summary": "Cette vue explique la liaison déterministe du batch et de la route shard sans revendiquer l’autorité de règlement.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le mode, la génération de route, les nombres d’entrées et d’opérations, et les digests.",
                  "Indisponible signifie qu’aucun instantané `BatchPlanned` vérifié n’est connecté."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Configuration, génération, digest de table de routes et plan recalculé doivent correspondre.",
                  "La planification ne finalise ni règlement, ni publication, ni vérité de stockage."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Placement de l’agrégateur",
        "summary": "Cette vue explique la génération shard, le primaire, l’état des secondaires et la lignée de journal détenus par le runtime.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez `ShardPlacementView` sans déduire une topologie globale.",
                  "Indisponible signifie qu’aucune observation actuelle de la table de placement n’est connectée."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La table doit posséder exactement le shard et la génération de routage.",
                  "Les IDs d’agrégateur sont opérationnels ; endpoints et identités du portefeuille restent cachés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Publication de l’agrégateur",
        "summary": "Cette vue explique la liaison d’un batch ordonné au checkpoint, quorum, DA et preuves de cycle de vie.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Suivez `PublicationRequest` vers `PublishedBatch` et `PublicationRecord`.",
                  "Indisponible signifie qu’aucune publication ou bundle de disponibilité vérifié n’est connecté."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Provider, hauteur, manifeste, payload, statement et evidence incomplets ou divergents sont rejetés.",
                  "Le stockage détient les racines, preuves et la vérité du cycle de vie."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Récupération de l’agrégateur",
        "summary": "Cette vue explique les contrôles de redémarrage et reprise secondaire sur route, génération, primaire et lignée engagés.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez `ShardRecoveryRecord`, l’intention, l’état durable et le ticket d’exécution.",
                  "Indisponible signifie qu’aucun instantané de récupération engagé n’est connecté."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Génération, primaire, shard, batch, route ou lignée incorrects sont rejetés.",
                  "Le renderer ne peut ni déclencher le failover ni modifier la vérité de récupération du stockage."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Observateurs — Aperçu",
        "summary": "Observateurs — Aperçu : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Observateurs — Alertes",
        "summary": "Observateurs — Alertes : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Observateurs — Publication",
        "summary": "Observateurs — Publication : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Observateurs — Fournisseurs DA",
        "summary": "Observateurs — Fournisseurs DA : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Observateurs — Signaux de censure",
        "summary": "Observateurs — Signaux de censure : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Observateurs — Export de preuves",
        "summary": "Observateurs — Export de preuves : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Explorateur — Aperçu",
        "summary": "Explorateur — Aperçu : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Explorateur — Rechercher",
        "summary": "Explorateur — Rechercher : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Explorateur — Points de contrôle",
        "summary": "Explorateur — Points de contrôle : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Explorateur — Lots",
        "summary": "Explorateur — Lots : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Explorateur — Preuves publiques",
        "summary": "Explorateur — Preuves publiques : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Découvrir",
        "summary": "Découvrir : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Installées",
        "summary": "Installées : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Connexions",
        "summary": "Connexions : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Autorisations",
        "summary": "Autorisations : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Swap privé",
        "summary": "Swap privé explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choisissez un actif détenu, un montant et un actif cible compatible, puis vérifiez l’aperçu.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Place d’échange",
        "summary": "Place d’échange explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choisissez Hyperliquid Spot pour un carnet d’ordres ou NEAR Intents pour une requête inter-chaînes pilotée par solveur.",
                  "Vérifiez paire ou route, destinataire/remboursement, glissement et délai. Devis, sortie, frais, adresse de dépôt et statut restent indisponibles sans connecteur vérifié."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Boîte de réception",
        "summary": "Boîte de réception : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.",
                  "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.",
                  "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Envoyés",
        "summary": "Envoyés : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.",
                  "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.",
                  "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Conversations",
        "summary": "Conversations : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.",
                  "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.",
                  "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Utilisation du disque",
        "summary": "Utilisation du disque : compteurs locaux agrégés sans données privées.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les ressources sans ouvrir les enregistrements du portefeuille.",
                  "Les valeurs affichées sont des données de démonstration déterministes."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts, messages, routes, activités et secrets sont exclus.",
                  "L’application doit obtenir uniquement des agrégats via une capacité native limitée."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Utilisation du réseau",
        "summary": "Utilisation du réseau : compteurs locaux agrégés sans données privées.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les ressources sans ouvrir les enregistrements du portefeuille.",
                  "Les valeurs affichées sont des données de démonstration déterministes."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contacts, messages, routes, activités et secrets sont exclus.",
                  "L’application doit obtenir uniquement des agrégats via une capacité native limitée."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Contacts",
        "summary": "Contacts : aide sur les libellés locaux, cartes de réception et changements d’identité explicites.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les données locales, l’expiration, la révocation et les preuves de changement d’identité.",
                  "Un libellé enregistré ne prouve ni identité ni confiance ; toute donnée modifiée exige un examen."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les contacts restent locaux et ne sont jamais publiés comme graphe d’adresses ou de présence.",
                  "Supprimer un contact local ne révoque pas les droits externes et ne change pas le règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Paramètres généraux",
        "summary": "Paramètres généraux explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Choisissez la langue, le format régional, le fuseau d’affichage et la préférence de notification de l’application.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Notifications",
        "summary": "Notifications : préférences locales de notification, vibration et sonnerie.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Activez les notifications avant de choisir vibration et sonnerie.",
                  "Les choix dépendants sont désactivés lorsque les notifications le sont."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La démo ne demande aucune autorisation système.",
                  "L’application doit signaler clairement une capacité audio ou haptique indisponible."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Apparence",
        "summary": "Apparence explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Basculez Clair ou Sombre, choisissez une palette et le thème local de coloration YAML.",
                  "Les états indisponible, lecture seule et en attente sont explicites."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les secrets du portefeuille et les données de transport privées restent hors de l’aide.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Détails de l’actif",
        "summary": "Consultez l’identité, l’émetteur, l’offre et la classification locale de l’actif sélectionné.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Le nom et le symbole identifient l’actif ; Propriétaire et ID d’actif indiquent sa source déclarée.",
                  "L’offre actuelle et maximale reste indisponible sans source locale faisant autorité."
                ]
              }
            ]
          },
          {
            "title": "Fonctionnement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ces champs sont en lecture seule et ne prouvent ni valeur de marché, ni propriété, ni confiance protocolaire.",
                  "L’icône, les métadonnées et cette aide sont locales et fonctionnent hors ligne."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps — détails",
        "summary": "dApps — détails : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps — examen de l’autorisation",
        "summary": "dApps — examen de l’autorisation : aide sur l’aperçu dApps local limité et sa frontière d’autorisation.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les descriptions locales, intentions limitées et résultats explicites.",
                  "Avant d’accepter, vérifiez portée, utilisations, expiration, valeur, frais, divulgation et révocation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps est un aperçu de feuille de route : aucun code distant, URL arbitraire ou signature générique n’est exécuté.",
                  "Le portefeuille revalide toute intention acceptée ; cette vue ne modifie pas ses objets."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Messagerie — détails",
        "summary": "Messagerie — détails : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.",
                  "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.",
                  "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Messagerie — examen de la demande",
        "summary": "Messagerie — examen de la demande : aide sur l’aperçu privé de coordination des demandes et son transfert vers le portefeuille.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les messages, demandes, reçus, expirations et états de récupération locaux.",
                  "Accepter une demande crée une intention à examiner dans le portefeuille sans règlement ni mutation."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger est un aperçu de feuille de route pour relais éphémères, pas une messagerie permanente sur chaîne.",
                  "Ouvrir, supprimer, bloquer ou signaler ne change jamais l’état de règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Contacts — détails",
        "summary": "Contacts — détails : aide sur les libellés locaux, cartes de réception et changements d’identité explicites.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les données locales, l’expiration, la révocation et les preuves de changement d’identité.",
                  "Un libellé enregistré ne prouve ni identité ni confiance ; toute donnée modifiée exige un examen."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les contacts restent locaux et ne sont jamais publiés comme graphe d’adresses ou de présence.",
                  "Supprimer un contact local ne révoque pas les droits externes et ne change pas le règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Contacts — examen de l’identité",
        "summary": "Contacts — examen de l’identité : aide sur les libellés locaux, cartes de réception et changements d’identité explicites.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Examinez les données locales, l’expiration, la révocation et les preuves de changement d’identité.",
                  "Un libellé enregistré ne prouve ni identité ni confiance ; toute donnée modifiée exige un examen."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Les contacts restent locaux et ne sont jamais publiés comme graphe d’adresses ou de présence.",
                  "Supprimer un contact local ne révoque pas les droits externes et ne change pas le règlement du portefeuille."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Observateurs — détails de l’alerte",
        "summary": "Observateurs — détails de l’alerte : aide sur l’aperçu Watchers en lecture seule et sa limite de preuves publiques.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez les données déterministes de publication sans modifier l’état du réseau.",
                  "Les états indisponibles, périmés, malformés et en erreur restent explicites et échouent de façon sûre."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers est un aperçu de feuille de route fondé sur des données locales, pas une fonction protocolaire livrée.",
                  "Les libellés du portefeuille, contreparties, routes, messages et secrets ne sont jamais exposés."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Explorateur — détails",
        "summary": "Explorateur — détails : aide sur l’aperçu Explorer respectueux de la vie privée pour les identifiants publics pris en charge.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "N’utilisez que les identifiants publics pris en charge pour les points de contrôle, lots, alertes ou preuves.",
                  "Les identifiants inconnus, privés, malformés ou indisponibles échouent sans consulter le portefeuille."
                ]
              }
            ]
          },
          {
            "title": "Comportement local et sûr",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer est un aperçu de feuille de route fondé sur des données locales, pas un service de données du portefeuille.",
                  "Soldes locaux, contacts, messages, notes, routes et secrets n’entrent jamais dans Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "de": {
      "app": {
        "id": "app",
        "title": "Anwendungshilfe",
        "summary": "Die lokale Hilfe erklärt diese Ansicht und bleibt offline verfügbar.",
        "scope": "global",
        "sections": [
          {
            "title": "Diese Hilfe verwenden",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Öffnen Sie die globale Hilfe für Navigation und Offline-Verhalten; die Frageaktion einer Ansicht erklärt deren Bedienelemente.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          },
          {
            "title": "Testtext",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Test",
                  "Test"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "Über",
        "summary": "Über: Z00Z-Version, Zweck und Aktualisierungskanal.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die aktuelle Demoversion dieser Sitzung.",
                  "Die JavaScript-Demo definiert das UX-Ziel für Rust und Tauri."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die Demo lädt oder installiert keine Aktualisierung.",
                  "Die App muss ein signiertes Veröffentlichungsmanifest prüfen."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Vermögenswerte",
        "summary": "Münzen, Token und NFTs der gewählten Wallet mit lokalen Salden und Markt­datenstatus durchsuchen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Mit Alle, Münzen, Token oder NFTs filtern Sie die Assets der gewählten Wallet.",
                  "Der Saldo gehört zur Wallet. Wert und Preis bleiben ohne vertrauenswürdigen Marktfeed nicht verfügbar."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Eine Zeile öffnet schreibgeschützte Metadaten; Senden und Empfangen bleiben getrennte Wallet-Aktionen.",
                  "Asset-Symbole und diese Hilfe sind lokal enthalten und funktionieren offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Gutscheine",
        "summary": "Gutscheine erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtern Sie Gutscheine nach Lebenszyklus, öffnen Sie Bedingungen oder erstellen Sie den ersten Gutschein.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Berechtigungen",
        "summary": "Berechtigungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtern Sie wertlose Rechte nach Gehalten, Delegiert oder Verwendet und öffnen Sie ihre begrenzte Autorität.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Quarantäne",
        "summary": "Quarantäne: lokale Hilfe für Objekte, die eine ausdrückliche Wallet-Prüfung benötigen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie vor jeder Aktion den angegebenen Grund, die Quelle und den lokalen Status.",
                  "Eine nicht verfügbare Aktion bleibt gesperrt, bis die native Wallet einen sicheren nächsten Schritt meldet."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die native Wallet-Richtlinie entscheidet endgültig, nicht diese Ansicht.",
                  "Geheimnisse und private Transportdaten gelangen nie in die Hilfe."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Senden",
        "summary": "Senden erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wählen Sie zuerst Assets, Gutscheine oder Berechtigungen: Wert, regelgebundener bedingter Wert oder begrenzte wertlose Autorität.",
                  "Prüfen Sie Empfänger sowie Saldo oder Regeln, Ablauf, verbleibende Nutzungen, Umfang und Delegation vor der einmaligen Autorisierung."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Empfangen",
        "summary": "Empfangen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Zeigen Sie die Empfängerkarte der Wallet und kopieren Sie den gekürzten Empfänger für die getrennte Weitergabe.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "Verlauf",
        "summary": "Verlauf erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtern Sie Wallet-Ereignisse nach Objektfamilie und öffnen Sie eine Zeile für Beleg und technischen Lebenszyklus.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Staken",
        "summary": "Staken: lokale Hilfe für Objekte, die eine ausdrückliche Wallet-Prüfung benötigen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie vor jeder Aktion den angegebenen Grund, die Quelle und den lokalen Status.",
                  "Eine nicht verfügbare Aktion bleibt gesperrt, bis die native Wallet einen sicheren nächsten Schritt meldet."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die native Wallet-Richtlinie entscheidet endgültig, nicht diese Ansicht.",
                  "Geheimnisse und private Transportdaten gelangen nie in die Hilfe."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Unstaken",
        "summary": "Unstaken: lokale Hilfe für Objekte, die eine ausdrückliche Wallet-Prüfung benötigen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie vor jeder Aktion den angegebenen Grund, die Quelle und den lokalen Status.",
                  "Eine nicht verfügbare Aktion bleibt gesperrt, bis die native Wallet einen sicheren nächsten Schritt meldet."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die native Wallet-Richtlinie entscheidet endgültig, nicht diese Ansicht.",
                  "Geheimnisse und private Transportdaten gelangen nie in die Hilfe."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Wallet-Sicherung",
        "summary": "Wallet-Sicherung erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Datum, Integrität und Ziel des letzten lokalen Backups, bevor Sie ein neues verschlüsseltes Backup erstellen.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Allgemeine Wallet-Einstellungen",
        "summary": "Allgemeine Wallet-Einstellungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Benennen Sie nur die gewählte Wallet um; Wallet-ID und Erstellungs-Chain bleiben schreibgeschützt.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Wallet-Sicherheit",
        "summary": "Wallet-Sicherheit erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Legen Sie die Inaktivitätssperre fest, sperren Sie sofort oder ändern Sie das Passwort der ausgewählten Wallet.",
                  "Zugriff auf die Wiederherstellungsphrase und Master-Key-Rotation erfordern erneute Authentifizierung und ausdrückliche Bestätigung; prüfen Sie vorher ein Backup."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Wallet-Backup",
        "summary": "Wallet-Backup erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Automatisches Backup, Intervall, Erstellen und Wiederherstellen gelten nur für die ausgewählte Wallet.",
                  "Vor dem Ersetzen wird die Integrität geprüft. Eine reine Seed-Wiederherstellung stellt Labels, lokalen Verlauf, Empfängerkontext und Offenlegungsartefakte nicht wieder her."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Wallet-Richtlinien",
        "summary": "Wallet-Richtlinien erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Profil, lokale Ausgabelimits, gesperrte Protokollregeln und Compliance-Verfügbarkeit.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Erweiterte Wallet-Einstellungen",
        "summary": "Erweiterte Wallet-Einstellungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Validieren und übernehmen Sie den sicheren lokalen YAML-Entwurf; Geheimnisse und Dateipfade sind ausgeschlossen.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum-Übersicht",
        "summary": "Reticulum-Übersicht zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Übersicht-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulum-Knoten",
        "summary": "Reticulum-Knoten zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Knoten-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulum-Schnittstellen",
        "summary": "Reticulum-Schnittstellen zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Schnittstellen-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum-Funk",
        "summary": "Reticulum-Funk zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Funk-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulum-Einstiegspunkte",
        "summary": "Reticulum-Einstiegspunkte zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Einstiegspunkte-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulum-Pfade",
        "summary": "Reticulum-Pfade zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Pfade-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulum-Sonden",
        "summary": "Reticulum-Sonden zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Sonden-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulum-Verbindungen",
        "summary": "Reticulum-Verbindungen zeigt schreibgeschützte Trägernachweise der registrierten lokalen Reticulum-Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten Reticulum-Verbindungen-Nachweise; diese Ansicht ändert Reticulum nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Adressen, Ziele, Routen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet-Übersicht",
        "summary": "OnionNet-Übersicht zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Übersicht-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNet-Epoche",
        "summary": "OnionNet-Epoche zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Epoche-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNet-Privatsphäre",
        "summary": "OnionNet-Privatsphäre zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Privatsphäre-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNet-Transport",
        "summary": "OnionNet-Transport zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Transport-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNet-Warteschlangen und Wiederholung",
        "summary": "OnionNet-Warteschlangen und Wiederholung zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Warteschlangen und Wiederholung-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet-Prüfung",
        "summary": "OnionNet-Prüfung zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Prüfung-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNet-Eingang",
        "summary": "OnionNet-Eingang zeigt datenschutzgerechte OnionNet-Aggregate ohne Routen oder Sitzungen offenzulegen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die von der lokalen Brücke gelieferten OnionNet-Eingang-Aggregate; diese Ansicht ändert OnionNet nicht.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; Routen, Endpunkte, Sitzungskennungen und Nutzdaten bleiben verborgen."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Aggregatoren-Übersicht",
        "summary": "Aggregatoren-Übersicht zeigt schreibgeschützte Publikations- und Platzierungsnachweise der lokalen Brücke.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Publikations-, Platzierungs-, Validierungs- und Lebenszyklusnachweise der lokalen Brücke.",
                  "Nicht verfügbar bedeutet, dass kein aktueller lokaler Snapshot vorliegt; die Demo erfindet keinen Netzwerkzustand."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Aggregator-Eingang",
        "summary": "Diese Ansicht erklärt, wie die Runtime eine Transaktion oder Claim-Nutzlast als digest-gebundenes WorkItem zulässt.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie den Vertrag `WorkPayload` zu `WorkItem` oder `RejectRecord`.",
                  "Nicht verfügbar bedeutet, dass kein aktueller Admission-Snapshot vorliegt, nicht Annahme oder Ablehnung."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed-Grenze",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Das Binden eines Object Package ändert Admission-Digest und Intake-Identität.",
                  "Rohe Payloads, Empfänger, Memos und lokale Wallet-Routen bleiben außerhalb der Hilfe."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Aggregator-Planung",
        "summary": "Diese Ansicht erklärt deterministische Batch- und Shard-Routenbindung ohne Settlement-Autorität zu behaupten.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Modus, Routengeneration, Intake- und Operationsanzahl sowie Digest-Besitz.",
                  "Nicht verfügbar bedeutet, dass kein verifizierter `BatchPlanned`-Snapshot verbunden ist."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed-Grenze",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Konfiguration, Generation, Routentabellen-Digest und neu berechneter Plan müssen übereinstimmen.",
                  "Planung finalisiert weder Settlement noch Publikation oder Storage-Wahrheit."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Aggregator-Platzierung",
        "summary": "Diese Ansicht erklärt Shard-Generation, Primärbesitz, Sekundärbereitschaft und Journal-Lineage der Runtime.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie `ShardPlacementView`, ohne eine globale Topologie abzuleiten.",
                  "Nicht verfügbar bedeutet, dass keine aktuelle Placement-Table-Beobachtung verbunden ist."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed-Grenze",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die Tabelle muss exakt den Shard und die Routing-Generation besitzen.",
                  "Aggregator-IDs sind Betriebsdaten; Endpoints und Wallet-Identitäten bleiben verborgen."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Aggregator-Publikation",
        "summary": "Diese Ansicht erklärt die Bindung eines geordneten Batch an Checkpoint-, Quorum-, DA- und Lifecycle-Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Folgen Sie `PublicationRequest` zu `PublishedBatch` und `PublicationRecord`.",
                  "Nicht verfügbar bedeutet, dass keine verifizierte Publikation oder Readiness-Bundle verbunden ist."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed-Grenze",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Unvollständige oder abweichende Provider-, Höhen-, Manifest-, Payload-, Statement- oder Evidence-Daten werden abgelehnt.",
                  "Storage besitzt Checkpoint-Wurzeln, Beweise und Lifecycle-Wahrheit."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Aggregator-Wiederherstellung",
        "summary": "Diese Ansicht erklärt Neustart- und Secondary-Takeover-Prüfungen gegen gebundene Route, Generation, Primärbesitz und Journal-Lineage.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie `ShardRecoveryRecord`, Intent, dauerhaften Zustand und Ausführungsticket.",
                  "Nicht verfügbar bedeutet, dass kein gebundener Recovery-Snapshot verbunden ist."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed-Grenze",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Falsche Generation, Primary, Shard, Batch, Route oder Lineage werden abgelehnt.",
                  "Der Renderer kann kein Failover starten oder die Recovery-Wahrheit des Storage ändern."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Beobachter – Übersicht",
        "summary": "Beobachter – Übersicht: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Beobachter – Warnungen",
        "summary": "Beobachter – Warnungen: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Beobachter – Veröffentlichung",
        "summary": "Beobachter – Veröffentlichung: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Beobachter – DA-Anbieter",
        "summary": "Beobachter – DA-Anbieter: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Beobachter – Zensursignale",
        "summary": "Beobachter – Zensursignale: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Beobachter – Beweisexport",
        "summary": "Beobachter – Beweisexport: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Explorer – Übersicht",
        "summary": "Explorer – Übersicht: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Explorer – Suchen",
        "summary": "Explorer – Suchen: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Explorer – Prüfpunkte",
        "summary": "Explorer – Prüfpunkte: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Explorer – Stapel",
        "summary": "Explorer – Stapel: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Explorer – Öffentliche Beweise",
        "summary": "Explorer – Öffentliche Beweise: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Entdecken",
        "summary": "Entdecken: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Installiert",
        "summary": "Installiert: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Verbindungen",
        "summary": "Verbindungen: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Berechtigungen",
        "summary": "Berechtigungen: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Privater Tausch",
        "summary": "Privater Tausch erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wählen Sie gehaltenes Quell-Asset, Betrag und kompatibles Ziel-Asset und prüfen Sie anschließend die Vorschau.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Börse",
        "summary": "Börse erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wählen Sie Hyperliquid Spot für eine Orderbuch-Anfrage oder NEAR Intents für eine Solver-basierte Cross-Chain-Anfrage.",
                  "Prüfen Sie Paar oder Route, Empfänger/Rückzahlung, Slippage und Frist. Angebot, Ausgabe, Gebühren, Einzahlungsadresse und Status bleiben ohne verifizierten Connector nicht verfügbar."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Posteingang",
        "summary": "Posteingang: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.",
                  "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.",
                  "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Gesendet",
        "summary": "Gesendet: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.",
                  "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.",
                  "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Unterhaltungen",
        "summary": "Unterhaltungen: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.",
                  "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.",
                  "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Speichernutzung",
        "summary": "Speichernutzung: aggregierte lokale Zähler ohne private Daten.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Ressourcen, ohne Wallet-Datensätze zu öffnen.",
                  "Die Werte sind deterministische Demodaten."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kontakte, Nachrichten, Routen, Aktivitäten und Geheimnisse sind ausgeschlossen.",
                  "Die App darf Aggregate nur über eine begrenzte native Funktion beziehen."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Netzwerknutzung",
        "summary": "Netzwerknutzung: aggregierte lokale Zähler ohne private Daten.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Ressourcen, ohne Wallet-Datensätze zu öffnen.",
                  "Die Werte sind deterministische Demodaten."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kontakte, Nachrichten, Routen, Aktivitäten und Geheimnisse sind ausgeschlossen.",
                  "Die App darf Aggregate nur über eine begrenzte native Funktion beziehen."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Kontakte",
        "summary": "Kontakte: Hilfe zu lokalen Kontaktbezeichnungen, Empfängerkarten und ausdrücklicher Identitätsprüfung.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Kontaktdaten, Ablauf, Widerruf und Nachweise einer Identitätsänderung.",
                  "Eine gespeicherte Bezeichnung beweist weder Identität noch Vertrauen; geänderte Daten müssen geprüft werden."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kontakte bleiben lokal und werden nie als Adress- oder Präsenzgraph hochgeladen.",
                  "Das Entfernen eines lokalen Kontakts widerruft keine externen Rechte und ändert keine Wallet-Abrechnung."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Allgemeine Einstellungen",
        "summary": "Allgemeine Einstellungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wählen Sie App-Sprache, Regionalformat, Anzeigezeitzone und Benachrichtigungseinstellung.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Benachrichtigungen",
        "summary": "Benachrichtigungen: lokale Einstellungen für Benachrichtigung, Vibration und Klingelton.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Aktivieren Sie Benachrichtigungen, bevor Sie Vibration und Ton wählen.",
                  "Abhängige Optionen sind bei deaktivierten Benachrichtigungen gesperrt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die Demo fordert keine Systemberechtigung an.",
                  "Die App muss fehlende Audio- oder Haptikfunktionen klar melden."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Darstellung",
        "summary": "Darstellung erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wechseln Sie Hell oder Dunkel, wählen Sie eine Palette und das lokale YAML-Hervorhebungsthema.",
                  "Nicht verfügbar, schreibgeschützt und ausstehend werden eindeutig angezeigt."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet-Geheimnisse und private Transportdaten gelangen nie in die Hilfe.",
                  "Diese Hilfe ist in der App enthalten und funktioniert offline."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Asset-Details",
        "summary": "Identität, Herausgeber, Angebot und lokale Klassifizierung des gewählten Assets prüfen.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Name und Kürzel bezeichnen das Asset; Eigentümer und Asset-ID nennen die deklarierte Quelle.",
                  "Aktuelles und maximales Angebot bleiben ohne maßgebliche lokale Quelle nicht verfügbar."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Die Felder sind schreibgeschützt und beweisen weder Marktwert noch Eigentum oder Protokollvertrauen.",
                  "Symbol, Metadaten und Hilfe sind lokal enthalten und funktionieren offline."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps – Details",
        "summary": "dApps – Details: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps – Berechtigung prüfen",
        "summary": "dApps – Berechtigung prüfen: Hilfe zur begrenzten lokalen dApps-Vorschau und ihrer Berechtigungsgrenze.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Beschreibungen, begrenzte Absichten und eindeutige Ergebnisse.",
                  "Prüfen Sie vor Annahme Umfang, Nutzungen, Ablauf, Wert, Gebühren, Offenlegung und Widerruf."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps ist eine Roadmap-Vorschau: Kein entfernter Code, keine beliebige URL und keine generische Signatur wird ausgeführt.",
                  "Die Wallet prüft angenommene Absichten erneut; diese Ansicht verändert keine Wallet-Objekte."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Messenger – Details",
        "summary": "Messenger – Details: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.",
                  "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.",
                  "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Messenger – Anfrage prüfen",
        "summary": "Messenger – Anfrage prüfen: Hilfe zur privaten Vorschau für Anfragekoordination und Wallet-Übergabe.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Nachrichten, Anfragen, Belege, Ablauf- und Wiederherstellungszustände.",
                  "Das Annehmen erzeugt eine Wallet-Prüfabsicht, führt aber keine Abrechnung oder Änderung aus."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger ist eine Roadmap-Vorschau für kurzlebige Relays, kein dauerhafter On-Chain-Chat.",
                  "Öffnen, Löschen, Blockieren oder Melden ändert niemals den Wallet-Abrechnungsstatus."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Kontakte – Details",
        "summary": "Kontakte – Details: Hilfe zu lokalen Kontaktbezeichnungen, Empfängerkarten und ausdrücklicher Identitätsprüfung.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Kontaktdaten, Ablauf, Widerruf und Nachweise einer Identitätsänderung.",
                  "Eine gespeicherte Bezeichnung beweist weder Identität noch Vertrauen; geänderte Daten müssen geprüft werden."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kontakte bleiben lokal und werden nie als Adress- oder Präsenzgraph hochgeladen.",
                  "Das Entfernen eines lokalen Kontakts widerruft keine externen Rechte und ändert keine Wallet-Abrechnung."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Kontakte – Identität prüfen",
        "summary": "Kontakte – Identität prüfen: Hilfe zu lokalen Kontaktbezeichnungen, Empfängerkarten und ausdrücklicher Identitätsprüfung.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokale Kontaktdaten, Ablauf, Widerruf und Nachweise einer Identitätsänderung.",
                  "Eine gespeicherte Bezeichnung beweist weder Identität noch Vertrauen; geänderte Daten müssen geprüft werden."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kontakte bleiben lokal und werden nie als Adress- oder Präsenzgraph hochgeladen.",
                  "Das Entfernen eines lokalen Kontakts widerruft keine externen Rechte und ändert keine Wallet-Abrechnung."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Beobachter – Warnungsdetails",
        "summary": "Beobachter – Warnungsdetails: Hilfe zur schreibgeschützten Watchers-Vorschau und ihrer Grenze für öffentliche Nachweise.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie deterministische Publikationsdaten, ohne den Netzwerkzustand zu ändern.",
                  "Nicht verfügbare, veraltete, fehlerhafte und ungültige Zustände bleiben sichtbar und scheitern sicher."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers ist eine Roadmap-Vorschau mit lokalen Fixtures, keine ausgelieferte Protokollfunktion.",
                  "Wallet-Namen, Gegenparteien, Routen, Nachrichten und Geheimnisse werden nicht offengelegt."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Explorer – Details",
        "summary": "Explorer – Details: Hilfe zur datenschutzbegrenzten Explorer-Vorschau für unterstützte öffentliche Kennungen.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verwenden Sie nur unterstützte öffentliche Kennungen für Prüfpunkte, Stapel, Warnungen oder Nachweise.",
                  "Unbekannte, private, ungültige oder nicht verfügbare Kennungen scheitern ohne Wallet-Abfrage."
                ]
              }
            ]
          },
          {
            "title": "Lokales und sicheres Verhalten",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer ist eine Roadmap-Vorschau mit lokalen Fixtures, kein Wallet-Datendienst.",
                  "Lokale Salden, Kontakte, Nachrichten, Notizen, Routen und Geheimnisse gelangen nie in Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "es": {
      "app": {
        "id": "app",
        "title": "Ayuda de la aplicación",
        "summary": "La ayuda local explica esta vista y sigue disponible sin conexión.",
        "scope": "global",
        "sections": [
          {
            "title": "Usar esta ayuda",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Abra la ayuda global para navegación y uso sin conexión; el botón de pregunta de una vista explica sus controles.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          },
          {
            "title": "Texto de prueba",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prueba",
                  "Prueba"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "Acerca de",
        "summary": "Acerca de: versión, propósito y canal de actualizaciones de Z00Z.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Compruebe la versión actual de la demo para esta sesión.",
                  "La demo JavaScript define el objetivo UX para Rust y Tauri."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La demo no descarga ni instala actualizaciones.",
                  "La aplicación debe verificar un manifiesto de versión firmado."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Activos",
        "summary": "Consulte monedas, tokens y NFT de la cartera seleccionada con sus saldos locales y el estado de datos de mercado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Todos, Monedas, Tokens o NFT para filtrar los activos de la cartera seleccionada.",
                  "El saldo pertenece a la cartera. Valor y Precio siguen no disponibles sin una fuente de mercado fiable."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Una fila abre metadatos de solo lectura; Enviar y Recibir siguen siendo acciones separadas.",
                  "Los iconos y esta ayuda se incluyen localmente y funcionan sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Vales",
        "summary": "Vales explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre vales por ciclo de vida, abra una fila para ver sus condiciones o cree el primer vale.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Permisos",
        "summary": "Permisos explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre derechos de valor cero por Conservado, Delegado o Usado y abra una fila para ver su autoridad limitada.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Cuarentena",
        "summary": "Cuarentena: ayuda local para objetos que requieren una revisión explícita de la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el motivo, el origen y el estado local indicados antes de realizar cualquier acción.",
                  "Una acción no disponible permanece bloqueada hasta que la cartera nativa indique un paso seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La política de la cartera nativa toma la decisión final, no esta vista.",
                  "Los secretos y los datos privados de transporte nunca entran en la ayuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Enviar",
        "summary": "Enviar explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Elija primero Activos, Vales o Permisos: valor, valor condicional sujeto a política o autoridad limitada de valor cero.",
                  "Antes de autorizar una vez, revise el destinatario y el saldo o la política, caducidad, usos restantes, alcance y delegación."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Recibir",
        "summary": "Recibir explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Muestre la tarjeta receptora de la cartera y copie el receptor abreviado para compartirlo por otro canal.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "Historial",
        "summary": "Historial explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre eventos por familia de objeto y abra una fila para ver su recibo y ciclo técnico.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Hacer staking",
        "summary": "Hacer staking: ayuda local para objetos que requieren una revisión explícita de la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el motivo, el origen y el estado local indicados antes de realizar cualquier acción.",
                  "Una acción no disponible permanece bloqueada hasta que la cartera nativa indique un paso seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La política de la cartera nativa toma la decisión final, no esta vista.",
                  "Los secretos y los datos privados de transporte nunca entran en la ayuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Retirar staking",
        "summary": "Retirar staking: ayuda local para objetos que requieren una revisión explícita de la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el motivo, el origen y el estado local indicados antes de realizar cualquier acción.",
                  "Una acción no disponible permanece bloqueada hasta que la cartera nativa indique un paso seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La política de la cartera nativa toma la decisión final, no esta vista.",
                  "Los secretos y los datos privados de transporte nunca entran en la ayuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Copia de la cartera",
        "summary": "Copia de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise fecha, integridad y destino de la última copia antes de crear una nueva copia cifrada.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Ajustes generales de la cartera",
        "summary": "Ajustes generales de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cambie solo el nombre de la cartera seleccionada; su ID y cadena de creación son de solo lectura.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Seguridad de la cartera",
        "summary": "Seguridad de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Configure el bloqueo por inactividad, bloquee ahora o cambie la contraseña de la cartera seleccionada.",
                  "El acceso a la frase de recuperación y la rotación de la clave maestra requieren nueva autenticación y confirmación explícita; verifique una copia antes de rotar."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Copia de seguridad de la cartera",
        "summary": "Copia de seguridad de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La copia automática, el intervalo, la creación y la restauración solo se aplican a la cartera seleccionada.",
                  "La restauración valida la integridad antes de reemplazar datos. Recuperar solo con la semilla no restaura etiquetas, historial local, contexto del receptor ni artefactos de divulgación."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Políticas de la cartera",
        "summary": "Políticas de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el perfil, los límites locales, las reglas de protocolo bloqueadas y la disponibilidad de cumplimiento.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Ajustes avanzados de la cartera",
        "summary": "Ajustes avanzados de la cartera explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Valide y aplique el borrador YAML local seguro; se excluyen secretos y rutas de archivos.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Resumen de Reticulum",
        "summary": "Resumen de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Resumen de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Nodo de Reticulum",
        "summary": "Nodo de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Nodo de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Interfaces de Reticulum",
        "summary": "Interfaces de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Interfaces de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Radio de Reticulum",
        "summary": "Radio de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Radio de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Puntos de entrada de Reticulum",
        "summary": "Puntos de entrada de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Puntos de entrada de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Rutas de Reticulum",
        "summary": "Rutas de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Rutas de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Sondas de Reticulum",
        "summary": "Sondas de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Sondas de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Enlaces de Reticulum",
        "summary": "Enlaces de Reticulum muestra evidencia de transporte de solo lectura del puente Reticulum local registrado.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de Enlaces de Reticulum suministrada por el puente local; esta vista no modifica Reticulum.",
                  "No disponible significa que no existe una instantánea local reciente; direcciones, destinos, rutas y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "Resumen de OnionNet",
        "summary": "Resumen de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Resumen de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "Época de OnionNet",
        "summary": "Época de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Época de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "Privacidad de OnionNet",
        "summary": "Privacidad de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Privacidad de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "Transporte de OnionNet",
        "summary": "Transporte de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Transporte de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "Colas y repetición de OnionNet",
        "summary": "Colas y repetición de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Colas y repetición de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "Prueba de OnionNet",
        "summary": "Prueba de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Prueba de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "Entrada de OnionNet",
        "summary": "Entrada de OnionNet muestra agregados de OnionNet que preservan la privacidad sin revelar rutas ni sesiones.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los agregados de Entrada de OnionNet suministrados por el puente local; esta vista no modifica OnionNet.",
                  "No disponible significa que no existe una instantánea local reciente; rutas, endpoints, sesiones y cargas permanecen ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Resumen de agregadores",
        "summary": "Resumen de agregadores muestra evidencia de publicación y colocación de solo lectura del puente local.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la evidencia de publicación, colocación, validación y ciclo de vida del puente local.",
                  "No disponible significa que no existe una instantánea local reciente; la demo no inventa el estado de la red."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Entrada del agregador",
        "summary": "Esta vista explica cómo el runtime admite una transacción o reclamación como trabajo ligado a un digest.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el contrato `WorkPayload` hacia `WorkItem` o `RejectRecord`.",
                  "No disponible significa que no hay snapshot reciente de admisión, no que fue aceptado o rechazado."
                ]
              }
            ]
          },
          {
            "title": "Límite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vincular un object package cambia el digest de admisión y la identidad de entrada.",
                  "Payloads sin filtrar, receptores, notas y rutas locales de cartera no entran en la ayuda."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Planificación del agregador",
        "summary": "Esta vista explica la vinculación determinista de batch y ruta shard sin afirmar autoridad de liquidación.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise modo, generación de ruta, recuentos de entradas y operaciones, y propiedad de digests.",
                  "No disponible significa que no hay un snapshot `BatchPlanned` verificado conectado."
                ]
              }
            ]
          },
          {
            "title": "Límite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Configuración, generación, digest de tabla de rutas y plan recalculado deben coincidir.",
                  "La planificación no finaliza liquidación, publicación ni verdad de almacenamiento."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Colocación del agregador",
        "summary": "Esta vista explica generación shard, propietario primario, estado de secundarios y linaje del journal del runtime.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise `ShardPlacementView` sin deducir una topología global.",
                  "No disponible significa que no hay una observación actual de la tabla de colocación."
                ]
              }
            ]
          },
          {
            "title": "Límite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La tabla debe poseer exactamente el shard y la generación de enrutamiento.",
                  "Los IDs de agregador son datos operativos; endpoints e identidades de cartera permanecen ocultos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Publicación del agregador",
        "summary": "Esta vista explica cómo un batch ordenado se vincula a checkpoint, quorum, DA y evidencia de ciclo de vida.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Siga `PublicationRequest` hacia `PublishedBatch` y `PublicationRecord`.",
                  "No disponible significa que no hay publicación o paquete de readiness verificado."
                ]
              }
            ]
          },
          {
            "title": "Límite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Datos incompletos o divergentes de proveedor, altura, manifiesto, payload, statement o evidence se rechazan.",
                  "Storage es autoridad de raíces, pruebas y ciclo de vida del checkpoint."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Recuperación del agregador",
        "summary": "Esta vista explica controles de reinicio y toma secundaria contra ruta, generación, primario y linaje confirmados.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise `ShardRecoveryRecord`, intención, estado durable y ticket de ejecución.",
                  "No disponible significa que no hay snapshot de recuperación confirmado conectado."
                ]
              }
            ]
          },
          {
            "title": "Límite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Generación, primario, shard, batch, ruta o linaje incorrectos se rechazan.",
                  "El renderer no puede iniciar failover ni modificar la verdad de recuperación de storage."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Observadores — Resumen",
        "summary": "Observadores — Resumen: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Observadores — Alertas",
        "summary": "Observadores — Alertas: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Observadores — Publicación",
        "summary": "Observadores — Publicación: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Observadores — Proveedores DA",
        "summary": "Observadores — Proveedores DA: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Observadores — Señales de censura",
        "summary": "Observadores — Señales de censura: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Observadores — Exportar pruebas",
        "summary": "Observadores — Exportar pruebas: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Explorador — Resumen",
        "summary": "Explorador — Resumen: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Explorador — Buscar",
        "summary": "Explorador — Buscar: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Explorador — Puntos de control",
        "summary": "Explorador — Puntos de control: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Explorador — Lotes",
        "summary": "Explorador — Lotes: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Explorador — Pruebas públicas",
        "summary": "Explorador — Pruebas públicas: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Descubrir",
        "summary": "Descubrir: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Instaladas",
        "summary": "Instaladas: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Conexiones",
        "summary": "Conexiones: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Permisos",
        "summary": "Permisos: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Intercambio privado",
        "summary": "Intercambio privado explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Elija un activo de origen disponible, el importe y un destino compatible y revise la vista previa.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Casa de cambio",
        "summary": "Casa de cambio explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Elija Hyperliquid Spot para un libro de órdenes o NEAR Intents para una solicitud entre cadenas mediante solver.",
                  "Revise par o ruta, destinatario/reembolso, deslizamiento y plazo. Cotización, salida, comisiones, dirección de depósito y estado quedan no disponibles sin un conector verificado."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Bandeja de entrada",
        "summary": "Bandeja de entrada: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.",
                  "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.",
                  "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Enviados",
        "summary": "Enviados: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.",
                  "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.",
                  "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Conversaciones",
        "summary": "Conversaciones: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.",
                  "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.",
                  "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Uso del disco",
        "summary": "Uso del disco: contadores locales agregados sin datos privados.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise recursos sin abrir registros de la cartera.",
                  "Los valores mostrados son datos de demostración deterministas."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Se excluyen contactos, mensajes, rutas, actividad y secretos.",
                  "La aplicación solo debe obtener agregados mediante una capacidad nativa limitada."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Uso de red",
        "summary": "Uso de red: contadores locales agregados sin datos privados.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise recursos sin abrir registros de la cartera.",
                  "Los valores mostrados son datos de demostración deterministas."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Se excluyen contactos, mensajes, rutas, actividad y secretos.",
                  "La aplicación solo debe obtener agregados mediante una capacidad nativa limitada."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Contactos",
        "summary": "Contactos: ayuda sobre etiquetas locales, tarjetas receptoras y revisión explícita de identidad.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los datos locales, la caducidad, la revocación y las pruebas de cambio de identidad.",
                  "Una etiqueta guardada no demuestra identidad ni confianza; los datos modificados requieren revisión."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los contactos permanecen locales y nunca se publican como un grafo de direcciones o presencia.",
                  "Eliminar un contacto local no revoca derechos externos ni cambia la liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Ajustes generales",
        "summary": "Ajustes generales explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Elija el idioma, formato regional, zona horaria de visualización y preferencia de notificaciones.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Notificaciones",
        "summary": "Notificaciones: preferencias locales de notificación, vibración y tono.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Active las notificaciones antes de elegir vibración y tono.",
                  "Las opciones dependientes se desactivan al apagar las notificaciones."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "La demo no solicita permisos del sistema.",
                  "La aplicación debe indicar claramente si no hay sonido o vibración."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Apariencia",
        "summary": "Apariencia explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cambie entre Claro y Oscuro, elija una paleta y el tema local de resaltado YAML.",
                  "Los estados no disponible, solo lectura y pendiente se muestran claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los secretos de la cartera y los datos privados de transporte no entran en la ayuda.",
                  "Esta ayuda está incluida en la aplicación y funciona sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Detalles del activo",
        "summary": "Consulte la identidad, el emisor, el suministro y la clasificación local del activo seleccionado.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Nombre y símbolo identifican el activo; Propietario e ID del activo indican su fuente declarada.",
                  "El suministro actual y máximo sigue no disponible sin una fuente local autorizada."
                ]
              }
            ]
          },
          {
            "title": "Funcionamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los campos son de solo lectura y no prueban valor de mercado, propiedad ni confianza en el protocolo.",
                  "El icono, los metadatos y esta ayuda son locales y funcionan sin conexión."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps — detalles",
        "summary": "dApps — detalles: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps — revisión del permiso",
        "summary": "dApps — revisión del permiso: ayuda sobre la vista previa local y limitada de dApps y su límite de permisos.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise descriptores locales, intenciones limitadas y resultados explícitos.",
                  "Antes de aceptar, revise alcance, usos, caducidad, valor, comisión, divulgación y revocación."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps es una vista previa de la hoja de ruta: no ejecuta código remoto, URL arbitrarias ni firmas genéricas.",
                  "La cartera vuelve a validar cada intención aceptada; esta vista no modifica sus objetos."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Mensajero — detalles",
        "summary": "Mensajero — detalles: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.",
                  "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.",
                  "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Mensajero — revisión de la solicitud",
        "summary": "Mensajero — revisión de la solicitud: ayuda sobre la vista previa privada de coordinación de solicitudes y su entrega a la cartera.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise mensajes, solicitudes, recibos, caducidad y estados de recuperación locales.",
                  "Aceptar crea una intención para revisar en la cartera, pero no liquida ni modifica su estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger es una vista previa de la hoja de ruta para relés breves, no un chat permanente en cadena.",
                  "Abrir, borrar, bloquear o denunciar nunca cambia el estado de liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Contactos — detalles",
        "summary": "Contactos — detalles: ayuda sobre etiquetas locales, tarjetas receptoras y revisión explícita de identidad.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los datos locales, la caducidad, la revocación y las pruebas de cambio de identidad.",
                  "Una etiqueta guardada no demuestra identidad ni confianza; los datos modificados requieren revisión."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los contactos permanecen locales y nunca se publican como un grafo de direcciones o presencia.",
                  "Eliminar un contacto local no revoca derechos externos ni cambia la liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Contactos — revisión de identidad",
        "summary": "Contactos — revisión de identidad: ayuda sobre etiquetas locales, tarjetas receptoras y revisión explícita de identidad.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise los datos locales, la caducidad, la revocación y las pruebas de cambio de identidad.",
                  "Una etiqueta guardada no demuestra identidad ni confianza; los datos modificados requieren revisión."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Los contactos permanecen locales y nunca se publican como un grafo de direcciones o presencia.",
                  "Eliminar un contacto local no revoca derechos externos ni cambia la liquidación de la cartera."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Observadores — detalles de la alerta",
        "summary": "Observadores — detalles de la alerta: ayuda sobre la vista previa de solo lectura de Watchers y su límite de pruebas públicas.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise datos deterministas de publicación sin cambiar el estado de la red.",
                  "Los estados no disponibles, obsoletos, mal formados y de error siguen explícitos y fallan de forma segura."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers es una vista previa de la hoja de ruta con datos locales, no una función de protocolo disponible.",
                  "No se exponen etiquetas de cartera, contrapartes, rutas, mensajes ni secretos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Explorador — detalles",
        "summary": "Explorador — detalles: ayuda sobre la vista previa privada de Explorer para identificadores públicos compatibles.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use solo identificadores públicos compatibles de puntos de control, lotes, alertas o pruebas.",
                  "Los identificadores desconocidos, privados, mal formados o no disponibles fallan sin consultar la cartera."
                ]
              }
            ]
          },
          {
            "title": "Comportamiento local y seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer es una vista previa de la hoja de ruta con datos locales, no un servicio de datos de cartera.",
                  "Los saldos, contactos, mensajes, notas, rutas y secretos locales nunca entran en Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "pt": {
      "app": {
        "id": "app",
        "title": "Ajuda da aplicação",
        "summary": "A ajuda local explica esta vista e permanece disponível offline.",
        "scope": "global",
        "sections": [
          {
            "title": "Utilizar esta ajuda",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Abra a ajuda global para navegação e funcionamento offline; a pergunta numa vista explica os respetivos controlos.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          },
          {
            "title": "Texto de teste",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Teste",
                  "Teste"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "Sobre",
        "summary": "Sobre: versão, objetivo e canal de atualização Z00Z.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique a versão atual da demonstração nesta sessão.",
                  "A demonstração JavaScript define o objetivo UX para Rust e Tauri."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A demonstração não transfere nem instala atualizações.",
                  "A aplicação deve verificar um manifesto de versão assinado."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Ativos",
        "summary": "Consulte moedas, tokens e NFTs da carteira selecionada com os saldos locais e o estado dos dados de mercado.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use Todos, Moedas, Tokens ou NFTs para filtrar os ativos da carteira selecionada.",
                  "O saldo pertence à carteira. Valor e Preço ficam indisponíveis sem uma fonte de mercado fiável."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Uma linha abre metadados só de leitura; Enviar e Receber continuam ações separadas.",
                  "Os ícones e esta ajuda estão incluídos localmente e funcionam offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Vales",
        "summary": "Vales explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre vales por ciclo de vida, abra uma linha para as condições ou crie o primeiro vale.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "Permissões",
        "summary": "Permissões explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre direitos sem valor por Detido, Delegado ou Usado e abra uma linha para a autoridade limitada.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Quarentena",
        "summary": "Quarentena: ajuda local para objetos que exigem revisão explícita da carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o motivo, a origem e o estado local indicados antes de qualquer ação.",
                  "Uma ação indisponível permanece bloqueada até a carteira nativa indicar um passo seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A política da carteira nativa toma a decisão final, não esta vista.",
                  "Segredos e dados de transporte privados nunca entram na Ajuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Enviar",
        "summary": "Enviar explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Escolha primeiro Ativos, Vouchers ou Permissões: valor, valor condicional sujeito a política ou autoridade limitada de valor zero.",
                  "Antes da autorização única, reveja o destinatário e o saldo ou política, validade, utilizações restantes, âmbito e delegação."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Receber",
        "summary": "Receber explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Mostre o cartão de receção da carteira e copie o destinatário abreviado para partilha noutro canal.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "Histórico",
        "summary": "Histórico explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Filtre eventos por família de objeto e abra uma linha para o recibo e ciclo técnico.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Fazer staking",
        "summary": "Fazer staking: ajuda local para objetos que exigem revisão explícita da carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o motivo, a origem e o estado local indicados antes de qualquer ação.",
                  "Uma ação indisponível permanece bloqueada até a carteira nativa indicar um passo seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A política da carteira nativa toma a decisão final, não esta vista.",
                  "Segredos e dados de transporte privados nunca entram na Ajuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Retirar staking",
        "summary": "Retirar staking: ajuda local para objetos que exigem revisão explícita da carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o motivo, a origem e o estado local indicados antes de qualquer ação.",
                  "Uma ação indisponível permanece bloqueada até a carteira nativa indicar um passo seguro."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A política da carteira nativa toma a decisão final, não esta vista.",
                  "Segredos e dados de transporte privados nunca entram na Ajuda."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Cópia da carteira",
        "summary": "Cópia da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja data, integridade e destino da última cópia antes de criar uma nova cópia encriptada.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Definições gerais da carteira",
        "summary": "Definições gerais da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Mude apenas o nome da carteira selecionada; o ID e a cadeia de criação permanecem só de leitura.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Segurança da carteira",
        "summary": "Segurança da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Configure o bloqueio por inatividade, bloqueie já ou altere a palavra-passe da carteira selecionada.",
                  "O acesso à frase de recuperação e a rotação da chave principal exigem nova autenticação e confirmação explícita; verifique uma cópia antes da rotação."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Cópia de segurança da carteira",
        "summary": "Cópia de segurança da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A cópia automática, o intervalo, a criação e o restauro aplicam-se apenas à carteira selecionada.",
                  "O restauro valida a integridade antes da substituição. A recuperação apenas pela seed não restaura etiquetas, histórico local, contexto do recetor nem artefactos de divulgação."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Políticas da carteira",
        "summary": "Políticas da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o perfil, limites locais, regras de protocolo bloqueadas e disponibilidade de conformidade.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Definições avançadas da carteira",
        "summary": "Definições avançadas da carteira explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Valide e aplique o rascunho YAML local seguro; segredos e caminhos de ficheiros são excluídos.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Visão geral do Reticulum",
        "summary": "Visão geral do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Visão geral do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Nó do Reticulum",
        "summary": "Nó do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Nó do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Interfaces do Reticulum",
        "summary": "Interfaces do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Interfaces do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Rádio do Reticulum",
        "summary": "Rádio do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Rádio do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Pontos de entrada do Reticulum",
        "summary": "Pontos de entrada do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Pontos de entrada do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Caminhos do Reticulum",
        "summary": "Caminhos do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Caminhos do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Sondas do Reticulum",
        "summary": "Sondas do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Sondas do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Ligações do Reticulum",
        "summary": "Ligações do Reticulum apresenta evidências de transporte só de leitura da ponte Reticulum local registada.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte as evidências de Ligações do Reticulum fornecidas pela ponte local; esta vista não altera o Reticulum.",
                  "Indisponível significa que não existe um snapshot local recente; endereços, destinos, rotas e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "Visão geral do OnionNet",
        "summary": "Visão geral do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Visão geral do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "Época do OnionNet",
        "summary": "Época do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Época do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "Privacidade do OnionNet",
        "summary": "Privacidade do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Privacidade do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "Transporte do OnionNet",
        "summary": "Transporte do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Transporte do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "Filas e repetição do OnionNet",
        "summary": "Filas e repetição do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Filas e repetição do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "Verificação do OnionNet",
        "summary": "Verificação do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Verificação do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "Entrada do OnionNet",
        "summary": "Entrada do OnionNet apresenta agregados OnionNet seguros para a privacidade sem expor rotas ou sessões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte os agregados de Entrada do OnionNet fornecidos pela ponte local; esta vista não altera o OnionNet.",
                  "Indisponível significa que não existe um snapshot local recente; rotas, endpoints, sessões e payloads permanecem ocultos."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Visão geral dos agregadores",
        "summary": "Visão geral dos agregadores apresenta evidências de publicação e colocação só de leitura da ponte local.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte evidências de publicação, colocação, validação e ciclo de vida fornecidas pela ponte local.",
                  "Indisponível significa que não existe um snapshot local recente; a demo não inventa o estado da rede."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Entrada do agregador",
        "summary": "Esta vista explica como o runtime admite uma transação ou claim como trabalho ligado a um digest.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique o contrato `WorkPayload` para `WorkItem` ou `RejectRecord`.",
                  "Indisponível significa que não há snapshot recente de admissão, não que foi aceite ou rejeitado."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ligar um object package altera o digest de admissão e a identidade de entrada.",
                  "Payloads brutos, destinatários, notas e rotas locais da carteira não entram na Ajuda."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Planeamento do agregador",
        "summary": "Esta vista explica a ligação determinística de batch e rota shard sem reivindicar autoridade de settlement.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique modo, geração da rota, contagens de entradas e operações e propriedade dos digests.",
                  "Indisponível significa que nenhum snapshot `BatchPlanned` verificado está ligado."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Configuração, geração, digest da tabela de rotas e plano recalculado devem coincidir.",
                  "O planeamento não finaliza settlement, publicação ou verdade de armazenamento."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Colocação do agregador",
        "summary": "Esta vista explica geração shard, proprietário primário, prontidão dos secundários e linhagem do journal do runtime.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique `ShardPlacementView` sem inferir uma topologia global.",
                  "Indisponível significa que nenhuma observação atual da tabela de colocação está ligada."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A tabela deve possuir exatamente o shard e a geração de routing.",
                  "IDs de agregador são dados operacionais; endpoints e identidades da carteira ficam ocultos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Publicação do agregador",
        "summary": "Esta vista explica como um batch ordenado é ligado a checkpoint, quorum, DA e evidência de ciclo de vida.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Siga `PublicationRequest` para `PublishedBatch` e `PublicationRecord`.",
                  "Indisponível significa que não há publicação ou pacote de readiness verificado."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Dados incompletos ou divergentes de provider, altura, manifesto, payload, statement ou evidence são rejeitados.",
                  "Storage possui as raízes, provas e verdade de ciclo de vida do checkpoint."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Recuperação do agregador",
        "summary": "Esta vista explica verificações de reinício e takeover secundário contra rota, geração, primário e linhagem comprometidos.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique `ShardRecoveryRecord`, intenção, estado durável e ticket de execução.",
                  "Indisponível significa que nenhum snapshot de recuperação comprometido está ligado."
                ]
              }
            ]
          },
          {
            "title": "Limite fail-closed",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Geração, primário, shard, batch, rota ou linhagem incorretos são rejeitados.",
                  "O renderer não pode iniciar failover ou alterar a verdade de recuperação do storage."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "Observadores — Visão geral",
        "summary": "Observadores — Visão geral: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "Observadores — Alertas",
        "summary": "Observadores — Alertas: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "Observadores — Publicação",
        "summary": "Observadores — Publicação: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "Observadores — Fornecedores DA",
        "summary": "Observadores — Fornecedores DA: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "Observadores — Sinais de censura",
        "summary": "Observadores — Sinais de censura: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "Observadores — Exportar evidências",
        "summary": "Observadores — Exportar evidências: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Explorador — Visão geral",
        "summary": "Explorador — Visão geral: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Explorador — Pesquisar",
        "summary": "Explorador — Pesquisar: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Explorador — Pontos de controlo",
        "summary": "Explorador — Pontos de controlo: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Explorador — Lotes",
        "summary": "Explorador — Lotes: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Explorador — Evidência pública",
        "summary": "Explorador — Evidência pública: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Descobrir",
        "summary": "Descobrir: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Instaladas",
        "summary": "Instaladas: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Ligações",
        "summary": "Ligações: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "Permissões",
        "summary": "Permissões: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Troca privada",
        "summary": "Troca privada explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Escolha o ativo de origem detido, o montante e um alvo compatível e reveja a pré-visualização.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Câmbio",
        "summary": "Câmbio explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Escolha Hyperliquid Spot para um livro de ordens ou NEAR Intents para um pedido entre cadeias orientado por solver.",
                  "Reveja par ou rota, destinatário/reembolso, slippage e prazo. Cotação, saída, taxas, endereço de depósito e estado ficam indisponíveis sem conector verificado."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Caixa de entrada",
        "summary": "Caixa de entrada: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.",
                  "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.",
                  "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Enviados",
        "summary": "Enviados: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.",
                  "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.",
                  "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Conversas",
        "summary": "Conversas: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.",
                  "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.",
                  "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Utilização do disco",
        "summary": "Utilização do disco: contadores locais agregados sem dados privados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte recursos sem abrir registos da carteira.",
                  "Os valores apresentados são dados de demonstração determinísticos."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contactos, mensagens, rotas, atividade e segredos são excluídos.",
                  "A aplicação deve obter apenas agregados por uma capacidade nativa limitada."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Utilização da rede",
        "summary": "Utilização da rede: contadores locais agregados sem dados privados.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte recursos sem abrir registos da carteira.",
                  "Os valores apresentados são dados de demonstração determinísticos."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Contactos, mensagens, rotas, atividade e segredos são excluídos.",
                  "A aplicação deve obter apenas agregados por uma capacidade nativa limitada."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Contactos",
        "summary": "Contactos: ajuda sobre etiquetas locais, cartões de receção e revisão explícita da identidade.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja dados locais, validade, revogação e evidência de alteração da identidade.",
                  "Uma etiqueta guardada não prova identidade nem confiança; dados alterados exigem revisão."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os contactos permanecem locais e nunca são publicados como grafo de endereços ou presença.",
                  "Remover um contacto local não revoga direitos externos nem altera a liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Definições gerais",
        "summary": "Definições gerais explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Escolha o idioma, formato regional, fuso horário de apresentação e preferência de notificações.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Notificações",
        "summary": "Notificações: preferências locais de notificação, vibração e toque.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ative as notificações antes de escolher vibração e toque.",
                  "As opções dependentes ficam desativadas quando as notificações estão desligadas."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A demonstração não pede permissões do sistema.",
                  "A aplicação deve indicar claramente som ou vibração indisponíveis."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Aparência",
        "summary": "Aparência explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Alterne Claro ou Escuro, escolha uma paleta e o tema local de realce YAML.",
                  "Os estados indisponível, só de leitura e pendente são mostrados claramente."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os segredos da carteira e os dados privados de transporte não entram na ajuda.",
                  "Esta ajuda está incluída na aplicação e funciona offline."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Detalhes do ativo",
        "summary": "Consulte a identidade, o emissor, a oferta e a classificação local do ativo selecionado.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Nome e símbolo identificam o ativo; Proprietário e ID do ativo indicam a origem declarada.",
                  "A oferta atual e máxima fica indisponível sem uma fonte local autoritativa."
                ]
              }
            ]
          },
          {
            "title": "Funcionamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os campos são só de leitura e não provam valor de mercado, propriedade ou confiança no protocolo.",
                  "O ícone, os metadados e esta ajuda são locais e funcionam offline."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps — detalhes",
        "summary": "dApps — detalhes: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps — revisão da permissão",
        "summary": "dApps — revisão da permissão: ajuda sobre a pré-visualização local limitada de dApps e o respetivo limite de permissões.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja descritores locais, intenções limitadas e resultados explícitos.",
                  "Antes de aceitar, reveja âmbito, utilizações, validade, valor, taxa, divulgação e revogação."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps é uma pré-visualização do roteiro: não executa código remoto, URL arbitrários nem assinatura genérica.",
                  "A carteira revalida cada intenção aceite; esta vista não altera objetos da carteira."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Mensageiro — detalhes",
        "summary": "Mensageiro — detalhes: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.",
                  "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.",
                  "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Mensageiro — revisão do pedido",
        "summary": "Mensageiro — revisão do pedido: ajuda sobre a pré-visualização privada de coordenação de pedidos e passagem para a carteira.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja mensagens, pedidos, recibos, validade e estados de recuperação locais.",
                  "Aceitar cria uma intenção para revisão na carteira, mas não liquida nem altera o estado."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger é uma pré-visualização do roteiro para retransmissão temporária, não conversa permanente na cadeia.",
                  "Abrir, eliminar, bloquear ou denunciar nunca altera o estado de liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Contactos — detalhes",
        "summary": "Contactos — detalhes: ajuda sobre etiquetas locais, cartões de receção e revisão explícita da identidade.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja dados locais, validade, revogação e evidência de alteração da identidade.",
                  "Uma etiqueta guardada não prova identidade nem confiança; dados alterados exigem revisão."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os contactos permanecem locais e nunca são publicados como grafo de endereços ou presença.",
                  "Remover um contacto local não revoga direitos externos nem altera a liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Contactos — revisão da identidade",
        "summary": "Contactos — revisão da identidade: ajuda sobre etiquetas locais, cartões de receção e revisão explícita da identidade.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja dados locais, validade, revogação e evidência de alteração da identidade.",
                  "Uma etiqueta guardada não prova identidade nem confiança; dados alterados exigem revisão."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Os contactos permanecem locais e nunca são publicados como grafo de endereços ou presença.",
                  "Remover um contacto local não revoga direitos externos nem altera a liquidação da carteira."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "Observadores — detalhes do alerta",
        "summary": "Observadores — detalhes do alerta: ajuda sobre a pré-visualização Watchers só de leitura e o limite de evidência pública.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consulte dados determinísticos de publicação sem alterar o estado da rede.",
                  "Estados indisponíveis, desatualizados, inválidos e de erro continuam explícitos e falham com segurança."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers é uma pré-visualização do roteiro com dados locais, não uma função de protocolo disponibilizada.",
                  "Etiquetas da carteira, contrapartes, rotas, mensagens e segredos nunca são expostos."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Explorador — detalhes",
        "summary": "Explorador — detalhes: ajuda sobre a pré-visualização privada do Explorer para identificadores públicos suportados.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Use apenas identificadores públicos suportados de pontos de controlo, lotes, alertas ou evidência.",
                  "Identificadores desconhecidos, privados, inválidos ou indisponíveis falham sem consultar a carteira."
                ]
              }
            ]
          },
          {
            "title": "Comportamento local e seguro",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer é uma pré-visualização do roteiro com dados locais, não um serviço de dados da carteira.",
                  "Saldos, contactos, mensagens, notas, rotas e segredos locais nunca entram no Explorer."
                ]
              }
            ]
          }
        ]
      }
    },
    "ko": {
      "app": {
        "id": "app",
        "title": "애플리케이션 도움말",
        "summary": "로컬 도움말은 이 화면을 설명하며 오프라인에서도 사용할 수 있습니다.",
        "scope": "global",
        "sections": [
          {
            "title": "이 도움말 사용",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "전체 도움말에서 앱 탐색과 오프라인 동작을 확인하고 각 화면의 물음표로 해당 컨트롤을 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          },
          {
            "title": "테스트 텍스트",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "테스트",
                  "테스트"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "정보",
        "summary": "정보: Z00Z 버전, 목적 및 업데이트 채널입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "이 세션의 현재 데모 버전을 확인하세요.",
                  "JavaScript 데모는 Rust 및 Tauri UX 목표를 정의합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "데모는 업데이트를 다운로드하거나 설치하지 않습니다.",
                  "패키지 앱은 서명된 릴리스 매니페스트를 확인해야 합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "자산",
        "summary": "선택한 지갑의 코인, 토큰, NFT를 로컬 잔액 및 시세 데이터 상태와 함께 확인합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "전체, 코인, 토큰, NFT로 선택한 지갑의 자산 목록을 좁힙니다.",
                  "잔액은 지갑 소유 데이터입니다. 신뢰할 수 있는 시세 피드가 없으면 가치와 가격은 사용할 수 없음으로 남습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "행을 선택하면 읽기 전용 자산 메타데이터가 열리며 보내기와 받기는 별도의 지갑 작업입니다.",
                  "자산 아이콘과 도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "바우처",
        "summary": "바우처 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "수명 주기로 바우처를 필터링하고 조건을 열거나 첫 바우처를 만듭니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "권한",
        "summary": "권한 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "0가치 권한을 보유·위임·사용으로 필터링하고 행을 열어 제한된 권한을 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "격리",
        "summary": "격리: 지갑의 명시적 검토가 필요한 객체에 대한 로컬 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "작업하기 전에 표시된 이유, 출처, 로컬 상태를 확인하세요.",
                  "네이티브 지갑이 안전한 다음 단계를 제공할 때까지 사용할 수 없는 작업은 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "최종 결정은 이 화면이 아니라 네이티브 지갑 정책이 내립니다.",
                  "비밀 정보와 비공개 전송 데이터는 도움말에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "보내기",
        "summary": "보내기 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "먼저 자산, 바우처 또는 권한을 선택합니다. 각각 가치, 정책 기반 조건부 가치, 가치가 0인 제한된 권한입니다.",
                  "한 번 승인하기 전에 수신자와 잔액 또는 정책, 만료, 남은 사용 횟수, 범위, 위임 조건을 확인합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "받기",
        "summary": "받기 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "선택한 지갑의 수신 카드를 표시하고 축약된 수신자를 별도 채널로 공유합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "기록",
        "summary": "기록 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "객체 종류로 지갑 이벤트를 필터링하고 행을 열어 영수증과 기술 수명 주기를 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "스테이킹",
        "summary": "스테이킹: 지갑의 명시적 검토가 필요한 객체에 대한 로컬 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "작업하기 전에 표시된 이유, 출처, 로컬 상태를 확인하세요.",
                  "네이티브 지갑이 안전한 다음 단계를 제공할 때까지 사용할 수 없는 작업은 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "최종 결정은 이 화면이 아니라 네이티브 지갑 정책이 내립니다.",
                  "비밀 정보와 비공개 전송 데이터는 도움말에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "언스테이킹",
        "summary": "언스테이킹: 지갑의 명시적 검토가 필요한 객체에 대한 로컬 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "작업하기 전에 표시된 이유, 출처, 로컬 상태를 확인하세요.",
                  "네이티브 지갑이 안전한 다음 단계를 제공할 때까지 사용할 수 없는 작업은 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "최종 결정은 이 화면이 아니라 네이티브 지갑 정책이 내립니다.",
                  "비밀 정보와 비공개 전송 데이터는 도움말에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "지갑 백업",
        "summary": "지갑 백업 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "새 암호화 백업을 만들기 전에 최근 로컬 백업의 날짜, 무결성과 대상을 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "지갑 일반 설정",
        "summary": "지갑 일반 설정 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "선택한 지갑만 이름을 바꿀 수 있으며 지갑 ID와 생성 시 체인은 읽기 전용입니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "지갑 보안",
        "summary": "지갑 보안 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "비활성 잠금을 설정하거나 즉시 잠그거나 선택한 지갑의 비밀번호를 변경합니다.",
                  "복구 문구 확인과 마스터 키 교체에는 재인증과 명시적 확인이 필요하며, 교체 전에 백업을 검증해야 합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "지갑 백업",
        "summary": "지갑 백업 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "자동 백업, 간격, 생성 및 복원 설정은 선택한 지갑에만 적용됩니다.",
                  "복원은 교체 전에 무결성을 검증합니다. 시드만으로 복구하면 레이블, 로컬 기록, 수신자 컨텍스트 및 공개 아티팩트가 복원되지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "지갑 정책",
        "summary": "지갑 정책 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "이 지갑의 프로필, 로컬 지출 제한, 잠긴 프로토콜 규칙과 규정 준수 가능 여부를 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "지갑 고급 설정",
        "summary": "지갑 고급 설정 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "선택한 지갑의 안전한 로컬 YAML 초안을 검증하고 적용합니다. 비밀과 파일 경로는 제외됩니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum 개요",
        "summary": "Reticulum 개요는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 개요 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulum 노드",
        "summary": "Reticulum 노드는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 노드 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulum 인터페이스",
        "summary": "Reticulum 인터페이스는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 인터페이스 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum 라디오",
        "summary": "Reticulum 라디오는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 라디오 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulum 진입점",
        "summary": "Reticulum 진입점는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 진입점 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulum 경로",
        "summary": "Reticulum 경로는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 경로 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulum 프로브",
        "summary": "Reticulum 프로브는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 프로브 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulum 링크",
        "summary": "Reticulum 링크는 등록된 로컬 Reticulum 브리지의 읽기 전용 캐리어 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 Reticulum 링크 증거를 확인하세요. 이 화면은 Reticulum을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 주소, 목적지, 경로, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet 개요",
        "summary": "OnionNet 개요는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 개요 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNet 에포크",
        "summary": "OnionNet 에포크는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 에포크 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNet 개인정보 보호",
        "summary": "OnionNet 개인정보 보호는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 개인정보 보호 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNet 전송",
        "summary": "OnionNet 전송는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 전송 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNet 대기열 및 재생",
        "summary": "OnionNet 대기열 및 재생는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 대기열 및 재생 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet 검사",
        "summary": "OnionNet 검사는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 검사 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNet 수신",
        "summary": "OnionNet 수신는 경로나 세션을 노출하지 않는 개인정보 보호 OnionNet 집계를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 OnionNet 수신 집계를 확인하세요. 이 화면은 OnionNet을 변경하지 않습니다.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 경로, 엔드포인트, 세션 ID, 페이로드는 숨겨집니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "애그리게이터 개요",
        "summary": "애그리게이터 개요는 로컬 브리지의 읽기 전용 게시 및 배치 증거를 표시합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 브리지가 제공한 게시, 배치, 검증, 수명 주기 증거를 확인하세요.",
                  "사용 불가는 최신 로컬 스냅샷이 없다는 뜻이며 데모는 네트워크 상태를 추정하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "애그리게이터 인그레스",
        "summary": "런타임이 트랜잭션 또는 클레임 payload를 digest에 바인딩된 작업 항목으로 승인하는 방식을 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`WorkPayload`에서 `WorkItem` 또는 `RejectRecord`로 이어지는 계약을 확인하세요.",
                  "사용 불가는 최신 승인 스냅샷이 없다는 뜻이며 승인이나 거부를 뜻하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 경계",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Object package 바인딩은 admission digest와 intake identity를 변경합니다.",
                  "Raw payload, 수신자, 메모와 지갑 로컬 경로는 도움말에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "애그리게이터 계획",
        "summary": "결제 권한을 주장하지 않고 결정론적 batch와 shard route 바인딩을 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Planner mode, route generation, intake와 operation 수, digest 소유권을 확인하세요.",
                  "사용 불가는 검증된 `BatchPlanned` 스냅샷이 연결되지 않았다는 뜻입니다."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 경계",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "설정, generation, route-table digest와 재계산된 plan은 일치해야 합니다.",
                  "계획은 settlement, publication 또는 storage truth를 확정하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "애그리게이터 배치",
        "summary": "런타임 소유의 shard generation, primary, secondary 준비 상태와 journal lineage를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "글로벌 토폴로지를 추정하지 말고 `ShardPlacementView` 계약을 확인하세요.",
                  "사용 불가는 현재 placement table 관측이 연결되지 않았다는 뜻입니다."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 경계",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Placement table은 정확한 shard와 routing generation을 소유해야 합니다.",
                  "Aggregator ID는 운영 데이터이며 endpoint와 지갑 identity는 숨겨집니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "애그리게이터 게시",
        "summary": "정렬된 batch가 checkpoint, quorum, DA 및 lifecycle evidence에 바인딩되는 방식을 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`PublicationRequest`에서 `PublishedBatch`와 `PublicationRecord`로 이어지는 흐름을 확인하세요.",
                  "사용 불가는 검증된 게시 또는 readiness bundle이 연결되지 않았다는 뜻입니다."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 경계",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Provider, height, manifest, payload, statement 또는 evidence가 불완전하거나 다르면 거부됩니다.",
                  "Storage가 checkpoint root, proof와 lifecycle truth를 소유합니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "애그리게이터 복구",
        "summary": "커밋된 route, generation, primary와 journal lineage에 대한 재시작 및 secondary takeover 검사를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`ShardRecoveryRecord`, recovery intent, durable state와 execution ticket을 확인하세요.",
                  "사용 불가는 커밋된 복구 스냅샷이 연결되지 않았다는 뜻입니다."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 경계",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Generation, primary, shard, batch, route 또는 lineage가 틀리면 거부됩니다.",
                  "Renderer는 failover를 시작하거나 storage recovery truth를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "감시자 · 개요",
        "summary": "감시자 · 개요: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "감시자 · 알림",
        "summary": "감시자 · 알림: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "감시자 · 게시",
        "summary": "감시자 · 게시: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "감시자 · DA 제공자",
        "summary": "감시자 · DA 제공자: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "감시자 · 검열 신호",
        "summary": "감시자 · 검열 신호: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "감시자 · 증거 내보내기",
        "summary": "감시자 · 증거 내보내기: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "탐색기 · 개요",
        "summary": "탐색기 · 개요: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "탐색기 · 검색",
        "summary": "탐색기 · 검색: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "탐색기 · 검사점",
        "summary": "탐색기 · 검사점: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "탐색기 · 배치",
        "summary": "탐색기 · 배치: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "탐색기 · 공개 증거",
        "summary": "탐색기 · 공개 증거: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "찾아보기",
        "summary": "찾아보기: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "설치됨",
        "summary": "설치됨: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "연결",
        "summary": "연결: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "권한",
        "summary": "권한: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "비공개 스왑",
        "summary": "비공개 스왑 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "보유한 원본 자산, 금액과 호환 대상 자산을 선택하고 제출 전에 미리보기를 확인합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "거래소",
        "summary": "거래소 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "오더북 요청은 Hyperliquid Spot, solver 기반 크로스체인 요청은 NEAR Intents를 선택합니다.",
                  "페어 또는 경로, 수신자/환불, 슬리피지와 기한을 확인합니다. 검증된 커넥터 없이는 견적, 출력, 수수료, 입금 주소, 실행 상태가 제공되지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "받은편지함",
        "summary": "받은편지함: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.",
                  "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.",
                  "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "보낸편지함",
        "summary": "보낸편지함: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.",
                  "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.",
                  "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "대화",
        "summary": "대화: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.",
                  "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.",
                  "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "디스크 사용량",
        "summary": "디스크 사용량: 비공개 데이터가 없는 집계 로컬 카운터입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 기록을 열지 않고 리소스 사용량을 확인하세요.",
                  "표시 값은 결정적 데모 데이터입니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "연락처, 메시지, 경로, 활동 및 비밀은 제외됩니다.",
                  "패키지 앱은 제한된 네이티브 기능으로 집계 값만 가져와야 합니다."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "네트워크 사용량",
        "summary": "네트워크 사용량: 비공개 데이터가 없는 집계 로컬 카운터입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 기록을 열지 않고 리소스 사용량을 확인하세요.",
                  "표시 값은 결정적 데모 데이터입니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "연락처, 메시지, 경로, 활동 및 비밀은 제외됩니다.",
                  "패키지 앱은 제한된 네이티브 기능으로 집계 값만 가져와야 합니다."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "연락처",
        "summary": "연락처: 로컬 연락처 레이블, 수신자 카드, 명시적 신원 변경 검토에 대한 도움말입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 연락처 데이터, 만료, 취소, 신원 변경 증거를 검토하세요.",
                  "저장된 레이블은 신원이나 신뢰를 증명하지 않으며 변경된 데이터는 검토해야 합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "연락처는 로컬에 유지되며 주소 또는 온라인 상태 그래프로 업로드되지 않습니다.",
                  "로컬 연락처 삭제는 외부 권한을 취소하거나 지갑 결제를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "일반 설정",
        "summary": "일반 설정 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "앱 언어, 지역 형식, 표시 시간대와 알림 환경설정을 선택합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "알림",
        "summary": "알림: 알림, 진동 및 벨소리의 로컬 설정입니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "진동과 벨소리를 선택하기 전에 알림을 켜세요.",
                  "알림이 꺼지면 관련 선택 항목도 비활성화됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "데모는 운영 체제 권한을 요청하지 않습니다.",
                  "패키지 앱은 소리나 진동을 사용할 수 없을 때 명확히 알려야 합니다."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "모양",
        "summary": "모양 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "라이트 또는 다크 모드, 팔레트와 로컬 YAML 강조 테마를 선택합니다.",
                  "사용 불가, 읽기 전용, 대기 상태를 명확하게 표시합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지갑 비밀과 개인 전송 데이터는 도움말에 포함되지 않습니다.",
                  "도움말은 앱에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "자산 세부 정보",
        "summary": "선택한 자산의 식별 정보, 발행자, 공급량, 로컬 분류를 확인합니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "이름과 티커는 자산을 식별하고 소유자와 자산 ID는 선언된 출처를 나타냅니다.",
                  "신뢰할 수 있는 로컬 출처가 없으면 현재 및 최대 공급량은 사용할 수 없음으로 표시됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "이 필드는 읽기 전용이며 시장 가치, 소유권 또는 프로토콜 신뢰를 증명하지 않습니다.",
                  "아이콘, 메타데이터, 도움말은 로컬에 포함되어 오프라인에서도 작동합니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps · 세부 정보",
        "summary": "dApps · 세부 정보: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps · 권한 검토",
        "summary": "dApps · 권한 검토: 제한된 로컬 dApps 미리보기와 권한 경계에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 설명자, 범위가 제한된 인텐트, 명시적 결과를 검토하세요.",
                  "수락 전에 범위, 사용 횟수, 만료, 금액, 수수료, 공개, 취소 조건을 확인하세요."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps는 로드맵 미리보기이며 원격 코드, 임의 URL, 범용 서명을 실행하지 않습니다.",
                  "수락한 인텐트는 지갑이 다시 검증하며 이 화면은 지갑 객체를 변경할 수 없습니다."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "메신저 · 세부 정보",
        "summary": "메신저 · 세부 정보: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.",
                  "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.",
                  "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "메신저 · 요청 검토",
        "summary": "메신저 · 요청 검토: 비공개 요청 조정 미리보기와 지갑 전달에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 메시지, 요청, 영수증, 만료, 복구 상태를 검토하세요.",
                  "요청 수락은 지갑 검토 인텐트를 만들 뿐 결제하거나 지갑 상태를 변경하지 않습니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger는 단기 릴레이용 로드맵 미리보기이며 영구 온체인 채팅이 아닙니다.",
                  "열기, 삭제, 차단, 신고는 지갑 결제 상태를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "연락처 · 세부 정보",
        "summary": "연락처 · 세부 정보: 로컬 연락처 레이블, 수신자 카드, 명시적 신원 변경 검토에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 연락처 데이터, 만료, 취소, 신원 변경 증거를 검토하세요.",
                  "저장된 레이블은 신원이나 신뢰를 증명하지 않으며 변경된 데이터는 검토해야 합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "연락처는 로컬에 유지되며 주소 또는 온라인 상태 그래프로 업로드되지 않습니다.",
                  "로컬 연락처 삭제는 외부 권한을 취소하거나 지갑 결제를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "연락처 · 신원 검토",
        "summary": "연락처 · 신원 검토: 로컬 연락처 레이블, 수신자 카드, 명시적 신원 변경 검토에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 연락처 데이터, 만료, 취소, 신원 변경 증거를 검토하세요.",
                  "저장된 레이블은 신원이나 신뢰를 증명하지 않으며 변경된 데이터는 검토해야 합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "연락처는 로컬에 유지되며 주소 또는 온라인 상태 그래프로 업로드되지 않습니다.",
                  "로컬 연락처 삭제는 외부 권한을 취소하거나 지갑 결제를 변경하지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "감시자 · 알림 세부 정보",
        "summary": "감시자 · 알림 세부 정보: 읽기 전용 Watchers 로드맵 미리보기와 공개 증거 경계에 대한 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "네트워크 상태를 변경하지 않고 결정론적 게시 데이터를 검토하세요.",
                  "사용 불가, 오래됨, 잘못된 형식, 오류 상태는 명확하게 표시되고 안전하게 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers는 로컬 픽스처 기반 로드맵 미리보기이며 출시된 프로토콜 기능이 아닙니다.",
                  "지갑 레이블, 상대방, 경로, 메시지, 비밀 정보는 노출되지 않습니다."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "탐색기 · 세부 정보",
        "summary": "탐색기 · 세부 정보: 지원되는 공개 식별자만 다루는 개인정보 보호형 Explorer 미리보기 도움말입니다.",
        "scope": "dialog",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "지원되는 공개 검사점, 배치, 알림 또는 증거 식별자만 사용하세요.",
                  "알 수 없거나 비공개이거나 잘못되었거나 사용할 수 없는 식별자는 지갑 조회 없이 차단됩니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer는 로컬 픽스처 기반 로드맵 미리보기이며 지갑 데이터 서비스가 아닙니다.",
                  "로컬 잔액, 연락처, 메시지, 메모, 경로, 비밀 정보는 Explorer에 포함되지 않습니다."
                ]
              }
            ]
          }
        ]
      }
    },
    "tr": {
      "app": {
        "id": "app",
        "title": "Uygulama yardımı",
        "summary": "Yerel yardım bu görünümü açıklar ve çevrimdışı kullanılabilir.",
        "scope": "global",
        "sections": [
          {
            "title": "Bu yardımı kullanma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Gezinme ve çevrimdışı çalışma için genel Yardım’ı; ekran denetimleri için o ekrandaki soru düğmesini açın.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          },
          {
            "title": "Test metni",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Test",
                  "Test"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "Hakkında",
        "summary": "Hakkında: Z00Z sürümü, amacı ve güncelleme kanalı.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Bu oturumun güncel demo sürümünü kontrol edin.",
                  "JavaScript demosu Rust ve Tauri UX hedefini tanımlar."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Demo güncelleme indirmez veya kurmaz.",
                  "Paket uygulama imzalı sürüm bildirimini doğrulamalıdır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "Varlıklar",
        "summary": "Seçili cüzdanın coin, token ve NFT’lerini yerel bakiyeleri ve piyasa verisi durumuyla inceleyin.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Tümü, Coinler, Tokenlar veya NFT’ler ile seçili cüzdanın varlık listesini daraltın.",
                  "Bakiye cüzdana aittir. Güvenilir piyasa akışı yoksa Değer ve Fiyat kullanılamaz kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Bir satır salt okunur varlık verisini açar; Gönder ve Al ayrı cüzdan işlemleridir.",
                  "Varlık simgeleri ve Yardım uygulamada yereldir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "Kuponlar",
        "summary": "Kuponlar, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kuponları yaşam döngüsüne göre filtreleyin, koşulları açın veya ilk kuponu oluşturun.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "İzinler",
        "summary": "İzinler, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Sıfır değerli hakları Tutulan, Devredilen veya Kullanılan olarak filtreleyip sınırlı yetkiyi açın.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "Karantina",
        "summary": "Karantina: açık cüzdan incelemesi gerektiren nesneler için yerel yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Herhangi bir işlemden önce belirtilen nedeni, kaynağı ve yerel durumu inceleyin.",
                  "Yerel cüzdan güvenli bir sonraki adım sunana kadar kullanılamayan işlem engelli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Son kararı bu görünüm değil, yerel cüzdan ilkesi verir.",
                  "Gizli bilgiler ve özel aktarım verileri Yardım’a asla girmez."
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "Gönder",
        "summary": "Gönder, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Önce Varlıklar, Kuponlar veya İzinler’i seçin: değer, politikaya bağlı koşullu değer veya sıfır değerli sınırlı yetki.",
                  "Tek seferlik yetkilendirmeden önce alıcıyı ve bakiye ya da politika, süre, kalan kullanım, kapsam ve devri inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "Al",
        "summary": "Al, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Seçili cüzdanın Alıcı Kartını gösterin ve kısaltılmış alıcıyı ayrı bir kanaldan paylaşın.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "Geçmiş",
        "summary": "Geçmiş, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan olaylarını nesne ailesine göre filtreleyin ve makbuz ile teknik yaşam döngüsü için satırı açın.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "Stake et",
        "summary": "Stake et: açık cüzdan incelemesi gerektiren nesneler için yerel yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Herhangi bir işlemden önce belirtilen nedeni, kaynağı ve yerel durumu inceleyin.",
                  "Yerel cüzdan güvenli bir sonraki adım sunana kadar kullanılamayan işlem engelli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Son kararı bu görünüm değil, yerel cüzdan ilkesi verir.",
                  "Gizli bilgiler ve özel aktarım verileri Yardım’a asla girmez."
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "Stake’den çıkar",
        "summary": "Stake’den çıkar: açık cüzdan incelemesi gerektiren nesneler için yerel yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Herhangi bir işlemden önce belirtilen nedeni, kaynağı ve yerel durumu inceleyin.",
                  "Yerel cüzdan güvenli bir sonraki adım sunana kadar kullanılamayan işlem engelli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Son kararı bu görünüm değil, yerel cüzdan ilkesi verir.",
                  "Gizli bilgiler ve özel aktarım verileri Yardım’a asla girmez."
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "Cüzdan yedeği",
        "summary": "Cüzdan yedeği, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yeni şifreli yedek oluşturmadan önce son yerel yedeğin tarihini, bütünlüğünü ve hedefini inceleyin.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "Genel cüzdan ayarları",
        "summary": "Genel cüzdan ayarları, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnız seçili cüzdanı yeniden adlandırın; cüzdan kimliği ve oluşturma zinciri salt okunurdur.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "Cüzdan güvenliği",
        "summary": "Cüzdan güvenliği, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Hareketsizlik kilidini ayarlayın, hemen kilitleyin veya seçili cüzdanın parolasını değiştirin.",
                  "Kurtarma ifadesine erişim ve ana anahtar döndürme yeniden kimlik doğrulama ve açık onay gerektirir; döndürmeden önce yedeği doğrulayın."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "Cüzdan yedeği",
        "summary": "Cüzdan yedeği, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Otomatik yedekleme, aralık, oluşturma ve geri yükleme yalnızca seçili cüzdana uygulanır.",
                  "Geri yükleme, değiştirmeden önce bütünlüğü doğrular. Yalnızca seed ile kurtarma; etiketleri, yerel geçmişi, alıcı bağlamını veya açıklama yapıtlarını geri getirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "Cüzdan politikaları",
        "summary": "Cüzdan politikaları, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Bu cüzdanın profilini, yerel harcama sınırlarını, kilitli protokol kurallarını ve uyumluluk durumunu inceleyin.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "Gelişmiş cüzdan ayarları",
        "summary": "Gelişmiş cüzdan ayarları, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Seçili cüzdanın güvenli yerel YAML taslağını doğrulayıp uygulayın; sırlar ve dosya yolları hariçtir.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum genel bakış",
        "summary": "Reticulum genel bakış, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum genel bakış kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulum düğümü",
        "summary": "Reticulum düğümü, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum düğümü kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulum arayüzleri",
        "summary": "Reticulum arayüzleri, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum arayüzleri kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum radyosu",
        "summary": "Reticulum radyosu, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum radyosu kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulum giriş noktaları",
        "summary": "Reticulum giriş noktaları, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum giriş noktaları kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulum yolları",
        "summary": "Reticulum yolları, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum yolları kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulum probları",
        "summary": "Reticulum probları, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum probları kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulum bağlantıları",
        "summary": "Reticulum bağlantıları, kayıtlı yerel Reticulum köprüsünden salt okunur taşıyıcı kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı Reticulum bağlantıları kanıtını inceleyin; bu görünüm Reticulum durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; adresler, hedefler, rotalar ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet genel bakış",
        "summary": "OnionNet genel bakış, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet genel bakış toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNet dönemi",
        "summary": "OnionNet dönemi, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet dönemi toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNet gizliliği",
        "summary": "OnionNet gizliliği, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet gizliliği toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNet taşıması",
        "summary": "OnionNet taşıması, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet taşıması toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNet kuyrukları ve yeniden oynatma",
        "summary": "OnionNet kuyrukları ve yeniden oynatma, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet kuyrukları ve yeniden oynatma toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet denetimi",
        "summary": "OnionNet denetimi, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet denetimi toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNet girişi",
        "summary": "OnionNet girişi, rota veya oturumları açığa çıkarmadan gizliliği koruyan OnionNet toplamlarını sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı OnionNet girişi toplamlarını inceleyin; bu görünüm OnionNet durumunu değiştirmez.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; rotalar, uç noktalar, oturum kimlikleri ve yükler gizli kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "Toplayıcılara genel bakış",
        "summary": "Toplayıcılara genel bakış, yerel köprüden salt okunur yayın ve yerleştirme kanıtı sunar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel köprünün sağladığı yayın, yerleştirme, doğrulama ve yaşam döngüsü kanıtını inceleyin.",
                  "Kullanılamaz, güncel yerel anlık görüntü olmadığı anlamına gelir; demo ağ durumunu uydurmaz."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "Toplayıcı girişi",
        "summary": "Runtime’ın bir işlem veya claim payload’unu digest bağlı bir iş öğesi olarak nasıl kabul ettiğini açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`WorkPayload` → `WorkItem` veya `RejectRecord` sözleşmesini inceleyin.",
                  "Kullanılamaz, güncel kabul snapshot’ı olmadığı anlamına gelir; kabul veya ret anlamına gelmez."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed sınırı",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Object package bağlama admission digest ve intake identity değerini değiştirir.",
                  "Ham payload, alıcı, memo ve cüzdanın yerel rotaları Yardım içine girmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "Toplayıcı planlama",
        "summary": "Settlement yetkisi iddia etmeden deterministik batch ve shard route bağlamasını açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Planner mode, route generation, intake ve operation sayıları ile digest sahipliğini inceleyin.",
                  "Kullanılamaz, doğrulanmış `BatchPlanned` snapshot’ı bağlı olmadığı anlamına gelir."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed sınırı",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yapılandırma, generation, route-table digest ve yeniden hesaplanan plan eşleşmelidir.",
                  "Planlama settlement, publication veya storage truth değerini kesinleştirmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "Toplayıcı yerleşimi",
        "summary": "Runtime’a ait shard generation, primary owner, secondary readiness ve journal lineage görünümünü açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Küresel topoloji çıkarmadan `ShardPlacementView` sözleşmesini inceleyin.",
                  "Kullanılamaz, güncel placement table gözlemi bağlı olmadığı anlamına gelir."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed sınırı",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Tablo tam shard ve routing generation değerine sahip olmalıdır.",
                  "Aggregator ID operasyonel veridir; endpoint ve cüzdan kimlikleri gizli kalır."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "Toplayıcı yayını",
        "summary": "Sıralı batch’in checkpoint, quorum, DA ve lifecycle evidence ile nasıl bağlandığını açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`PublicationRequest` → `PublishedBatch` → `PublicationRecord` akışını izleyin.",
                  "Kullanılamaz, doğrulanmış publication veya readiness bundle bağlı olmadığı anlamına gelir."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed sınırı",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Eksik ya da uyuşmayan provider, height, manifest, payload, statement veya evidence reddedilir.",
                  "Checkpoint root, proof ve lifecycle truth Storage’a aittir."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "Toplayıcı kurtarma",
        "summary": "Bağlı route, generation, primary ve journal lineage karşısında restart ve secondary takeover kontrollerini açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`ShardRecoveryRecord`, recovery intent, durable state ve execution ticket sözleşmesini inceleyin.",
                  "Kullanılamaz, bağlı committed recovery snapshot olmadığı anlamına gelir."
                ]
              }
            ]
          },
          {
            "title": "Fail-closed sınırı",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yanlış generation, primary, shard, batch, route veya lineage reddedilir.",
                  "Renderer failover başlatamaz veya Storage recovery truth değerini değiştiremez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "İzleyiciler — Genel bakış",
        "summary": "İzleyiciler — Genel bakış: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "İzleyiciler — Uyarılar",
        "summary": "İzleyiciler — Uyarılar: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "İzleyiciler — Yayınlama",
        "summary": "İzleyiciler — Yayınlama: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "İzleyiciler — DA sağlayıcıları",
        "summary": "İzleyiciler — DA sağlayıcıları: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "İzleyiciler — Sansür sinyalleri",
        "summary": "İzleyiciler — Sansür sinyalleri: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "İzleyiciler — Kanıt dışa aktarma",
        "summary": "İzleyiciler — Kanıt dışa aktarma: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "Gezgin — Genel bakış",
        "summary": "Gezgin — Genel bakış: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "Gezgin — Ara",
        "summary": "Gezgin — Ara: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "Gezgin — Kontrol noktaları",
        "summary": "Gezgin — Kontrol noktaları: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "Gezgin — Partiler",
        "summary": "Gezgin — Partiler: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "Gezgin — Genel kanıt",
        "summary": "Gezgin — Genel kanıt: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "Keşfet",
        "summary": "Keşfet: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "Yüklü",
        "summary": "Yüklü: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "Bağlantılar",
        "summary": "Bağlantılar: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "İzinler",
        "summary": "İzinler: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "Özel takas",
        "summary": "Özel takas, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Eldeki kaynak varlığı, tutarı ve uyumlu hedefi seçin; göndermeden önce önizlemeyi inceleyin.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "Borsa",
        "summary": "Borsa, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Emir defteri isteği için Hyperliquid Spot’u, solver tabanlı zincirler arası istek için NEAR Intents’i seçin.",
                  "Çift veya rota, alıcı/iade, kayma ve süreyi inceleyin. Doğrulanmış bağlayıcı olmadan teklif, çıktı, ücret, yatırma adresi ve durum kullanılamaz kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "Gelen kutusu",
        "summary": "Gelen kutusu: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.",
                  "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.",
                  "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "Gönderilenler",
        "summary": "Gönderilenler: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.",
                  "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.",
                  "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "Konuşmalar",
        "summary": "Konuşmalar: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.",
                  "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.",
                  "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "Disk Kullanımı",
        "summary": "Disk Kullanımı: özel veri içermeyen toplu yerel sayaçlar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan kayıtlarını açmadan kaynak kullanımını inceleyin.",
                  "Gösterilen değerler belirlenimci demo verileridir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kişiler, mesajlar, rotalar, etkinlik ve gizli bilgiler hariç tutulur.",
                  "Paket uygulama yalnızca sınırlı yerel yetenekten toplu değer almalıdır."
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "Ağ Kullanımı",
        "summary": "Ağ Kullanımı: özel veri içermeyen toplu yerel sayaçlar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan kayıtlarını açmadan kaynak kullanımını inceleyin.",
                  "Gösterilen değerler belirlenimci demo verileridir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kişiler, mesajlar, rotalar, etkinlik ve gizli bilgiler hariç tutulur.",
                  "Paket uygulama yalnızca sınırlı yerel yetenekten toplu değer almalıdır."
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "Kişiler",
        "summary": "Kişiler: yerel kişi etiketleri, alıcı kartları ve açık kimlik değişikliği incelemesi hakkında yardım.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel kişi verilerini, süreyi, iptali ve kimlik değişikliği kanıtını inceleyin.",
                  "Kaydedilmiş etiket kimliği veya güveni kanıtlamaz; değişen veriler açıkça incelenmelidir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kişiler yerel kalır ve adres ya da çevrim içi durum grafiği olarak yüklenmez.",
                  "Yerel kişiyi kaldırmak dış hakları iptal etmez veya Cüzdan ödemesini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "Genel ayarlar",
        "summary": "Genel ayarlar, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Uygulama dilini, bölgesel biçimi, görüntüleme saat dilimini ve bildirim tercihini seçin.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "Bildirimler",
        "summary": "Bildirimler: yerel bildirim, titreşim ve zil sesi tercihleri.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Titreşim ve zil sesi seçmeden önce bildirimleri açın.",
                  "Bildirimler kapalıyken bağlı seçenekler devre dışıdır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Demo işletim sistemi izni istemez.",
                  "Paket uygulama ses veya titreşim kullanılamadığında bunu açıkça bildirmelidir."
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "Görünüm",
        "summary": "Görünüm, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Açık veya Koyu modu, paleti ve yerel YAML vurgulama temasını seçin.",
                  "Kullanılamaz, salt okunur ve bekleyen durumlar açıkça gösterilir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Cüzdan sırları ve özel taşıma verileri Yardım içine girmez.",
                  "Bu Yardım uygulamayla paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "Varlık ayrıntıları",
        "summary": "Seçili varlığın kimliğini, ihraççısını, arzını ve yerel sınıflandırmasını inceleyin.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ad ve sembol varlığı; Sahip ve Varlık Kimliği beyan edilen kaynağı tanımlar.",
                  "Yetkili yerel kaynak yoksa mevcut ve azami arz kullanılamaz kalır."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli çalışma",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Alanlar salt okunurdur; piyasa değerini, sahipliği veya protokol güvenini kanıtlamaz.",
                  "Simge, meta veri ve Yardım yerel olarak paketlenir ve çevrimdışı çalışır."
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps — ayrıntılar",
        "summary": "dApps — ayrıntılar: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps — izin incelemesi",
        "summary": "dApps — izin incelemesi: sınırlı yerel dApps önizlemesi ve izin sınırı hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel tanımları, kapsamlı niyetleri ve açık sonuçları inceleyin.",
                  "Kabulden önce kapsamı, kullanımları, süreyi, değeri, ücreti, açıklamayı ve iptali inceleyin."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps bir yol haritası önizlemesidir; uzak kod, rastgele URL veya genel imza çalıştırmaz.",
                  "Kabul edilen niyeti Cüzdan yeniden doğrular; bu görünüm cüzdan nesnelerini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "Mesajlaşma — ayrıntılar",
        "summary": "Mesajlaşma — ayrıntılar: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.",
                  "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.",
                  "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "Mesajlaşma — istek incelemesi",
        "summary": "Mesajlaşma — istek incelemesi: özel istek eşgüdümü önizlemesi ve Cüzdan aktarımı hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel mesajları, istekleri, makbuzları, süreyi ve kurtarma durumlarını inceleyin.",
                  "Kabul, Cüzdan inceleme niyeti oluşturur; ödeme yapmaz veya cüzdan durumunu değiştirmez."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger kısa süreli aktarım için yol haritası önizlemesidir; kalıcı zincir üstü sohbet değildir.",
                  "Açma, silme, engelleme veya bildirme Cüzdan ödeme durumunu değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "Kişiler — ayrıntılar",
        "summary": "Kişiler — ayrıntılar: yerel kişi etiketleri, alıcı kartları ve açık kimlik değişikliği incelemesi hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel kişi verilerini, süreyi, iptali ve kimlik değişikliği kanıtını inceleyin.",
                  "Kaydedilmiş etiket kimliği veya güveni kanıtlamaz; değişen veriler açıkça incelenmelidir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kişiler yerel kalır ve adres ya da çevrim içi durum grafiği olarak yüklenmez.",
                  "Yerel kişiyi kaldırmak dış hakları iptal etmez veya Cüzdan ödemesini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "Kişiler — kimlik incelemesi",
        "summary": "Kişiler — kimlik incelemesi: yerel kişi etiketleri, alıcı kartları ve açık kimlik değişikliği incelemesi hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel kişi verilerini, süreyi, iptali ve kimlik değişikliği kanıtını inceleyin.",
                  "Kaydedilmiş etiket kimliği veya güveni kanıtlamaz; değişen veriler açıkça incelenmelidir."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kişiler yerel kalır ve adres ya da çevrim içi durum grafiği olarak yüklenmez.",
                  "Yerel kişiyi kaldırmak dış hakları iptal etmez veya Cüzdan ödemesini değiştirmez."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "İzleyiciler — uyarı ayrıntıları",
        "summary": "İzleyiciler — uyarı ayrıntıları: salt okunur Watchers yol haritası önizlemesi ve genel kanıt sınırı hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Ağ durumunu değiştirmeden belirlenimci yayın verilerini inceleyin.",
                  "Kullanılamayan, eski, bozuk ve hata durumları açık kalır ve güvenli biçimde başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers, yerel verili bir yol haritası önizlemesidir; yayımlanmış protokol özelliği değildir.",
                  "Cüzdan etiketleri, taraflar, yollar, mesajlar ve gizli bilgiler açığa çıkmaz."
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "Gezgin — ayrıntılar",
        "summary": "Gezgin — ayrıntılar: desteklenen genel kimlikler için gizlilik sınırlı Explorer önizlemesi hakkında yardım.",
        "scope": "dialog",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yalnızca desteklenen genel kontrol noktası, parti, uyarı veya kanıt kimliklerini kullanın.",
                  "Bilinmeyen, özel, bozuk veya kullanılamayan kimlikler cüzdan sorgusu olmadan başarısız olur."
                ]
              }
            ]
          },
          {
            "title": "Yerel ve güvenli davranış",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer, yerel verili bir yol haritası önizlemesidir; cüzdan veri hizmeti değildir.",
                  "Yerel bakiyeler, kişiler, mesajlar, notlar, yollar ve gizli bilgiler Explorer’a girmez."
                ]
              }
            ]
          }
        ]
      }
    },
    "ja": {
      "app": {
        "id": "app",
        "title": "アプリケーションヘルプ",
        "summary": "ローカルヘルプはこの画面を説明し、オフラインでも利用できます。",
        "scope": "global",
        "sections": [
          {
            "title": "このヘルプの使い方",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "アプリの移動とオフライン動作は全体ヘルプ、各画面の操作はその画面の質問ボタンで確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          },
          {
            "title": "テストテキスト",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "テスト",
                  "テスト"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "このアプリについて",
        "summary": "このアプリについて：Z00Z のバージョン、目的、更新チャネルです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "このセッションの現在のデモバージョンを確認します。",
                  "JavaScript デモは Rust と Tauri の UX 目標を定義します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "デモは更新をダウンロードまたはインストールしません。",
                  "製品版は署名済みリリースマニフェストを検証する必要があります。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "資産",
        "summary": "選択したウォレットのコイン、トークン、NFTをローカル残高と市場データの状態とともに確認します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "「すべて」「コイン」「トークン」「NFT」で選択中のウォレットの資産を絞り込みます。",
                  "残高はウォレットのデータです。信頼できる市場フィードがない場合、価値と価格は「利用不可」のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "行を選ぶと読み取り専用の資産情報が開きます。送信と受信は別のウォレット操作です。",
                  "資産アイコンとヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "バウチャー",
        "summary": "バウチャーの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ライフサイクルでバウチャーを絞り込み、条件を開くか最初のバウチャーを作成します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "権限",
        "summary": "権限の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ゼロ価値の権利を保有・委任・使用済みで絞り込み、行を開いて限定権限を確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "隔離",
        "summary": "隔離：ウォレットによる明示的な確認が必要な対象のローカルヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "操作する前に、表示された理由、提供元、ローカル状態を確認してください。",
                  "ネイティブウォレットが安全な次の手順を示すまで、利用不可の操作はブロックされます。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最終判断はこの画面ではなく、ネイティブウォレットのポリシーが行います。",
                  "秘密情報と非公開の転送データがヘルプに入ることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "送信",
        "summary": "送信の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最初に資産、バウチャー、権限を選びます。これらは価値、ポリシーに従う条件付き価値、ゼロ価値の限定権限です。",
                  "1回の承認前に、受取人と残高またはポリシー、有効期限、残り使用回数、範囲、委任条件を確認します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "受信",
        "summary": "受信の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "選択したウォレットの受信カードを表示し、省略された受信者を別経路で共有します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "履歴",
        "summary": "履歴の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "オブジェクト種類でイベントを絞り込み、行を開いて受領証と技術的ライフサイクルを確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "ステーク",
        "summary": "ステーク：ウォレットによる明示的な確認が必要な対象のローカルヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "操作する前に、表示された理由、提供元、ローカル状態を確認してください。",
                  "ネイティブウォレットが安全な次の手順を示すまで、利用不可の操作はブロックされます。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最終判断はこの画面ではなく、ネイティブウォレットのポリシーが行います。",
                  "秘密情報と非公開の転送データがヘルプに入ることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "アンステーク",
        "summary": "アンステーク：ウォレットによる明示的な確認が必要な対象のローカルヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "操作する前に、表示された理由、提供元、ローカル状態を確認してください。",
                  "ネイティブウォレットが安全な次の手順を示すまで、利用不可の操作はブロックされます。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最終判断はこの画面ではなく、ネイティブウォレットのポリシーが行います。",
                  "秘密情報と非公開の転送データがヘルプに入ることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "ウォレットのバックアップ",
        "summary": "ウォレットのバックアップの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "新しい暗号化バックアップを作成する前に、最新コピーの日付、整合性、保存先を確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "ウォレット一般設定",
        "summary": "ウォレット一般設定の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "選択中のウォレットだけを改名できます。ウォレット ID と作成時のチェーンは読み取り専用です。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "ウォレットセキュリティ",
        "summary": "ウォレットセキュリティの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "非アクティブ時のロック、即時ロック、または選択したウォレットのパスワード変更を設定します。",
                  "リカバリーフレーズの表示とマスターキーのローテーションには再認証と明示的な確認が必要です。ローテーション前にバックアップを検証してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "ウォレットバックアップ",
        "summary": "ウォレットバックアップの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "自動バックアップ、間隔、作成、復元は選択したウォレットだけに適用されます。",
                  "復元は置換前に整合性を検証します。シードのみの復元では、ラベル、ローカル履歴、受信者コンテキスト、開示アーティファクトは戻りません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "ウォレットポリシー",
        "summary": "ウォレットポリシーの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "このウォレットのプロファイル、ローカル支出制限、固定プロトコル規則、コンプライアンス可否を確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "ウォレット詳細設定",
        "summary": "ウォレット詳細設定の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "安全なローカル YAML 下書きを検証して適用します。秘密情報とファイルパスは含まれません。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum概要",
        "summary": "Reticulum概要は、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulum概要の証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulumノード",
        "summary": "Reticulumノードは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumノードの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulumインターフェース",
        "summary": "Reticulumインターフェースは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumインターフェースの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum無線",
        "summary": "Reticulum無線は、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulum無線の証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulumエントリーポイント",
        "summary": "Reticulumエントリーポイントは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumエントリーポイントの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulumパス",
        "summary": "Reticulumパスは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumパスの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulumプローブ",
        "summary": "Reticulumプローブは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumプローブの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulumリンク",
        "summary": "Reticulumリンクは、登録済みローカルReticulumブリッジからの読み取り専用キャリア証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するReticulumリンクの証拠を確認します。この画面からReticulumは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、アドレス、宛先、経路、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet概要",
        "summary": "OnionNet概要は、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNet概要の集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNetエポック",
        "summary": "OnionNetエポックは、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNetエポックの集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNetプライバシー",
        "summary": "OnionNetプライバシーは、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNetプライバシーの集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNetトランスポート",
        "summary": "OnionNetトランスポートは、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNetトランスポートの集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNetキューと再生",
        "summary": "OnionNetキューと再生は、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNetキューと再生の集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet検査",
        "summary": "OnionNet検査は、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNet検査の集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNetイングレス",
        "summary": "OnionNetイングレスは、経路やセッションを公開しないプライバシー保護OnionNet集約を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供するOnionNetイングレスの集約を確認します。この画面からOnionNetは変更できません。",
                  "利用不可は新しいローカルスナップショットがないことを示し、経路、エンドポイント、セッションID、ペイロードは非表示のままです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "アグリゲーター概要",
        "summary": "アグリゲーター概要は、ローカルブリッジからの読み取り専用の公開・配置証拠を表示します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルブリッジが提供する公開、配置、検証、ライフサイクルの証拠を確認します。",
                  "利用不可は新しいローカルスナップショットがないことを示し、デモはネットワーク状態を推測しません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "アグリゲーター入力",
        "summary": "Runtime がトランザクションまたは claim payload を digest に結び付いた作業項目として受け入れる方法を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`WorkPayload` から `WorkItem` または `RejectRecord` への契約を確認します。",
                  "利用不可は新しい admission snapshot がないことを示し、受理や拒否を意味しません。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 境界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Object package の結合は admission digest と intake identity を変更します。",
                  "Raw payload、受取人、メモ、ウォレットのローカル経路はヘルプに入りません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "アグリゲーター計画",
        "summary": "Settlement 権限を主張せず、決定論的な batch と shard route の結合を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Planner mode、route generation、intake と operation の数、digest の所有者を確認します。",
                  "利用不可は検証済み `BatchPlanned` snapshot が接続されていないことを示します。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 境界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "設定、generation、route-table digest、再計算した plan は一致する必要があります。",
                  "計画は settlement、publication、storage truth を確定しません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "アグリゲーター配置",
        "summary": "Runtime が所有する shard generation、primary、secondary readiness、journal lineage を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "グローバルなトポロジーを推測せず `ShardPlacementView` 契約を確認します。",
                  "利用不可は現在の placement table 観測が接続されていないことを示します。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 境界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Table は正確な shard と routing generation を所有する必要があります。",
                  "Aggregator ID は運用データで、endpoint とウォレット identity は非表示です。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "アグリゲーター公開",
        "summary": "Ordered batch が checkpoint、quorum、DA、lifecycle evidence に結び付く方法を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`PublicationRequest` から `PublishedBatch`、`PublicationRecord` への流れを確認します。",
                  "利用不可は検証済み publication または readiness bundle が接続されていないことを示します。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 境界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Provider、height、manifest、payload、statement、evidence が不完全または不一致なら拒否されます。",
                  "Storage が checkpoint root、proof、lifecycle truth を所有します。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "アグリゲーター復旧",
        "summary": "確定した route、generation、primary、journal lineage に対する再起動と secondary takeover の検査を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "`ShardRecoveryRecord`、recovery intent、durable state、execution ticket を確認します。",
                  "利用不可は確定済み recovery snapshot が接続されていないことを示します。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 境界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Generation、primary、shard、batch、route、lineage が誤っていれば拒否されます。",
                  "Renderer は failover を開始したり Storage の recovery truth を変更したりできません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "監視：概要",
        "summary": "監視：概要：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "監視：アラート",
        "summary": "監視：アラート：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "監視：公開",
        "summary": "監視：公開：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "監視：DAプロバイダー",
        "summary": "監視：DAプロバイダー：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "監視：検閲シグナル",
        "summary": "監視：検閲シグナル：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "監視：証拠をエクスポート",
        "summary": "監視：証拠をエクスポート：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "エクスプローラー：概要",
        "summary": "エクスプローラー：概要：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "エクスプローラー：検索",
        "summary": "エクスプローラー：検索：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "エクスプローラー：チェックポイント",
        "summary": "エクスプローラー：チェックポイント：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "エクスプローラー：バッチ",
        "summary": "エクスプローラー：バッチ：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "エクスプローラー：公開証拠",
        "summary": "エクスプローラー：公開証拠：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "探索",
        "summary": "探索：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "インストール済み",
        "summary": "インストール済み：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "接続",
        "summary": "接続：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "権限",
        "summary": "権限：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "プライベートスワップ",
        "summary": "プライベートスワップの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "保有する元資産、金額、互換性のある対象資産を選び、送信前にプレビューを確認します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "取引所",
        "summary": "取引所の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "注文板リクエストには Hyperliquid Spot、solver によるクロスチェーンリクエストには NEAR Intents を選びます。",
                  "ペアまたはルート、受取人/返金、スリッページ、期限を確認します。検証済みコネクタなしでは見積、出力、手数料、入金先、実行状態は利用できません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "受信トレイ",
        "summary": "受信トレイ：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。",
                  "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。",
                  "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "送信済み",
        "summary": "送信済み：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。",
                  "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。",
                  "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "会話",
        "summary": "会話：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。",
                  "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。",
                  "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "ディスク使用量",
        "summary": "ディスク使用量：非公開データを含まない集計ローカル値です。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレット記録を開かずにリソース使用量を確認します。",
                  "表示値は決定的なデモデータです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "連絡先、メッセージ、経路、操作、秘密情報は除外されます。",
                  "製品版は制限されたネイティブ機能から集計値のみを取得する必要があります。"
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "ネットワーク使用量",
        "summary": "ネットワーク使用量：非公開データを含まない集計ローカル値です。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレット記録を開かずにリソース使用量を確認します。",
                  "表示値は決定的なデモデータです。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "連絡先、メッセージ、経路、操作、秘密情報は除外されます。",
                  "製品版は制限されたネイティブ機能から集計値のみを取得する必要があります。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "連絡先",
        "summary": "連絡先：ローカル連絡先ラベル、受取カード、明示的な本人情報変更確認のヘルプです。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルデータ、有効期限、失効、本人情報変更の証拠を確認します。",
                  "保存したラベルは本人確認や信頼の証明ではなく、変更されたデータは確認が必要です。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "連絡先はローカルに保持され、住所や在席状況のグラフとして送信されません。",
                  "ローカル連絡先の削除は外部権限を失効させず、ウォレット決済も変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "一般設定",
        "summary": "一般設定の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "アプリの言語、地域形式、表示タイムゾーン、通知設定を選択します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "通知",
        "summary": "通知：通知、振動、着信音のローカル設定です。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "振動と着信音を選ぶ前に通知を有効にします。",
                  "通知が無効な場合、関連する選択肢も無効です。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "デモは OS 権限を要求しません。",
                  "製品版は音声や振動を利用できない場合に明示する必要があります。"
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "外観",
        "summary": "外観の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ライト／ダーク、パレット、ローカル YAML 強調テーマを選択します。",
                  "利用不可、読み取り専用、保留中の状態を明確に表示します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ウォレットの秘密情報と非公開の通信データはヘルプに含まれません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "資産の詳細",
        "summary": "選択した資産の識別情報、発行者、供給量、ローカル分類を確認します。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "名前とティッカーは資産を、所有者と資産 ID は申告された出所を示します。",
                  "信頼できるローカル情報源がない場合、現在供給量と最大供給量は「利用不可」です。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "各項目は読み取り専用で、市場価値、所有権、プロトコルの信頼性を証明しません。",
                  "アイコン、メタデータ、ヘルプはローカルに同梱され、オフラインで動作します。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps：詳細",
        "summary": "dApps：詳細：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps：権限の確認",
        "summary": "dApps：権限の確認：制限されたローカル dApps プレビューと権限境界のヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカル記述、範囲限定インテント、明示された結果を確認します。",
                  "承認前に範囲、利用回数、有効期限、金額、手数料、開示、取り消しを確認してください。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps はロードマッププレビューであり、外部コード、任意 URL、汎用署名を実行しません。",
                  "承認したインテントはウォレットが再検証し、この画面はウォレット対象を変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "メッセンジャー：詳細",
        "summary": "メッセンジャー：詳細：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。",
                  "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。",
                  "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "メッセンジャー：リクエストの確認",
        "summary": "メッセンジャー：リクエストの確認：非公開リクエスト調整プレビューとウォレット引き渡しのヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルのメッセージ、リクエスト、受領書、期限、復旧状態を確認します。",
                  "承認はウォレット確認用インテントを作るだけで、決済や状態変更は行いません。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger は短期中継のロードマッププレビューであり、永続的なオンチェーンチャットではありません。",
                  "開く、削除、ブロック、報告の操作がウォレット決済状態を変えることはありません。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "連絡先：詳細",
        "summary": "連絡先：詳細：ローカル連絡先ラベル、受取カード、明示的な本人情報変更確認のヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルデータ、有効期限、失効、本人情報変更の証拠を確認します。",
                  "保存したラベルは本人確認や信頼の証明ではなく、変更されたデータは確認が必要です。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "連絡先はローカルに保持され、住所や在席状況のグラフとして送信されません。",
                  "ローカル連絡先の削除は外部権限を失効させず、ウォレット決済も変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "連絡先：本人情報の確認",
        "summary": "連絡先：本人情報の確認：ローカル連絡先ラベル、受取カード、明示的な本人情報変更確認のヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルデータ、有効期限、失効、本人情報変更の証拠を確認します。",
                  "保存したラベルは本人確認や信頼の証明ではなく、変更されたデータは確認が必要です。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "連絡先はローカルに保持され、住所や在席状況のグラフとして送信されません。",
                  "ローカル連絡先の削除は外部権限を失効させず、ウォレット決済も変更しません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "監視：アラートの詳細",
        "summary": "監視：アラートの詳細：読み取り専用の Watchers ロードマッププレビューと公開証拠の境界に関するヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ネットワーク状態を変更せず、決定的な公開データを確認します。",
                  "利用不可、古い、不正な形式、エラーの状態は明示され、安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers はローカルデータによるロードマッププレビューであり、提供済みのプロトコル機能ではありません。",
                  "ウォレット名、取引相手、経路、メッセージ、秘密情報は公開されません。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "エクスプローラー：詳細",
        "summary": "エクスプローラー：詳細：対応する公開 ID のみを扱うプライバシー制限付き Explorer プレビューのヘルプです。",
        "scope": "dialog",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "対応する公開チェックポイント、バッチ、アラート、証拠 ID のみを使用してください。",
                  "不明、非公開、不正、利用不可の ID はウォレットを参照せず安全側に失敗します。"
                ]
              }
            ]
          },
          {
            "title": "ローカルで安全な動作",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer はローカルデータによるロードマッププレビューであり、ウォレットデータサービスではありません。",
                  "ローカル残高、連絡先、メッセージ、メモ、経路、秘密情報は Explorer に入りません。"
                ]
              }
            ]
          }
        ]
      }
    },
    "zh-Hans": {
      "app": {
        "id": "app",
        "title": "应用帮助",
        "summary": "本地帮助说明此视图，并可离线使用。",
        "scope": "global",
        "sections": [
          {
            "title": "使用此帮助",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "使用全局帮助了解应用导航和离线行为；使用各视图中的问号了解该视图的控件。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          },
          {
            "title": "测试文本",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "测试",
                  "测试"
                ]
              }
            ]
          }
        ]
      },
      "about": {
        "id": "about",
        "title": "关于",
        "summary": "关于：Z00Z 版本、用途和更新通道。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "检查本次会话的当前演示版本。",
                  "JavaScript 演示定义 Rust 和 Tauri 的 UX 目标。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "演示不会下载或安装更新。",
                  "正式应用必须验证签名的发布清单。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.assets": {
        "id": "wallet.assets",
        "title": "资产",
        "summary": "查看所选钱包的代币、通证和 NFT，以及本地余额和市场数据状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "使用“全部”“代币”“通证”或“NFT”筛选所选钱包的资产列表。",
                  "余额属于钱包数据。没有可信市场源时，价值和价格保持“不可用”。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "选择一行会打开只读资产元数据；发送和接收仍是独立的钱包操作。",
                  "资产图标和帮助随应用本地打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.vouchers": {
        "id": "wallet.vouchers",
        "title": "凭证",
        "summary": "凭证说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "按生命周期筛选凭证，打开一行查看条件，或在没有凭证时创建一个。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.permissions": {
        "id": "wallet.permissions",
        "title": "权限",
        "summary": "权限说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "按持有、已委派或已使用筛选零价值权限，并打开一行查看其受限授权。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.quarantine": {
        "id": "wallet.quarantine",
        "title": "隔离",
        "summary": "隔离：针对需要钱包明确审核的对象的本地帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "执行任何操作前，请检查显示的原因、来源和本地状态。",
                  "在原生钱包提供安全的后续步骤前，不可用的操作保持阻止状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最终决定由原生钱包策略作出，而不是此视图。",
                  "机密和私密传输数据绝不会进入帮助。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.send": {
        "id": "wallet.send",
        "title": "发送",
        "summary": "发送说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "先选择资产、凭证或权限：它们分别表示价值、受策略约束的条件价值，以及零价值的有限权限。",
                  "单次授权前，请检查接收者，以及所选类别的余额或策略、到期时间、剩余次数、范围和委托限制。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.receive": {
        "id": "wallet.receive",
        "title": "接收",
        "summary": "接收说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "显示所选钱包的接收卡，并复制缩写接收者以通过其他渠道共享。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.history": {
        "id": "wallet.history",
        "title": "历史记录",
        "summary": "历史记录说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "按对象类型筛选钱包事件，并打开一行查看收据和技术生命周期。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.stake": {
        "id": "wallet.staking.stake",
        "title": "质押",
        "summary": "质押：针对需要钱包明确审核的对象的本地帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "执行任何操作前，请检查显示的原因、来源和本地状态。",
                  "在原生钱包提供安全的后续步骤前，不可用的操作保持阻止状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最终决定由原生钱包策略作出，而不是此视图。",
                  "机密和私密传输数据绝不会进入帮助。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.staking.unstake": {
        "id": "wallet.staking.unstake",
        "title": "解除质押",
        "summary": "解除质押：针对需要钱包明确审核的对象的本地帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "执行任何操作前，请检查显示的原因、来源和本地状态。",
                  "在原生钱包提供安全的后续步骤前，不可用的操作保持阻止状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "最终决定由原生钱包策略作出，而不是此视图。",
                  "机密和私密传输数据绝不会进入帮助。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.backup": {
        "id": "wallet.backup",
        "title": "钱包备份",
        "summary": "钱包备份说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "创建新的加密备份前，查看最近本地备份的日期、完整性和目标位置。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.general": {
        "id": "wallet.settings.general",
        "title": "钱包常规设置",
        "summary": "钱包常规设置说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "只能重命名所选钱包；钱包 ID 和创建时选择的链保持只读。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.security": {
        "id": "wallet.settings.security",
        "title": "钱包安全",
        "summary": "钱包安全说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "设置闲置锁定、立即锁定，或更改所选钱包的密码。",
                  "查看恢复短语和轮换主密钥需要重新认证及明确确认；轮换前请验证备份。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.backup": {
        "id": "wallet.settings.backup",
        "title": "钱包备份",
        "summary": "钱包备份说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "自动备份、间隔、创建和恢复仅作用于所选钱包。",
                  "恢复会在替换前验证完整性。仅用助记词恢复不会找回标签、本地历史、接收者上下文或披露资料。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.policies": {
        "id": "wallet.settings.policies",
        "title": "钱包策略",
        "summary": "钱包策略说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看此钱包的配置文件、本地支出限制、锁定的协议规则和合规可用性。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.settings.advanced": {
        "id": "wallet.settings.advanced",
        "title": "钱包高级设置",
        "summary": "钱包高级设置说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "验证并应用所选钱包的安全本地 YAML 草稿；其中不包含机密和文件路径。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.overview": {
        "id": "telemetry.reticulum.overview",
        "title": "Reticulum 概览",
        "summary": "Reticulum 概览显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 概览证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.node": {
        "id": "telemetry.reticulum.node",
        "title": "Reticulum 节点",
        "summary": "Reticulum 节点显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 节点证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.interfaces": {
        "id": "telemetry.reticulum.interfaces",
        "title": "Reticulum 接口",
        "summary": "Reticulum 接口显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 接口证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.radio": {
        "id": "telemetry.reticulum.radio",
        "title": "Reticulum 无线",
        "summary": "Reticulum 无线显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 无线证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.entrypoints": {
        "id": "telemetry.reticulum.entrypoints",
        "title": "Reticulum 入口点",
        "summary": "Reticulum 入口点显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 入口点证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.paths": {
        "id": "telemetry.reticulum.paths",
        "title": "Reticulum 路径",
        "summary": "Reticulum 路径显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 路径证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.probes": {
        "id": "telemetry.reticulum.probes",
        "title": "Reticulum 探针",
        "summary": "Reticulum 探针显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 探针证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.reticulum.links": {
        "id": "telemetry.reticulum.links",
        "title": "Reticulum 链路",
        "summary": "Reticulum 链路显示来自已注册本地 Reticulum 桥的只读载波证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的Reticulum 链路证据；此页面不能更改 Reticulum。",
                  "不可用表示没有新的本地快照；地址、目标、路由和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.overview": {
        "id": "telemetry.onionnet.overview",
        "title": "OnionNet 概览",
        "summary": "OnionNet 概览显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 概览聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.epoch": {
        "id": "telemetry.onionnet.epoch",
        "title": "OnionNet 纪元",
        "summary": "OnionNet 纪元显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 纪元聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.privacy": {
        "id": "telemetry.onionnet.privacy",
        "title": "OnionNet 隐私",
        "summary": "OnionNet 隐私显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 隐私聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.transport": {
        "id": "telemetry.onionnet.transport",
        "title": "OnionNet 传输",
        "summary": "OnionNet 传输显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 传输聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.queues": {
        "id": "telemetry.onionnet.queues",
        "title": "OnionNet 队列与重放",
        "summary": "OnionNet 队列与重放显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 队列与重放聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.probation": {
        "id": "telemetry.onionnet.probation",
        "title": "OnionNet 检查",
        "summary": "OnionNet 检查显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 检查聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.onionnet.ingress": {
        "id": "telemetry.onionnet.ingress",
        "title": "OnionNet 入口",
        "summary": "OnionNet 入口显示保护隐私的 OnionNet 聚合，不公开路由或会话。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的OnionNet 入口聚合；此页面不能更改 OnionNet。",
                  "不可用表示没有新的本地快照；路由、端点、会话标识和负载保持隐藏。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.overview": {
        "id": "telemetry.aggregators.overview",
        "title": "聚合器概览",
        "summary": "聚合器概览显示来自本地桥的只读发布和放置证据。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地桥提供的发布、放置、验证和生命周期证据。",
                  "不可用表示没有新的本地快照；演示不会虚构网络状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.ingress": {
        "id": "telemetry.aggregators.ingress",
        "title": "聚合器入口",
        "summary": "此视图说明运行时如何将交易或 claim payload 接纳为绑定 digest 的工作项。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看从 `WorkPayload` 到 `WorkItem` 或 `RejectRecord` 的契约。",
                  "不可用表示没有新的 admission snapshot，并不表示已接纳或拒绝。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 边界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "绑定 object package 会改变 admission digest 和 intake identity。",
                  "Raw payload、接收方、备注和钱包本地路由不会进入帮助内容。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.planning": {
        "id": "telemetry.aggregators.planning",
        "title": "聚合器规划",
        "summary": "此视图说明确定性的 batch 和 shard route 绑定，而不声称拥有 settlement 权限。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看 planner mode、route generation、intake 和 operation 数量以及 digest 所有权。",
                  "不可用表示未连接经过验证的 `BatchPlanned` snapshot。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 边界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "配置、generation、route-table digest 和重新计算的 plan 必须一致。",
                  "规划不会最终确定 settlement、publication 或 storage truth。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.placement": {
        "id": "telemetry.aggregators.placement",
        "title": "聚合器放置",
        "summary": "此视图说明运行时拥有的 shard generation、primary、secondary readiness 和 journal lineage。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看 `ShardPlacementView` 契约，不要推断全局拓扑。",
                  "不可用表示未连接当前 placement table 观测。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 边界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Table 必须拥有精确的 shard 和 routing generation。",
                  "Aggregator ID 是运行数据；endpoint 和钱包 identity 保持隐藏。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.publication": {
        "id": "telemetry.aggregators.publication",
        "title": "聚合器发布",
        "summary": "此视图说明 ordered batch 如何绑定 checkpoint、quorum、DA 和 lifecycle evidence。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看 `PublicationRequest` 到 `PublishedBatch` 和 `PublicationRecord` 的流程。",
                  "不可用表示未连接经过验证的 publication 或 readiness bundle。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 边界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Provider、height、manifest、payload、statement 或 evidence 不完整或不一致时会被拒绝。",
                  "Storage 拥有 checkpoint root、proof 和 lifecycle truth。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.aggregators.recovery": {
        "id": "telemetry.aggregators.recovery",
        "title": "聚合器恢复",
        "summary": "此视图说明针对已提交 route、generation、primary 和 journal lineage 的重启与 secondary takeover 检查。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看 `ShardRecoveryRecord`、recovery intent、durable state 和 execution ticket。",
                  "不可用表示未连接已提交的 recovery snapshot。"
                ]
              }
            ]
          },
          {
            "title": "Fail-closed 边界",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Generation、primary、shard、batch、route 或 lineage 错误时会被拒绝。",
                  "Renderer 不能启动 failover 或修改 Storage recovery truth。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.overview": {
        "id": "telemetry.watchers.overview",
        "title": "监测器：概览",
        "summary": "监测器：概览：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alerts": {
        "id": "telemetry.watchers.alerts",
        "title": "监测器：警报",
        "summary": "监测器：警报：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.publication": {
        "id": "telemetry.watchers.publication",
        "title": "监测器：发布",
        "summary": "监测器：发布：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.providers": {
        "id": "telemetry.watchers.providers",
        "title": "监测器：DA 提供商",
        "summary": "监测器：DA 提供商：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.censorship": {
        "id": "telemetry.watchers.censorship",
        "title": "监测器：审查信号",
        "summary": "监测器：审查信号：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.evidence": {
        "id": "telemetry.watchers.evidence",
        "title": "监测器：导出证据",
        "summary": "监测器：导出证据：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.overview": {
        "id": "telemetry.explorer.overview",
        "title": "浏览器：概览",
        "summary": "浏览器：概览：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.search": {
        "id": "telemetry.explorer.search",
        "title": "浏览器：搜索",
        "summary": "浏览器：搜索：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.checkpoints": {
        "id": "telemetry.explorer.checkpoints",
        "title": "浏览器：检查点",
        "summary": "浏览器：检查点：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.batches": {
        "id": "telemetry.explorer.batches",
        "title": "浏览器：批次",
        "summary": "浏览器：批次：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.evidence": {
        "id": "telemetry.explorer.evidence",
        "title": "浏览器：公开证据",
        "summary": "浏览器：公开证据：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.discover": {
        "id": "dapps.discover",
        "title": "发现",
        "summary": "发现：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.installed": {
        "id": "dapps.installed",
        "title": "已安装",
        "summary": "已安装：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.connections": {
        "id": "dapps.connections",
        "title": "连接",
        "summary": "连接：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.permissions": {
        "id": "dapps.permissions",
        "title": "权限",
        "summary": "权限：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.swap": {
        "id": "wallet.swap",
        "title": "私密兑换",
        "summary": "私密兑换说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "选择持有的源资产、金额和兼容目标资产，然后在提交前检查预览。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "wallet.exchange": {
        "id": "wallet.exchange",
        "title": "交易所",
        "summary": "交易所说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "订单簿请求选择 Hyperliquid Spot；solver 驱动的跨链请求选择 NEAR Intents。",
                  "检查交易对或路线、接收/退款地址、滑点和期限。没有已验证连接器时，报价、输出、费用、充值地址和执行状态保持不可用。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.inbox": {
        "id": "messenger.inbox",
        "title": "收件箱",
        "summary": "收件箱：私密请求协调预览及其钱包交接的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地消息、请求、回执、到期和恢复状态。",
                  "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger 是短期中继的路线图预览，并非永久链上聊天。",
                  "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.sent": {
        "id": "messenger.sent",
        "title": "已发送",
        "summary": "已发送：私密请求协调预览及其钱包交接的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地消息、请求、回执、到期和恢复状态。",
                  "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger 是短期中继的路线图预览，并非永久链上聊天。",
                  "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.conversations": {
        "id": "messenger.conversations",
        "title": "会话",
        "summary": "会话：私密请求协调预览及其钱包交接的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地消息、请求、回执、到期和恢复状态。",
                  "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger 是短期中继的路线图预览，并非永久链上聊天。",
                  "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"
                ]
              }
            ]
          }
        ]
      },
      "data-storage.disk-usage": {
        "id": "data-storage.disk-usage",
        "title": "磁盘使用情况",
        "summary": "磁盘使用情况：不含私密数据的本地汇总计数。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "无需打开钱包记录即可查看资源使用情况。",
                  "显示值为确定性的演示数据。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "联系人、消息、路径、活动和机密均被排除。",
                  "正式应用只能通过受限原生能力获取汇总值。"
                ]
              }
            ]
          }
        ]
      },
      "data-storage.network-usage": {
        "id": "data-storage.network-usage",
        "title": "网络使用情况",
        "summary": "网络使用情况：不含私密数据的本地汇总计数。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "无需打开钱包记录即可查看资源使用情况。",
                  "显示值为确定性的演示数据。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "联系人、消息、路径、活动和机密均被排除。",
                  "正式应用只能通过受限原生能力获取汇总值。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.list": {
        "id": "contacts.list",
        "title": "联系人",
        "summary": "联系人：本地联系人标签、接收卡和明确身份变更审核的帮助。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地联系人数据、到期、撤销和身份变更证据。",
                  "已保存的标签不能证明身份或信任；变更后的数据需要明确审核。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "联系人保留在本地，不会作为地址或在线状态图上传。",
                  "删除本地联系人不会撤销外部权限或更改钱包结算。"
                ]
              }
            ]
          }
        ]
      },
      "settings.general": {
        "id": "settings.general",
        "title": "常规设置",
        "summary": "常规设置说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "选择应用语言、区域格式、显示时区和通知偏好。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "settings.notifications": {
        "id": "settings.notifications",
        "title": "通知",
        "summary": "通知：本地通知、振动和铃声设置。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "选择振动和铃声前，请先启用通知。",
                  "关闭通知时，相关选项也会禁用。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "演示不会请求操作系统权限。",
                  "正式应用必须明确提示声音或振动不可用。"
                ]
              }
            ]
          }
        ]
      },
      "settings.appearance": {
        "id": "settings.appearance",
        "title": "外观",
        "summary": "外观说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "切换浅色或深色模式，选择调色板和本地 YAML 高亮主题。",
                  "不可用、只读和待处理状态会明确显示。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "钱包机密和私有传输数据不会进入帮助内容。",
                  "此帮助随应用打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "help.faq": {
        "id": "help.faq",
        "title": "Frequently asked questions",
        "summary": "Find short answers about the Z00Z wallet demo, roadmap screens, and local data.",
        "scope": "article",
        "sections": [
          {
            "title": "Can I buy or sell Z00Z here?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No live exchange, fiat provider, bridge, or settlement service is connected to this demo.",
                  "Swap and Exchange show a proposed roadmap flow only; unavailable actions remain blocked."
                ]
              }
            ]
          },
          {
            "title": "Does the demo use real wallet data?",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "The demo uses deterministic local fixtures and does not request a seed phrase or private key.",
                  "Help is packaged with the application and does not receive wallet secrets or private transport data."
                ]
              }
            ]
          }
        ]
      },
      "help.how-to": {
        "id": "help.how-to",
        "title": "How to use the demo",
        "summary": "Learn the navigation model and the safe way to explore planned Z00Z wallet flows.",
        "scope": "article",
        "sections": [
          {
            "title": "Navigate on desktop and mobile",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open a first-level accordion in the main menu, then select a workspace such as Wallet, Telemetry, or dApps.",
                  "Use the inner vertical navigation on desktop and the same workspace options as top tabs on mobile."
                ]
              }
            ]
          },
          {
            "title": "Explore roadmap exchange flows",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open dApps, then choose Swap or Exchange to review the proposed non-custodial flow.",
                  "The demo does not execute a trade, connect a provider, move funds, or accept wallet secrets."
                ]
              }
            ]
          },
          {
            "title": "Keep the boundary clear",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "A future packaged wallet must revalidate every quote, permission, signature, fee, and destination.",
                  "The original Ethereum and BOLD design research is retained under the English Help drafts and is not presented as a shipped capability."
                ]
              }
            ]
          }
        ]
      },
      "help.report-issues": {
        "id": "help.report-issues",
        "title": "Report issues",
        "summary": "Collect useful, non-sensitive details before reporting a problem with the Z00Z demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Describe the problem",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Record the screen, selected wallet label, language, palette, viewport size, and exact action that failed.",
                  "Include the expected result and the result you observed, with a screenshot when it does not expose private data."
                ]
              }
            ]
          },
          {
            "title": "Protect sensitive data",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Never include a seed phrase, private key, recovery material, private message, contact details, or full wallet history.",
                  "Use the Z00Z GitHub repository issue tracker for reproducible demo defects and remove sensitive data before submitting."
                ]
              }
            ]
          }
        ]
      },
      "help.tips-and-tricks": {
        "id": "help.tips-and-tricks",
        "title": "Tips and tricks",
        "summary": "Use faster navigation, contextual Help, search, and appearance controls across the demo.",
        "scope": "article",
        "sections": [
          {
            "title": "Move through the application",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Keep more than one first-level accordion open when comparing areas, and close either one without changing the other.",
                  "On mobile, use the top workspace tabs for repeated actions instead of reopening the main menu."
                ]
              }
            ]
          },
          {
            "title": "Find the right Help",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Open Help from the main menu for the complete tree, or use the question button for the current screen.",
                  "Search article titles and content, then switch language or palette without leaving the standalone Help application."
                ]
              }
            ]
          }
        ]
      },
      "help.video-tutorials": {
        "id": "help.video-tutorials",
        "title": "Video tutorials",
        "summary": "Review the planned tutorial subjects and the current availability boundary.",
        "scope": "article",
        "sections": [
          {
            "title": "Planned tutorials",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Wallet navigation, Assets, Send, Receive, Contacts, privacy controls, and safe backup are intended tutorial subjects.",
                  "Telemetry, dApps, Messenger, Watchers, and Explorer tutorials must identify roadmap-only behavior clearly."
                ]
              }
            ]
          },
          {
            "title": "Current availability",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "No official tutorial video is bundled with this demo yet.",
                  "Use the searchable Help tree for the current interaction model; future videos must match the same versioned Help content."
                ]
              }
            ]
          }
        ]
      },
      "asset.details": {
        "id": "asset.details",
        "title": "资产详情",
        "summary": "查看所选资产的标识、发行方、供应量和本地分类。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "名称和代码用于识别资产；所有者和资产 ID 表示其声明的来源。",
                  "没有权威本地来源时，当前供应量和最大供应量保持“不可用”。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "这些字段为只读，不能证明市场价值、所有权或协议可信度。",
                  "图标、元数据和帮助均在本地打包，可离线使用。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.detail": {
        "id": "dapps.detail",
        "title": "dApps：详情",
        "summary": "dApps：详情：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "dapps.permission-review": {
        "id": "dapps.permission-review",
        "title": "dApps：权限审核",
        "summary": "dApps：权限审核：受限本地 dApps 预览及其权限边界的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地描述、范围受限的意图和明确结果。",
                  "接受前请检查范围、使用次数、期限、金额、费用、披露和撤销条件。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "dApps 是路线图预览，不会执行远程代码、任意网址或通用签名。",
                  "钱包会重新验证已接受的意图；此视图不能更改钱包对象。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.detail": {
        "id": "messenger.detail",
        "title": "信使：详情",
        "summary": "信使：详情：私密请求协调预览及其钱包交接的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地消息、请求、回执、到期和恢复状态。",
                  "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger 是短期中继的路线图预览，并非永久链上聊天。",
                  "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"
                ]
              }
            ]
          }
        ]
      },
      "messenger.request-review": {
        "id": "messenger.request-review",
        "title": "信使：请求审核",
        "summary": "信使：请求审核：私密请求协调预览及其钱包交接的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地消息、请求、回执、到期和恢复状态。",
                  "接受请求只会创建钱包审核意图，不会结算或更改钱包状态。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Messenger 是短期中继的路线图预览，并非永久链上聊天。",
                  "打开、删除、屏蔽或举报内容不会更改钱包结算状态。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.detail": {
        "id": "contacts.detail",
        "title": "联系人：详情",
        "summary": "联系人：详情：本地联系人标签、接收卡和明确身份变更审核的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地联系人数据、到期、撤销和身份变更证据。",
                  "已保存的标签不能证明身份或信任；变更后的数据需要明确审核。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "联系人保留在本地，不会作为地址或在线状态图上传。",
                  "删除本地联系人不会撤销外部权限或更改钱包结算。"
                ]
              }
            ]
          }
        ]
      },
      "contacts.identity-review": {
        "id": "contacts.identity-review",
        "title": "联系人：身份审核",
        "summary": "联系人：身份审核：本地联系人标签、接收卡和明确身份变更审核的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地联系人数据、到期、撤销和身份变更证据。",
                  "已保存的标签不能证明身份或信任；变更后的数据需要明确审核。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "联系人保留在本地，不会作为地址或在线状态图上传。",
                  "删除本地联系人不会撤销外部权限或更改钱包结算。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.watchers.alert-detail": {
        "id": "telemetry.watchers.alert-detail",
        "title": "监测器：警报详情",
        "summary": "监测器：警报详情：只读 Watchers 路线图预览及其公开证据边界的帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看确定性的发布数据，而不更改网络状态。",
                  "不可用、过期、格式错误和异常状态均会明确显示并安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Watchers 是基于本地样本的路线图预览，并非已交付的协议功能。",
                  "钱包标签、交易对手、路径、消息和机密不会暴露。"
                ]
              }
            ]
          }
        ]
      },
      "telemetry.explorer.detail": {
        "id": "telemetry.explorer.detail",
        "title": "浏览器：详情",
        "summary": "浏览器：详情：面向受支持公开标识符、受隐私约束的 Explorer 预览帮助。",
        "scope": "dialog",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "仅使用受支持的公开检查点、批次、警报或证据标识符。",
                  "未知、私密、格式错误或不可用的标识符不会查询钱包，并会安全失败。"
                ]
              }
            ]
          },
          {
            "title": "本地和安全行为",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Explorer 是基于本地样本的路线图预览，并非钱包数据服务。",
                  "本地余额、联系人、消息、备注、路径和机密不会进入 Explorer。"
                ]
              }
            ]
          }
        ]
      }
    }
  }
});
})(typeof window === "undefined" ? globalThis : window);
