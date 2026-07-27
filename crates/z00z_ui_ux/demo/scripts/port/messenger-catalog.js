"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT) {
    throw new Error("Z00Z contracts must load before Messenger fixtures.");
  }

  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };

  const MESSENGER_REQUEST_TYPES = deepFreeze([
    "payment_request",
    "voucher_proposal",
    "claim_proposal",
    "permission_proposal",
    "receiver_card_invitation"
  ]);

  const request = ({
    type,
    objectFamily,
    action,
    exactScope,
    value = "None",
    fee = "Separate Wallet review",
    walletRoute,
    walletFlow,
    itemKey = null
  }) => deepFreeze({
    schemaVersion: "messenger-request-v1",
    type,
    objectFamily,
    action,
    exactScope,
    value,
    fee,
    walletTarget: {
      routeId: walletRoute,
      flow: walletFlow,
      itemKey
    },
    requiresWalletReview: true,
    recipientReference: "Withheld until Wallet review"
  });

  const MESSENGER_MESSAGES = deepFreeze([
    {
      id: "message_advisory_001",
      kind: "advisory",
      folders: ["inbox"],
      senderLabel: "Local wallet notice",
      subject: "Backup check reminder",
      preview: "Review the selected wallet backup status when convenient.",
      createdAt: "2026-07-26T10:25:00Z",
      expiresAt: "2026-07-29T10:25:00Z",
      deliveryState: "delivered",
      severity: "info",
      request: null
    },
    {
      id: "message_payment_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Local merchant",
      subject: "Payment request",
      preview: "Review a bounded 18.50 Z00Z payment request.",
      createdAt: "2026-07-26T09:40:00Z",
      expiresAt: "2026-07-27T09:40:00Z",
      deliveryState: "delivered",
      severity: "action",
      request: request({
        type: "payment_request",
        objectFamily: "asset",
        action: "Prepare one native-asset payment",
        exactScope: "18.50 Z00Z · one recipient · one review",
        value: "18.50 Z00Z",
        walletRoute: "wallet.send",
        walletFlow: "send",
        itemKey: "z00z"
      })
    },
    {
      id: "message_voucher_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Community desk",
      subject: "Private voucher proposal",
      preview: "Inspect a bounded community voucher before any Wallet action.",
      createdAt: "2026-07-26T08:15:00Z",
      expiresAt: "2026-07-30T08:15:00Z",
      deliveryState: "delivered",
      severity: "action",
      request: request({
        type: "voucher_proposal",
        objectFamily: "voucher",
        action: "Inspect one community voucher proposal",
        exactScope: "Community class · one claim · expires 30 Jul 2026",
        value: "12.00 Z00Z conditional value",
        walletRoute: "wallet.vouchers",
        walletFlow: "voucher_inspection"
      })
    },
    {
      id: "message_claim_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Local issuer",
      subject: "Claim proposal",
      preview: "Inspect one bounded refund claim.",
      createdAt: "2026-07-25T18:35:00Z",
      expiresAt: "2026-07-28T18:35:00Z",
      deliveryState: "delivered",
      severity: "action",
      request: request({
        type: "claim_proposal",
        objectFamily: "claim",
        action: "Inspect one refund claim",
        exactScope: "Refund claim · one review · no delegation",
        value: "34.00 Z00Z conditional value",
        walletRoute: "wallet.vouchers",
        walletFlow: "claim_inspection"
      })
    },
    {
      id: "message_permission_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Operations group",
      subject: "Permission proposal",
      preview: "Review a one-use publication permission.",
      createdAt: "2026-07-25T16:10:00Z",
      expiresAt: "2026-08-02T00:00:00Z",
      deliveryState: "delivered",
      severity: "action",
      request: request({
        type: "permission_proposal",
        objectFamily: "permission",
        action: "Inspect one publication permission",
        exactScope: "Release note publication · one use · attenuation only",
        walletRoute: "wallet.permissions",
        walletFlow: "permission_inspection"
      })
    },
    {
      id: "message_receiver_card_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Known local contact",
      subject: "Receiver-card invitation",
      preview: "Review updated receiver material before replacing a local contact.",
      createdAt: "2026-07-25T13:05:00Z",
      expiresAt: "2026-07-27T13:05:00Z",
      deliveryState: "delivered",
      severity: "attention",
      request: request({
        type: "receiver_card_invitation",
        objectFamily: "receiver_card",
        action: "Review receiver-card identity change",
        exactScope: "One local contact record · confirmation required",
        walletRoute: "contacts.list",
        walletFlow: "contact_identity_review"
      })
    },
    {
      id: "message_expired_001",
      kind: "request",
      folders: ["inbox"],
      senderLabel: "Service desk",
      subject: "Expired service request",
      preview: "This advisory request expired without Wallet action.",
      createdAt: "2026-07-20T07:30:00Z",
      expiresAt: "2026-07-22T07:30:00Z",
      deliveryState: "expired",
      severity: "muted",
      request: request({
        type: "permission_proposal",
        objectFamily: "permission",
        action: "Inspect expired service access",
        exactScope: "Expired · zero usable authority",
        walletRoute: "wallet.permissions",
        walletFlow: "permission_inspection"
      })
    },
    {
      id: "message_abuse_001",
      kind: "abuse",
      folders: ["inbox"],
      senderLabel: "Unknown local sender",
      subject: "Unsolicited repeated request",
      preview: "Potential abuse. No request payload is opened automatically.",
      createdAt: "2026-07-26T06:50:00Z",
      expiresAt: "2026-07-27T06:50:00Z",
      deliveryState: "delivered",
      severity: "danger",
      request: null
    }
  ]);

  const MESSENGER_CONVERSATIONS = deepFreeze([
    {
      id: "conversation_001",
      label: "Community desk",
      preview: "Voucher details remain local and expire with this thread.",
      updatedAt: "2026-07-26T08:17:00Z",
      retention: "Expires in 48 hours",
      messageCount: 3
    },
    {
      id: "conversation_002",
      label: "Operations group",
      preview: "Publication permission scope was clarified.",
      updatedAt: "2026-07-25T16:18:00Z",
      retention: "Expires in 24 hours",
      messageCount: 2
    }
  ]);

  const MESSENGER_SENT = deepFreeze([
    { id: "outbox_queued", subject: "Receiver card", state: "queued", updatedAt: "2026-07-26T10:30:00Z", summary: "Stored locally; relay unavailable." },
    { id: "outbox_relayed", subject: "Payment acknowledgement", state: "relayed", updatedAt: "2026-07-26T09:45:00Z", summary: "Relay accepted the advisory envelope; delivery is not proven." },
    { id: "outbox_acknowledged", subject: "Voucher response", state: "acknowledged", updatedAt: "2026-07-26T08:25:00Z", summary: "Advisory acknowledgement received; settlement is independent." },
    { id: "outbox_expired", subject: "Old receiver card", state: "expired", updatedAt: "2026-07-24T13:00:00Z", summary: "Expired locally without retry." },
    { id: "outbox_failed", subject: "Permission response", state: "failed", updatedAt: "2026-07-25T16:20:00Z", summary: "Relay attempt failed; Wallet state is unchanged." }
  ]);

  const MESSENGER_RELAY_STATES = deepFreeze([
    {
      id: "available",
      availability: "degraded",
      summary: "Local deterministic fixture; no durable mailbox or live relay is connected."
    },
    {
      id: "unavailable",
      availability: "unavailable",
      summary: "Relay unavailable. Advisory items stay local and Wallet truth is unaffected."
    },
    {
      id: "recovering",
      availability: "degraded",
      summary: "Recovery check is local-only; no network connection is attempted."
    }
  ]);

  const MESSAGE_BY_ID = new Map(MESSENGER_MESSAGES.map((entry) => [entry.id, entry]));

  function messengerMessage(messageId) {
    return MESSAGE_BY_ID.get(messageId) || null;
  }

  function assertMessengerFixtures() {
    const ids = MESSENGER_MESSAGES.map(({ id }) => id);
    if (new Set(ids).size !== ids.length) throw new Error("Messenger fixture IDs must be unique.");
    const requestTypes = new Set(MESSENGER_MESSAGES.flatMap(({ request: entry }) => entry ? [entry.type] : []));
    for (const requestType of MESSENGER_REQUEST_TYPES) {
      if (!requestTypes.has(requestType)) throw new Error(`Missing Messenger request fixture: ${requestType}`);
    }
    if (!MESSENGER_MESSAGES.some(({ kind }) => kind === "advisory")
      || !MESSENGER_MESSAGES.some(({ kind }) => kind === "abuse")
      || !MESSENGER_MESSAGES.some(({ deliveryState }) => deliveryState === "expired")
      || !MESSENGER_RELAY_STATES.some(({ id }) => id === "unavailable")) {
      throw new Error("Messenger fixtures must cover advisory, abuse, expiry, and unavailable relay states.");
    }
    const serialized = JSON.stringify({
      messages: MESSENGER_MESSAGES,
      conversations: MESSENGER_CONVERSATIONS,
      sent: MESSENGER_SENT
    });
    if (/((?:https?:)?\/\/)|\b(?:receiver_secret|ack_secret|route_bucket|raw_package|compact_request|private_key|seed_phrase|locator)\b/i.test(serialized)) {
      throw new Error("Messenger fixtures contain forbidden raw protocol or remote material.");
    }
    return true;
  }

  assertMessengerFixtures();

  Object.assign(root.Z00ZDemo, {
    MESSENGER_REQUEST_TYPES,
    MESSENGER_MESSAGES,
    MESSENGER_CONVERSATIONS,
    MESSENGER_SENT,
    MESSENGER_RELAY_STATES,
    messengerMessage,
    assertMessengerFixtures
  });
})(typeof window === "undefined" ? globalThis : window);
