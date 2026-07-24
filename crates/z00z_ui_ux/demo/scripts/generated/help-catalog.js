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
  "topics": [
    {
      "id": "app",
      "file": "app",
      "scope": "global",
      "match": {
        "global": "true"
      }
    },
    {
      "id": "app.home",
      "file": "app-home",
      "scope": "context",
      "match": {
        "view": "home"
      }
    },
    {
      "id": "wallet.assets",
      "file": "wallet-assets",
      "scope": "context",
      "match": {
        "view": "wallet",
        "walletSection": "assets"
      }
    },
    {
      "id": "wallet.vouchers",
      "file": "wallet-vouchers",
      "scope": "context",
      "match": {
        "view": "wallet",
        "walletSection": "vouchers"
      }
    },
    {
      "id": "wallet.permissions",
      "file": "wallet-permissions",
      "scope": "context",
      "match": {
        "view": "wallet",
        "walletSection": "permissions"
      }
    },
    {
      "id": "wallet.send",
      "file": "wallet-send",
      "scope": "context",
      "match": {
        "view": "wallet-send"
      }
    },
    {
      "id": "wallet.receive",
      "file": "wallet-receive",
      "scope": "context",
      "match": {
        "view": "wallet-receive"
      }
    },
    {
      "id": "wallet.swap",
      "file": "wallet-swap",
      "scope": "context",
      "match": {
        "view": "swap"
      }
    },
    {
      "id": "wallet.exchange",
      "file": "wallet-exchange",
      "scope": "context",
      "match": {
        "view": "exchange"
      }
    },
    {
      "id": "wallet.staking",
      "file": "wallet-staking",
      "scope": "context",
      "match": {
        "view": "staking"
      }
    },
    {
      "id": "wallet.backup",
      "file": "wallet-backup",
      "scope": "context",
      "match": {
        "view": "wallet-backup"
      }
    },
    {
      "id": "wallet.history",
      "file": "wallet-history",
      "scope": "context",
      "match": {
        "view": "activity"
      }
    },
    {
      "id": "wallet.settings.general",
      "file": "wallet-settings-general",
      "scope": "context",
      "match": {
        "view": "wallet-settings",
        "walletSettingsSection": "general"
      }
    },
    {
      "id": "wallet.settings.security",
      "file": "wallet-settings-security",
      "scope": "context",
      "match": {
        "view": "wallet-settings",
        "walletSettingsSection": "security"
      }
    },
    {
      "id": "wallet.settings.backup",
      "file": "wallet-settings-backup",
      "scope": "context",
      "match": {
        "view": "wallet-settings",
        "walletSettingsSection": "backup"
      }
    },
    {
      "id": "wallet.settings.policies",
      "file": "wallet-settings-policies",
      "scope": "context",
      "match": {
        "view": "wallet-settings",
        "walletSettingsSection": "policies"
      }
    },
    {
      "id": "wallet.settings.advanced",
      "file": "wallet-settings-advanced",
      "scope": "context",
      "match": {
        "view": "wallet-settings",
        "walletSettingsSection": "advanced"
      }
    },
    {
      "id": "settings.general",
      "file": "settings-general",
      "scope": "context",
      "match": {
        "view": "settings",
        "settingsSection": "general"
      }
    },
    {
      "id": "settings.appearance",
      "file": "settings-appearance",
      "scope": "context",
      "match": {
        "view": "settings",
        "settingsSection": "appearance"
      }
    },
    {
      "id": "settings.reticulum",
      "file": "settings-reticulum",
      "scope": "context",
      "match": {
        "view": "settings",
        "settingsSection": "reticulum"
      }
    },
    {
      "id": "settings.onionnet",
      "file": "settings-onionnet",
      "scope": "context",
      "match": {
        "view": "settings",
        "settingsSection": "onionnet"
      }
    },
    {
      "id": "telemetry.reticulum.overview",
      "file": "telemetry-reticulum-overview",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "overview"
      }
    },
    {
      "id": "telemetry.reticulum.node",
      "file": "telemetry-reticulum-node",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "node"
      }
    },
    {
      "id": "telemetry.reticulum.interfaces",
      "file": "telemetry-reticulum-interfaces",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "interfaces"
      }
    },
    {
      "id": "telemetry.reticulum.radio",
      "file": "telemetry-reticulum-radio",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "radio"
      }
    },
    {
      "id": "telemetry.reticulum.entrypoints",
      "file": "telemetry-reticulum-entrypoints",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "entrypoints"
      }
    },
    {
      "id": "telemetry.reticulum.paths",
      "file": "telemetry-reticulum-paths",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "paths"
      }
    },
    {
      "id": "telemetry.reticulum.probes",
      "file": "telemetry-reticulum-probes",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "probes"
      }
    },
    {
      "id": "telemetry.reticulum.links",
      "file": "telemetry-reticulum-links",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "reticulum",
        "reticulumTelemetryTab": "links"
      }
    },
    {
      "id": "telemetry.onionnet.overview",
      "file": "telemetry-onionnet-overview",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "overview"
      }
    },
    {
      "id": "telemetry.onionnet.epoch",
      "file": "telemetry-onionnet-epoch",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "epoch"
      }
    },
    {
      "id": "telemetry.onionnet.privacy",
      "file": "telemetry-onionnet-privacy",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "privacy"
      }
    },
    {
      "id": "telemetry.onionnet.transport",
      "file": "telemetry-onionnet-transport",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "transport"
      }
    },
    {
      "id": "telemetry.onionnet.queues",
      "file": "telemetry-onionnet-queues",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "queues"
      }
    },
    {
      "id": "telemetry.onionnet.probation",
      "file": "telemetry-onionnet-probation",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "probation"
      }
    },
    {
      "id": "telemetry.onionnet.ingress",
      "file": "telemetry-onionnet-ingress",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "onionnet",
        "onionnetTelemetryTab": "ingress"
      }
    },
    {
      "id": "telemetry.aggregators.overview",
      "file": "telemetry-aggregators-overview",
      "scope": "context",
      "match": {
        "view": "telemetry",
        "telemetrySource": "aggregators",
        "aggregatorsTelemetryTab": "overview"
      }
    },
    {
      "id": "asset.details",
      "file": "asset-details",
      "scope": "dialog",
      "match": {
        "dialog": "asset-detail"
      }
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
      "app.home": {
        "id": "app.home",
        "title": "Home",
        "summary": "Home brings together the selected wallet balance, private actions, and recent events.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review the selected wallet, then open Send, Receive, History, or an item that needs attention.",
                  "Wallet names and values belong to the selected local wallet profile."
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
                  "Unavailable capabilities stay labelled and no live balance or route is invented.",
                  "This Help is packaged with the application and works offline."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Staking",
        "summary": "Staking explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review available, staked, and reward amounts; choose an amount and validator only after chain verification.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum preferences",
        "summary": "Reticulum preferences explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review the local carrier service, interface mode, and Reticulum network identity settings.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet preferences",
        "summary": "OnionNet preferences explains the controls and status shown in this view.",
        "scope": "context",
        "sections": [
          {
            "title": "Use this view",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Review privacy mode, membership and replay checks, and route-age controls above the carrier.",
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
      "app.home": {
        "id": "app.home",
        "title": "Главная",
        "summary": "Главная объединяет баланс выбранного кошелька, приватные действия и последние события.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте выбранный кошелёк, затем откройте Отправить, Получить, Историю или требующее внимания событие.",
                  "Название и значения принадлежат выбранному локальному профилю кошелька."
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
                  "Недоступные возможности явно отмечены; приложение не выдумывает баланс или маршрут.",
                  "Справка встроена в приложение и работает без интернета."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Стейкинг",
        "summary": "Стейкинг: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте доступную, застейканную сумму и награды; выбирайте сумму и валидатора только после проверки сети.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Настройки Reticulum",
        "summary": "Настройки Reticulum: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте настройки локальной службы транспорта, режима интерфейсов и сетевой идентичности Reticulum.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "Настройки OnionNet",
        "summary": "Настройки OnionNet: доступные действия и состояния этого экрана.",
        "scope": "context",
        "sections": [
          {
            "title": "Как использовать экран",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Проверьте режим приватности, контроль membership/replay и возраст маршрута поверх транспорта.",
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
      "app.home": {
        "id": "app.home",
        "title": "Accueil",
        "summary": "L’accueil regroupe le solde du portefeuille sélectionné, les actions privées et les événements récents.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez le portefeuille sélectionné, puis ouvrez Envoyer, Recevoir, Historique ou un élément nécessitant votre attention.",
                  "Les noms et les valeurs appartiennent au profil de portefeuille local sélectionné."
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
                  "Les fonctions indisponibles restent signalées et aucun solde ou itinéraire réel n’est inventé.",
                  "Cette aide est intégrée à l’application et fonctionne hors ligne."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Staking",
        "summary": "Staking explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Vérifiez les montants disponibles, mis en jeu et récompensés ; choisissez montant et validateur après contrôle de chaîne.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Paramètres Reticulum",
        "summary": "Paramètres Reticulum explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez le service de transport local, le mode des interfaces et l’identité réseau Reticulum.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "Paramètres OnionNet",
        "summary": "Paramètres OnionNet explique les commandes et les états de cette vue.",
        "scope": "context",
        "sections": [
          {
            "title": "Utiliser cette vue",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Consultez le mode de confidentialité, les contrôles d’adhésion et de rejeu, et l’âge de la route.",
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
      "app.home": {
        "id": "app.home",
        "title": "Start",
        "summary": "Die Startansicht bündelt den Kontostand der gewählten Wallet, private Aktionen und aktuelle Ereignisse.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie die gewählte Wallet und öffnen Sie dann Senden, Empfangen, Verlauf oder einen Eintrag mit Handlungsbedarf.",
                  "Namen und Werte gehören zum ausgewählten lokalen Wallet-Profil."
                ]
              }
            ]
          },
          {
            "title": "Lokal und sicher",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Nicht verfügbare Funktionen bleiben gekennzeichnet; kein Kontostand oder Pfad wird erfunden.",
                  "Diese Hilfe ist in der Anwendung enthalten und funktioniert offline."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Staking",
        "summary": "Staking erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie verfügbare, eingesetzte und belohnte Beträge; wählen Sie Betrag und Validator erst nach Chain-Prüfung.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum-Einstellungen",
        "summary": "Reticulum-Einstellungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie lokalen Carrier-Dienst, Schnittstellenmodus und Reticulum-Netzwerkidentität.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet-Einstellungen",
        "summary": "OnionNet-Einstellungen erklärt die Bedienelemente und Zustände dieser Ansicht.",
        "scope": "context",
        "sections": [
          {
            "title": "Diese Ansicht verwenden",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Prüfen Sie Datenschutzmodus, Mitgliedschafts-/Replay-Kontrollen und Routenalter über dem Carrier.",
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
      "app.home": {
        "id": "app.home",
        "title": "Inicio",
        "summary": "Inicio reúne el saldo de la cartera seleccionada, acciones privadas y eventos recientes.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise la cartera seleccionada y abra Enviar, Recibir, Historial o un elemento que requiera atención.",
                  "Los nombres y valores pertenecen al perfil local de cartera seleccionado."
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
                  "Las funciones no disponibles permanecen indicadas y no se inventan saldos ni rutas.",
                  "Esta ayuda se incluye en la aplicación y funciona sin conexión."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Staking",
        "summary": "Staking explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise importes disponibles, apostados y recompensas; elija importe y validador tras verificar la cadena.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Ajustes de Reticulum",
        "summary": "Ajustes de Reticulum explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el servicio de transporte local, el modo de interfaces y la identidad de red Reticulum.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "Ajustes de OnionNet",
        "summary": "Ajustes de OnionNet explica los controles y estados de esta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Revise el modo de privacidad, los controles de membresía y repetición y la antigüedad de la ruta.",
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
      "app.home": {
        "id": "app.home",
        "title": "Início",
        "summary": "O início reúne o saldo da carteira selecionada, ações privadas e eventos recentes.",
        "scope": "context",
        "sections": [
          {
            "title": "Usar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Verifique a carteira selecionada e abra Enviar, Receber, Histórico ou um item que precisa de atenção.",
                  "Os nomes e valores pertencem ao perfil local de carteira selecionado."
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
                  "Recursos indisponíveis permanecem identificados e nenhum saldo ou rota é inventado.",
                  "Esta ajuda acompanha o aplicativo e funciona offline."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Staking",
        "summary": "Staking explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja valores disponíveis, em staking e recompensas; escolha valor e validador após verificar a cadeia.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Definições do Reticulum",
        "summary": "Definições do Reticulum explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o serviço de transporte local, o modo das interfaces e a identidade de rede Reticulum.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "Definições do OnionNet",
        "summary": "Definições do OnionNet explica os controlos e estados desta vista.",
        "scope": "context",
        "sections": [
          {
            "title": "Utilizar esta vista",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Reveja o modo de privacidade, os controlos de adesão e replay e a idade da rota.",
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
      "app.home": {
        "id": "app.home",
        "title": "홈",
        "summary": "홈은 선택한 지갑의 잔액, 비공개 작업, 최근 이벤트를 한곳에 모읍니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "선택한 지갑을 확인한 뒤 보내기, 받기, 기록 또는 주의가 필요한 항목을 여세요.",
                  "이름과 값은 선택한 로컬 지갑 프로필에 속합니다."
                ]
              }
            ]
          },
          {
            "title": "로컬 및 안전한 동작",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "사용할 수 없는 기능은 명확히 표시되며 잔액이나 경로를 임의로 만들지 않습니다.",
                  "이 도움말은 앱에 포함되어 오프라인에서도 작동합니다."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "스테이킹",
        "summary": "스테이킹 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "사용 가능액, 스테이킹액과 보상을 확인하고 체인 검증 후 금액과 검증자를 선택합니다.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum 설정",
        "summary": "Reticulum 설정 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "로컬 캐리어 서비스, 인터페이스 모드와 Reticulum 네트워크 ID 설정을 확인합니다.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet 설정",
        "summary": "OnionNet 설정 화면의 컨트롤과 상태를 설명합니다.",
        "scope": "context",
        "sections": [
          {
            "title": "이 화면 사용",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "캐리어 위의 개인정보 모드, 멤버십·재생 검사와 경로 수명 컨트롤을 확인합니다.",
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
      "app.home": {
        "id": "app.home",
        "title": "Ana sayfa",
        "summary": "Ana sayfa seçili cüzdan bakiyesini, özel işlemleri ve son olayları bir araya getirir.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanın",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Seçili cüzdanı kontrol edin; ardından Gönder, Al, Geçmiş veya dikkat gerektiren bir öğeyi açın.",
                  "Adlar ve değerler seçili yerel cüzdan profiline aittir."
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
                  "Kullanılamayan yetenekler işaretli kalır; bakiye veya rota uydurulmaz.",
                  "Bu yardım uygulamayla birlikte gelir ve çevrimdışı çalışır."
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "Stake etme",
        "summary": "Stake etme, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Kullanılabilir, stake edilmiş ve ödül tutarlarını inceleyin; zincir doğrulamasından sonra miktar ve doğrulayıcı seçin.",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum ayarları",
        "summary": "Reticulum ayarları, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Yerel taşıyıcı hizmetini, arayüz modunu ve Reticulum ağ kimliğini inceleyin.",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet ayarları",
        "summary": "OnionNet ayarları, bu görünümdeki denetimleri ve durumları açıklar.",
        "scope": "context",
        "sections": [
          {
            "title": "Bu görünümü kullanma",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "Taşıyıcı üzerindeki gizlilik modunu, üyelik/yeniden oynatma kontrollerini ve rota yaşını inceleyin.",
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
      "app.home": {
        "id": "app.home",
        "title": "ホーム",
        "summary": "ホームには、選択したウォレットの残高、非公開アクション、最近のイベントがまとまっています。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "選択したウォレットを確認し、送信、受信、履歴、または対応が必要な項目を開きます。",
                  "名前と値は、選択したローカルウォレットプロファイルに属します。"
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
                  "利用できない機能は明示され、残高や経路を推測して表示しません。",
                  "このヘルプはアプリに同梱され、オフラインで動作します。"
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "ステーキング",
        "summary": "ステーキングの操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "利用可能額、ステーク額、報酬を確認し、チェーン検証後に金額とバリデータを選択します。",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum設定",
        "summary": "Reticulum設定の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "ローカルキャリアサービス、インターフェースモード、Reticulum ネットワーク ID を確認します。",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet設定",
        "summary": "OnionNet設定の操作と状態を説明します。",
        "scope": "context",
        "sections": [
          {
            "title": "この画面の使い方",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "キャリア上のプライバシーモード、メンバーシップ／リプレイ検査、経路の経過時間を確認します。",
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
      "app.home": {
        "id": "app.home",
        "title": "首页",
        "summary": "首页集中显示所选钱包的余额、私密操作和最近事件。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此页面",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "检查所选钱包，然后打开发送、接收、历史记录或需要关注的项目。",
                  "名称和值属于所选的本地钱包配置。"
                ]
              }
            ]
          },
          {
            "title": "本地且安全",
            "target": "",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "不可用的功能会明确标注，应用不会虚构余额或路由。",
                  "此帮助随应用打包，可离线使用。"
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
      "wallet.staking": {
        "id": "wallet.staking",
        "title": "质押",
        "summary": "质押说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看可用、已质押和奖励金额；仅在链验证后选择金额和验证者。",
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
      "settings.reticulum": {
        "id": "settings.reticulum",
        "title": "Reticulum 设置",
        "summary": "Reticulum 设置说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看本地承载服务、接口模式和 Reticulum 网络身份设置。",
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
      "settings.onionnet": {
        "id": "settings.onionnet",
        "title": "OnionNet 设置",
        "summary": "OnionNet 设置说明此视图中的控件和状态。",
        "scope": "context",
        "sections": [
          {
            "title": "使用此视图",
            "target": "current-view",
            "blocks": [
              {
                "type": "list",
                "items": [
                  "查看承载层之上的隐私模式、成员与重放检查以及路径时长控件。",
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
      }
    }
  }
});
})(typeof window === "undefined" ? globalThis : window);
