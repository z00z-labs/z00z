import assert from "node:assert/strict";
import { readFile, stat } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const demoRoot = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const context = vm.createContext({
  URLSearchParams,
  structuredClone,
  window: {}
});
const modules = [
  "scripts/port/contracts.js",
  "scripts/port/navigation-model.js",
  "scripts/port/exchange-catalog.js",
  "scripts/port/dapp-catalog.js",
  "scripts/port/messenger-catalog.js",
  "scripts/port/contacts-catalog.js",
  "scripts/port/fixtures.js",
  "scripts/port/presentation-state.js",
  "scripts/port/mock-wallet-gateway.js",
  "scripts/port/mock-telemetry-gateway.js",
  "scripts/port/mock-dapp-gateway.js",
  "scripts/port/mock-messenger-gateway.js",
  "scripts/port/mock-contacts-gateway.js",
  "scripts/port/icon-registry.js",
  "scripts/port/locale-registry.js",
  "scripts/generated/help-catalog.js",
  "scripts/port/help-registry.js"
];

for (const modulePath of modules) {
  const source = await readFile(resolve(demoRoot, modulePath), "utf8");
  vm.runInContext(source, context, { filename: modulePath });
}

const demo = context.window.Z00ZDemo;
const locales = context.window.Z00ZLocaleRegistry;

assert.equal(demo.PORT_CONTRACT.rendererRuntime, "leptos-csr-wasm");
assert.equal(demo.PORT_CONTRACT.packagedHost, "tauri-2");
assert.equal(demo.PORT_CONTRACT.browserProduct, false);
assert.equal(demo.PORT_CONTRACT.walletBackendRuntime, "native-rust");
assert.ok(demo.PORT_CONTRACT.rendererForbiddenState.includes("session_token"));
assert.ok(demo.PORT_CONTRACT.forbiddenTransports.includes("websocket"));
assert.equal(demo.PORT_CONTRACT.capabilityStates, undefined);
assert.equal(demo.PORT_CONTRACT.routes.length, 63);
assert.equal(demo.APP_VERSION, "0.1.0");
assert.equal(demo.PORT_CONTRACT.appVersion, demo.APP_VERSION);
assert.match(
  await readFile(resolve(demoRoot, "../../..", "Cargo.toml"), "utf8"),
  new RegExp(`\\[workspace\\.package\\][\\s\\S]*?version\\s*=\\s*"${demo.APP_VERSION.replaceAll(".", "\\.")}"`)
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.palettes),
  ["z00z-default", "z00z-corporate"]
);
assert.deepEqual(
  Array.from(demo.PALETTE_OPTIONS, ({ id, colorScheme }) => ({ id, colorScheme })),
  [
    { id: "z00z-default", colorScheme: "dark" },
    { id: "z00z-corporate", colorScheme: "light" }
  ]
);
assert.equal(demo.resolveInitialPalettePreference("?palette=z00z-corporate&theme=dark"), "z00z-corporate");
assert.equal(demo.resolveInitialPalettePreference("?palette=removed-palette&theme=light"), "z00z-corporate");
assert.equal(demo.resolveInitialPalettePreference("?palette=removed-palette&theme=dark"), "z00z-default");
assert.equal(demo.resolveInitialPalettePreference("?palette=invalid&theme=invalid"), "z00z-default");
assert.deepEqual(
  Object.keys(demo.PORT_CONTRACT.defaultRouteByNamespace),
  Array.from(demo.PORT_CONTRACT.routeNamespaces)
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.walletChains),
  ["mainnet", "testnet-1", "testnet-2", "devnet-1", "devnet-2"]
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.telemetryTabs.aggregators),
  ["overview", "ingress", "planning", "placement", "publication", "recovery"]
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.telemetryTabs.watchers),
  ["overview", "alerts", "publication", "providers", "censorship", "evidence"]
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.telemetryTabs.explorer),
  ["overview", "search", "checkpoints", "batches", "evidence"]
);
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.telemetrySources),
  ["onionnet", "reticulum", "aggregators", "watchers", "explorer"]
);
assert.deepEqual(Object.keys(demo.EXCHANGE_PROVIDER_LUT), ["hyperliquid", "near-intents"]);
assert.equal(demo.exchangeProvider("unknown").id, "near-intents");
assert.deepEqual(Array.from(demo.exchangeDestinations("hyperliquid"), ({ id }) => id), [
  "hyperliquid-usdc",
  "hyperliquid-hype",
  "hyperliquid-btc"
]);
assert.ok(Object.isFrozen(demo.EXCHANGE_PROVIDER_LUT));
assert.deepEqual(Array.from(demo.DAPP_DESCRIPTOR_IDS), [
  "offline-pay",
  "private-voucher",
  "external-asset-locker",
  "scoped-expenses",
  "service-credits",
  "agent-budget"
]);
assert.equal(demo.DAPP_CATALOG.length, 6);
assert.equal(demo.assertDappCatalog(), true);
assert.ok(Object.isFrozen(demo.DAPP_CATALOG));
assert.ok(demo.DAPP_CATALOG.every((entry) => Object.isFrozen(entry)));
assert.equal(new Set(Array.from(demo.DAPP_CATALOG, ({ id }) => id)).size, 6);
assert.equal(new Set(Array.from(demo.DAPP_CATALOG, ({ iconName }) => iconName)).size, 6);
assert.equal(new Set(Array.from(demo.DAPP_CATALOG, ({ intentType }) => intentType)).size, 6);
assert.ok(demo.DAPP_CATALOG.every((entry) => entry.availability === "unavailable"));
assert.ok(demo.DAPP_CATALOG.every((entry) => entry.presentationMode === "roadmap_preview"));
assert.ok(demo.DAPP_CATALOG.every((entry) => entry.publisher.verified === false));
assert.ok(demo.DAPP_CATALOG.every((entry) => entry.executionBoundary === "typed_intent_only"));
assert.ok(demo.DAPP_CATALOG.every((entry) => !entry.remoteCodeAllowed && !entry.walletBridgeAllowed));
assert.ok(demo.DAPP_CATALOG.every((entry) => entry.requestedObjectFamilies.every(
  (family) => demo.DAPP_OBJECT_FAMILIES.includes(family)
)));
assert.equal(demo.dappDescriptor("unknown"), null);
assert.equal(demo.dappDescriptor("agent-budget").maturity, "concept");
assert.equal(demo.dappDescriptor("agent-budget").valuePath, "separate_wallet_review");
assert.equal(demo.dappDescriptor("agent-budget").feePath, "separate_wallet_review");
assert.match(demo.dappDescriptor("agent-budget").reviewBoundary, /No autonomous execution/);
assert.deepEqual(
  Array.from(demo.DAPP_CONNECTION_FIXTURES, ({ id, descriptorId, status }) => ({ id, descriptorId, status })),
  [
    { id: "connection_offline_pay", descriptorId: "offline-pay", status: "pending" },
    { id: "connection_scoped_expenses", descriptorId: "scoped-expenses", status: "active" },
    { id: "connection_service_credits", descriptorId: "service-credits", status: "expired" }
  ]
);
assert.deepEqual(
  Array.from(demo.DAPP_PERMISSION_FIXTURES, ({ status }) => status),
  ["active", "expiring", "expired"]
);
assert.equal(demo.DAPP_ACTIVITY_FIXTURES.length, 4);
assert.ok(Object.isFrozen(demo.DAPP_CONNECTION_FIXTURES));
assert.ok(Object.isFrozen(demo.DAPP_PERMISSION_FIXTURES));
assert.ok(Object.isFrozen(demo.DAPP_ACTIVITY_FIXTURES));
assert.ok(demo.DAPP_CONNECTION_FIXTURES.every((entry) => demo.dappDescriptor(entry.descriptorId)));
assert.ok(demo.DAPP_PERMISSION_FIXTURES.every((entry) => demo.dappDescriptor(entry.descriptorId)));
assert.ok(demo.DAPP_ACTIVITY_FIXTURES.every((entry) => demo.dappDescriptor(entry.descriptorId)));
assert.doesNotMatch(
  JSON.stringify(demo.DAPP_CATALOG),
  /((?:https?:)?\/\/)|\b(?:url|domain|iframe|bundle|executable|sourceCode)\b/i
);
const walletFixturesBeforeDappReview = JSON.stringify(demo.INITIAL_WALLET_FIXTURES);
const dappGateway = demo.createMockDappGateway();
const offlinePayReviewResult = dappGateway.readPermissionReview({
  connectionId: "connection_offline_pay"
});
assert.equal(offlinePayReviewResult.ok, true);
assert.equal(offlinePayReviewResult.data.schemaVersion, "dapp-permission-review-v1");
assert.equal(offlinePayReviewResult.data.intent.type, "prepare_offline_payment");
assert.equal(offlinePayReviewResult.data.permission.objectFamily, "asset");
assert.equal(offlinePayReviewResult.data.permission.exactScope, "Native Z00Z asset · one prepared handoff");
assert.equal(offlinePayReviewResult.data.permission.uses, "1 use");
assert.equal(offlinePayReviewResult.data.permission.expiry, "2026-07-27T09:15:00Z");
assert.equal(offlinePayReviewResult.data.permission.delegation, "Not allowed");
assert.equal(offlinePayReviewResult.data.value.display, "24.00 Z00Z");
assert.equal(offlinePayReviewResult.data.fee.display, "0.001 Z00Z");
assert.equal(offlinePayReviewResult.data.fee.path, "Separate wallet review");
assert.deepEqual(Array.from(offlinePayReviewResult.data.disclosures), [
  "asset family",
  "bounded value",
  "expiry",
  "recipient commitment"
]);
assert.match(offlinePayReviewResult.data.revoke.behavior, /Cancel the unused local approval/);
assert.equal(offlinePayReviewResult.data.reauth.required, true);
assert.equal(offlinePayReviewResult.data.reauth.authority, "wallet_review_only");
assert.deepEqual(
  Object.fromEntries(Object.entries(offlinePayReviewResult.data.boundary)),
  {
    genericSigning: false,
    arbitraryPayload: false,
    walletMutation: false,
    remoteExecution: false
  }
);
assert.ok(Object.isFrozen(offlinePayReviewResult));
assert.ok(Object.isFrozen(offlinePayReviewResult.data));
assert.deepEqual(
  dappGateway.readPermissionReview({ connectionId: "connection_offline_pay" }),
  offlinePayReviewResult
);
const missingScopeDecision = dappGateway.decidePermissionReview({
  reviewId: offlinePayReviewResult.data.reviewId,
  decision: "accepted"
});
assert.equal(missingScopeDecision.ok, false);
assert.equal(missingScopeDecision.error.code, "scope_confirmation_required");
assert.equal(missingScopeDecision.error.message, "Confirm the exact displayed scope before accepting.");
assert.equal(
  dappGateway.decidePermissionReview({
    reviewId: offlinePayReviewResult.data.reviewId,
    decision: "accepted",
    scopeConfirmed: true
  }).error.code,
  "reauth_acknowledgement_required"
);
const acceptedDappReview = dappGateway.decidePermissionReview({
  reviewId: offlinePayReviewResult.data.reviewId,
  decision: "accepted",
  scopeConfirmed: true,
  reauthAcknowledged: true
});
assert.equal(acceptedDappReview.ok, true);
assert.equal(acceptedDappReview.data.decision, "accepted");
assert.equal(acceptedDappReview.data.connectionId, "connection_offline_pay");
assert.equal(acceptedDappReview.data.walletReviewRequired, true);
assert.equal(acceptedDappReview.data.walletMutation, null);
assert.deepEqual(
  Object.fromEntries(Object.entries(acceptedDappReview.data.intentReference)),
  {
    type: "prepare_offline_payment",
    descriptorId: "offline-pay",
    reviewId: "review_connection_offline_pay"
  }
);
const rejectedDappReview = dappGateway.decidePermissionReview({
  reviewId: offlinePayReviewResult.data.reviewId,
  decision: "rejected"
});
assert.equal(rejectedDappReview.ok, true);
assert.equal(rejectedDappReview.data.walletReviewRequired, false);
assert.equal(rejectedDappReview.data.walletMutation, null);
const walletReviewHandoff = dappGateway.prepareWalletReview({
  decision: acceptedDappReview.data
});
assert.equal(walletReviewHandoff.ok, true);
assert.equal(walletReviewHandoff.data.schemaVersion, "dapp-wallet-review-handoff-v1");
assert.equal(walletReviewHandoff.data.source.intentType, "prepare_offline_payment");
assert.equal(walletReviewHandoff.data.target.routeId, "wallet.send");
assert.equal(walletReviewHandoff.data.target.flow, "send");
assert.deepEqual(
  Object.fromEntries(Object.entries(walletReviewHandoff.data.draft)),
  {
    family: "asset",
    itemKey: "z00z",
    amount: "24.00",
    recipient: "",
    memo: ""
  }
);
assert.deepEqual(
  Object.fromEntries(Object.entries(walletReviewHandoff.data.constraints)),
  {
    prefillOnly: true,
    recipientRequired: true,
    walletReviewRequired: true,
    walletMutation: false
  }
);
assert.ok(Object.isFrozen(walletReviewHandoff));
assert.ok(Object.isFrozen(walletReviewHandoff.data));
assert.equal(
  dappGateway.prepareWalletReview({ decision: rejectedDappReview.data }).error.code,
  "decision_not_accepted"
);
assert.equal(
  dappGateway.prepareWalletReview({
    decision: { ...acceptedDappReview.data, descriptorId: "agent-budget" }
  }).error.code,
  "invalid_decision"
);
const offlinePayProposal = dappGateway.readIntentProposal({
  connectionId: "connection_offline_pay"
});
const offlinePayHeldAuthority = dappGateway.readHeldAuthority({
  connectionId: "connection_offline_pay"
});
assert.equal(offlinePayProposal.ok, true);
assert.equal(offlinePayHeldAuthority.ok, true);
assert.equal(offlinePayProposal.data.schemaVersion, "dapp-intent-proposal-v1");
assert.equal(offlinePayHeldAuthority.data.schemaVersion, "dapp-held-authority-v1");
assert.ok(Object.isFrozen(offlinePayProposal.data));
assert.ok(Object.isFrozen(offlinePayHeldAuthority.data));
const safeIntentValidation = dappGateway.validateIntentProposal({
  proposal: offlinePayProposal.data,
  heldAuthority: offlinePayHeldAuthority.data
});
assert.equal(safeIntentValidation.ok, true);
assert.equal(safeIntentValidation.data.schemaVersion, "dapp-intent-validation-v1");
assert.equal(safeIntentValidation.data.result, "accepted_for_wallet_review");
assert.deepEqual(
  dappGateway.prepareWalletReview({
    decision: acceptedDappReview.data,
    proposal: offlinePayProposal.data,
    heldAuthority: offlinePayHeldAuthority.data
  }),
  walletReviewHandoff
);

const maliciousDappProposalCases = [
  {
    name: "generic signing",
    expectedCode: "generic_signing_forbidden",
    mutate(proposal) {
      proposal.boundary.genericSigning = true;
      proposal.payloadToSign = "opaque wallet message";
    }
  },
  {
    name: "arbitrary URL",
    expectedCode: "arbitrary_url_forbidden",
    mutate(proposal) {
      proposal.callbackUrl = "https://untrusted.invalid/callback";
    }
  },
  {
    name: "unknown intent type",
    expectedCode: "unknown_intent_type",
    mutate(proposal) {
      proposal.intent.type = "sign_anything";
    }
  },
  {
    name: "broader-than-held permission scope",
    expectedCode: "permission_exceeds_held_authority",
    mutate(proposal) {
      proposal.permission.exactScope = "All assets · all wallets · unlimited handoffs";
    }
  },
  {
    name: "broader-than-held permission uses",
    expectedCode: "permission_exceeds_held_authority",
    mutate(proposal) {
      proposal.permission.uses = "Unlimited";
    }
  },
  {
    name: "broader-than-held permission expiry",
    expectedCode: "permission_exceeds_held_authority",
    mutate(proposal) {
      proposal.permission.expiry = "2099-12-31T23:59:59Z";
    }
  },
  {
    name: "broader-than-held permission delegation",
    expectedCode: "permission_exceeds_held_authority",
    mutate(proposal) {
      proposal.permission.delegation = "Unrestricted";
    }
  },
  {
    name: "hidden value",
    expectedCode: "hidden_value_forbidden",
    mutate(proposal) {
      proposal.value.present = false;
      proposal.value.display = "";
    }
  },
  {
    name: "hidden fee",
    expectedCode: "hidden_fee_forbidden",
    mutate(proposal) {
      proposal.fee.present = false;
      proposal.fee.display = "";
      proposal.fee.path = "Undisclosed";
    }
  },
  {
    name: "remote resource loading",
    expectedCode: "remote_resource_forbidden",
    mutate(proposal) {
      proposal.boundary.remoteResourceLoading = true;
      proposal.remoteResource = "third-party executable package";
    }
  }
];
for (const { name, expectedCode, mutate } of maliciousDappProposalCases) {
  const proposal = structuredClone(offlinePayProposal.data);
  mutate(proposal);
  const result = dappGateway.prepareWalletReview({
    decision: acceptedDappReview.data,
    proposal,
    heldAuthority: offlinePayHeldAuthority.data
  });
  assert.equal(result.ok, false, `${name} must fail closed`);
  assert.equal(result.error.code, expectedCode, `${name} must use a specific rejection code`);
}
const broadenedHeldAuthority = structuredClone(offlinePayHeldAuthority.data);
broadenedHeldAuthority.permission.uses = "Unlimited";
assert.equal(
  dappGateway.prepareWalletReview({
    decision: acceptedDappReview.data,
    proposal: offlinePayProposal.data,
    heldAuthority: broadenedHeldAuthority
  }).error.code,
  "invalid_held_authority"
);
assert.ok(Object.isFrozen(demo.DAPP_INTENT_PROPOSALS));
assert.ok(Object.isFrozen(demo.DAPP_HELD_AUTHORITIES));
assert.equal(JSON.stringify(demo.INITIAL_WALLET_FIXTURES), walletFixturesBeforeDappReview);

assert.equal(demo.assertMessengerFixtures(), true);
assert.deepEqual(Array.from(demo.MESSENGER_REQUEST_TYPES), [
  "payment_request",
  "voucher_proposal",
  "claim_proposal",
  "permission_proposal",
  "receiver_card_invitation"
]);
assert.ok(Object.isFrozen(demo.MESSENGER_MESSAGES));
assert.ok(Object.isFrozen(demo.MESSENGER_CONVERSATIONS));
assert.ok(Object.isFrozen(demo.MESSENGER_SENT));
assert.ok(demo.MESSENGER_RELAY_STATES.some(({ id, availability }) => id === "unavailable" && availability === "unavailable"));
assert.doesNotMatch(
  JSON.stringify(demo.MESSENGER_MESSAGES),
  /((?:https?:)?\/\/)|\b(?:receiver_secret|ack_secret|route_bucket|raw_package|compact_request|private_key|seed_phrase|locator)\b/i
);
const messengerGateway = demo.createMockMessengerGateway();
assert.equal(messengerGateway.contractVersion, demo.PORT_CONTRACT.version);
assert.equal(messengerGateway.listMessages({ folder: "inbox" }).data.items.length, 8);
const paymentRequestReview = messengerGateway.readRequestReview({ messageId: "message_payment_001" });
assert.equal(paymentRequestReview.ok, true);
assert.equal(paymentRequestReview.data.schemaVersion, "messenger-request-review-v1");
assert.equal(paymentRequestReview.data.request.type, "payment_request");
assert.equal(paymentRequestReview.data.request.value, "18.50 Z00Z");
assert.equal(paymentRequestReview.data.boundary.walletMutation, false);
assert.equal(paymentRequestReview.data.boundary.settlementMutation, false);
const acceptedMessengerRequest = messengerGateway.decideRequest({
  reviewId: paymentRequestReview.data.reviewId,
  decision: "accepted"
});
assert.equal(acceptedMessengerRequest.ok, true);
assert.equal(acceptedMessengerRequest.data.walletMutation, null);
assert.equal(acceptedMessengerRequest.data.settlementMutation, null);
const messengerWalletHandoff = messengerGateway.prepareWalletReview({
  decision: acceptedMessengerRequest.data
});
assert.equal(messengerWalletHandoff.ok, true);
assert.equal(messengerWalletHandoff.data.schemaVersion, "messenger-wallet-review-handoff-v1");
assert.equal(messengerWalletHandoff.data.target.routeId, "wallet.send");
assert.equal(messengerWalletHandoff.data.draft.amount, "18.50");
assert.equal(messengerWalletHandoff.data.draft.recipient, "");
assert.equal(messengerWalletHandoff.data.constraints.walletRevalidationRequired, true);
const messengerWalletState = demo.createInitialState({ search: "?route=messenger.inbox" });
const messengerWalletSnapshot = JSON.stringify(messengerWalletState.wallets);
const messengerWalletGateway = demo.createMockWalletGateway(messengerWalletState);
const revalidatedMessengerHandoff = messengerWalletGateway.revalidateExternalReviewHandoff({
  walletId: "everyday",
  handoff: messengerWalletHandoff.data
});
assert.equal(revalidatedMessengerHandoff.ok, true);
assert.equal(revalidatedMessengerHandoff.data.result, "accepted_for_wallet_entry");
assert.equal(revalidatedMessengerHandoff.data.walletMutation, false);
const broadenedMessengerHandoff = structuredClone(messengerWalletHandoff.data);
broadenedMessengerHandoff.draft.recipient = "injected-recipient";
assert.equal(
  messengerWalletGateway.revalidateExternalReviewHandoff({
    walletId: "everyday",
    handoff: broadenedMessengerHandoff
  }).error.code,
  "external_review_rejected"
);
for (const action of ["opened", "acknowledged", "deleted", "blocked", "reported"]) {
  const result = messengerGateway.advisoryAction({
    messageId: "message_abuse_001",
    action
  });
  assert.equal(result.ok, true);
  assert.equal(result.data.walletMutation, false);
  assert.equal(result.data.settlementMutation, false);
}
const expiredRequestReview = messengerGateway.readRequestReview({ messageId: "message_expired_001" });
assert.equal(
  messengerGateway.decideRequest({
    reviewId: expiredRequestReview.data.reviewId,
    decision: "accepted"
  }).error.code,
  "request_expired"
);
assert.equal(JSON.stringify(messengerWalletState.wallets), messengerWalletSnapshot);

assert.equal(demo.assertContactFixtures(), true);
assert.deepEqual(Array.from(demo.CONTACT_STATUS_IDS), [
  "known_locally",
  "needs_confirmation",
  "identity_changed",
  "expired",
  "revoked"
]);
assert.ok(Object.isFrozen(demo.CONTACT_FIXTURES));
assert.ok(demo.CONTACT_FIXTURES.every((contact) => new Set([
  contact.contactIdentityKey,
  contact.reticulumDestinationRef,
  contact.walletRecipientRef
]).size === 3));
assert.doesNotMatch(
  JSON.stringify(demo.CONTACT_FIXTURES),
  /((?:https?:)?\/\/)|\b(?:public_presence|social_graph|trust_score|phone|email|raw_address|private_key|seed_phrase)\b/i
);
const contactsState = demo.createInitialState({ search: "?route=contacts.list" });
const contactsWalletSnapshot = JSON.stringify(contactsState.wallets);
const contactsGateway = demo.createMockContactsGateway(contactsState);
const contactsWalletGateway = demo.createMockWalletGateway(contactsState);
assert.equal(contactsGateway.listContacts().data.items.length, 6);
assert.deepEqual(
  Array.from(contactsGateway.listContacts({ sort: "nickname" }).data.items, ({ label }) => label),
  ["Ada", "Ben", "Community desk", "Old service", "Operations", "Revoked card"]
);
assert.deepEqual(
  Array.from(contactsGateway.listContacts({ sort: "date" }).data.items, ({ id }) => id),
  ["contact_ada", "contact_ben", "contact_community", "contact_ops", "contact_old_service", "contact_revoked"]
);
assert.equal(contactsGateway.listContacts({ query: "voucher" }).data.items.length, 1);
assert.equal(contactsGateway.listContacts({ status: "identity_changed" }).data.items.length, 1);
assert.equal(contactsGateway.listContacts().data.networkRequest, false);
assert.equal(
  contactsGateway.addContact({ sourceId: "qr_scan", label: "Scanned contact" }).error.code,
  "native_boundary_unavailable"
);
const addedContact = contactsGateway.addContact({
  sourceId: "manual",
  label: "Local demo",
  safeNote: "Reviewed public material"
});
assert.equal(addedContact.ok, true);
assert.equal(addedContact.data.localOnly, true);
assert.equal(addedContact.data.networkRequest, false);
assert.equal(contactsState.contacts.length, 7);
const editedContact = contactsGateway.editLabel({
  contactId: addedContact.data.contact.id,
  label: "Local demo edited",
  safeNote: "Still local"
});
assert.equal(editedContact.data.contact.label, "Local demo edited");
const identityReview = contactsGateway.reviewIdentityChange({
  contactId: "contact_ops",
  decision: "accepted"
});
assert.equal(identityReview.ok, true);
assert.equal(identityReview.data.implicitTrust, false);
assert.equal(identityReview.data.walletMutation, false);
const payContactHandoff = contactsGateway.prepareAction({
  contactId: "contact_ada",
  action: "pay"
});
assert.equal(payContactHandoff.ok, true);
assert.equal(payContactHandoff.data.target.routeId, "wallet.send");
assert.equal(payContactHandoff.data.target.referenceDomain, "wallet_recipient");
assert.equal(payContactHandoff.data.reference, "wallet_receiver_ref_9d20…11f7");
assert.equal(payContactHandoff.data.constraints.revalidationRequired, true);
const validatedContactPay = contactsWalletGateway.revalidateExternalReviewHandoff({
  walletId: "everyday",
  handoff: payContactHandoff.data
});
assert.equal(validatedContactPay.ok, true);
assert.equal(validatedContactPay.data.schemaVersion, "wallet-external-review-validation-v1");
assert.equal(validatedContactPay.data.draft, null);
const broadenedContactPay = structuredClone(payContactHandoff.data);
broadenedContactPay.target.referenceDomain = "contact_identity";
assert.equal(
  contactsWalletGateway.revalidateExternalReviewHandoff({
    walletId: "everyday",
    handoff: broadenedContactPay
  }).error.code,
  "external_review_rejected"
);
assert.equal(
  contactsGateway.prepareAction({ contactId: "contact_old_service", action: "pay" }).error.code,
  "contact_revalidation_required"
);
const removedContact = contactsGateway.removeContact({ contactId: addedContact.data.contact.id });
assert.equal(removedContact.ok, true);
assert.equal(removedContact.data.protocolRevocation, false);
assert.equal(removedContact.data.historyErasure, false);
assert.equal(contactsState.contacts.length, 6);
assert.equal(JSON.stringify(contactsState.wallets), contactsWalletSnapshot);

const defaults = demo.resolveInitialNavigation("?view=unknown&wallet=everything&settings=invalid");
assert.equal(defaults.view, "wallet");
assert.equal(defaults.walletSection, "assets");
assert.equal(defaults.settingsSection, "general");
const allowed = demo.resolveInitialNavigation("?view=wallet-send&wallet=permissions&walletSettings=advanced&settings=onionnet&onionTab=queues");
assert.equal(allowed.view, "wallet-send");
assert.equal(allowed.walletSection, "permissions");
assert.equal(allowed.walletSettingsSection, "advanced");
assert.equal(allowed.settingsSection, "onionnet");
assert.equal(allowed.onionnetTelemetryTab, "queues");
assert.equal(demo.canonicalRouteFromLegacyNavigation(allowed), "wallet.send");
assert.equal(
  demo.canonicalRouteFromLegacyNavigation(demo.resolveInitialNavigation("?view=telemetry&telemetry=reticulum&reticulumTab=links")),
  "telemetry.reticulum.links"
);
assert.equal(
  demo.canonicalRouteFromLegacyNavigation(demo.resolveInitialNavigation("?view=telemetry&telemetry=aggregators&aggregatorsTab=recovery")),
  "telemetry.aggregators.recovery"
);
assert.equal(
  demo.canonicalRouteFromLegacyNavigation(demo.resolveInitialNavigation("?view=telemetry&telemetry=explorer&explorerTab=evidence")),
  "telemetry.explorer.evidence"
);
assert.equal(
  demo.canonicalRouteFromLegacyNavigation(demo.resolveInitialNavigation("?view=staking")),
  "wallet.staking.stake"
);

const firstClone = demo.createInitialWallets();
const secondClone = demo.createInitialWallets();
firstClone[0].name = "Changed only here";
firstClone[0].activities.length = 0;
assert.equal(secondClone[0].name, "Everyday");
assert.ok(secondClone[0].activities.length > 0);
assert.equal(demo.INITIAL_WALLET_FIXTURES[0].name, "Everyday");
assert.ok(demo.INITIAL_WALLET_FIXTURES.every(({ chainId }) => chainId === "mainnet"));
const deterministicProfile = demo.createWalletProfile(
  [...secondClone, { id: "wallet-4" }],
  "Field wallet",
  "testnet-2"
);
assert.equal(deterministicProfile.id, "wallet-5");
assert.equal(deterministicProfile.address, "ZxN5q7…2305Pt");
assert.equal(deterministicProfile.chainId, "testnet-2");
assert.equal(demo.createEmptyWallet().summary.scan, "Unavailable");
const friendlyAssetKeys = Object.keys(demo.ASSET_ICON_LUT);
assert.deepEqual(Array.from(demo.DEFAULT_FRIENDLY_ASSET_KEYS), friendlyAssetKeys);
assert.equal(new Set(friendlyAssetKeys).size, 16);
assert.equal(demo.ASSET_CATALOG.length, 16);
for (const iconPath of Object.values(demo.ASSET_ICON_LUT)) {
  const iconInfo = await stat(resolve(demoRoot, iconPath));
  assert.ok(iconInfo.size > 0, `${iconPath} must exist and be non-empty`);
}
for (const wallet of demo.INITIAL_WALLET_FIXTURES) {
  assert.equal(wallet.assetKeys.length, 16);
  assert.ok(friendlyAssetKeys.every((key) => wallet.assetKeys.includes(key)));
}
assert.deepEqual(Array.from(deterministicProfile.assetKeys), friendlyAssetKeys);
assert.deepEqual(Array.from(demo.createEmptyWallet().assetKeys), friendlyAssetKeys);

const state = demo.createInitialState({ search: "?view=activity" });
assert.equal(state.view, "activity");
assert.equal(state.wallets.length, 3);
assert.equal(demo.activeWallet(state).id, "everyday");
const preferences = demo.ensureWalletPreferences(state);
assert.equal(preferences.defaultFee, "0.001");
assert.equal(preferences.lockAfterMinutes, "15");

const gateway = demo.createMockWalletGateway(state);
assert.equal(gateway.contractVersion, demo.PORT_CONTRACT.version);
assert.equal(typeof gateway.submitPayment, "function");
assert.equal(typeof gateway.reconcileOperation, "function");
assert.equal(gateway.createProfile({ name: "x" }).error.code, "validation");
assert.equal(gateway.createProfile({ name: "Valid wallet", chainId: "unknown" }).error.code, "validation");
assert.equal(gateway.removeProfiles({ walletIds: [] }).error.code, "validation");
assert.equal(gateway.removeProfiles({ walletIds: ["missing"] }).error.code, "validation");
assert.equal(gateway.renameWallet({ walletId: "missing", name: "Valid name" }).error.code, "validation");
assert.equal(gateway.renameWallet({ walletId: "everyday", name: "x" }).error.code, "validation");
assert.equal(gateway.changePassword({ walletId: "missing", currentPassword: "old-value", newPassword: "new-value" }).error.code, "validation");
assert.equal(gateway.changePassword({ walletId: "everyday", currentPassword: "same-value", newPassword: "same-value" }).error.code, "validation");
const created = gateway.createProfile({ name: "Field wallet", chainId: "devnet-2", scan: "Scanning" });
assert.equal(created.ok, true);
assert.equal(state.wallets.at(-1).name, "Field wallet");
assert.equal(state.wallets.at(-1).chainId, "devnet-2");
const renamed = gateway.renameWallet({ walletId: created.data.wallet.id, name: "Field savings" });
assert.equal(renamed.ok, true);
assert.equal(state.wallets.at(-1).initials, "F");

const operationState = demo.createInitialState({
  search: "?operationScenario=timeout_unknown_outcome"
});
const operationGateway = demo.createMockWalletGateway(operationState);
const initialActivityCount = operationState.wallets[0].activities.length;
const timedOutOperation = operationGateway.submitPayment({
  walletId: "everyday",
  family: "asset",
  itemKey: "z00z",
  amount: "12.50",
  recipient: "z00z1native-boundary",
  idempotencyKey: "payment-intent-1",
  scenario: operationState.demoOperationScenario
});
assert.equal(timedOutOperation.ok, false);
assert.equal(timedOutOperation.error.code, "timeout_unknown_outcome");
assert.match(timedOutOperation.error.operationId, /^payment-everyday-/);
const repeatedOperation = operationGateway.submitPayment({
  walletId: "everyday",
  family: "asset",
  itemKey: "z00z",
  amount: "12.50",
  recipient: "z00z1native-boundary",
  idempotencyKey: "payment-intent-1",
  scenario: operationState.demoOperationScenario
});
assert.equal(repeatedOperation.ok, true);
assert.equal(repeatedOperation.data.operationId, timedOutOperation.error.operationId);
assert.equal(operationState.wallets[0].activities.length, initialActivityCount + 1);
const reconciledOperation = operationGateway.reconcileOperation({
  operationId: timedOutOperation.error.operationId
});
assert.equal(reconciledOperation.ok, true);
assert.equal(reconciledOperation.data.status, "pending_confirmation");
assert.equal(operationState.wallets[0].activities.length, initialActivityCount + 1);

const currentSecret = "current-password-value";
const newSecret = "new-password-value";
const changed = gateway.changePassword({
  walletId: created.data.wallet.id,
  currentPassword: currentSecret,
  newPassword: newSecret
});
assert.equal(changed.ok, true);
const serializedState = JSON.stringify(state);
assert.equal(serializedState.includes(currentSecret), false);
assert.equal(serializedState.includes(newSecret), false);

demo.ensureWalletPreferences(state, state.wallets.find(({ id }) => id === "savings"));
assert.ok(state.walletPreferences.savings);
const preservedSelection = gateway.removeProfiles({
  walletIds: ["savings"],
  selectedWalletId: "everyday"
});
assert.equal(preservedSelection.ok, true);
assert.equal(preservedSelection.data.selectedWalletId, "everyday");
assert.equal(state.walletPreferences.savings, undefined);

const allIds = state.wallets.map(({ id }) => id);
const removed = gateway.removeProfiles({ walletIds: allIds, selectedWalletId: state.selectedWalletId });
assert.equal(removed.ok, true);
assert.equal(state.wallets.length, 0);
assert.equal(removed.data.selectedWalletId, null);

assert.equal(locales.length, 10);
assert.deepEqual(
  Array.from(locales, ({ id }) => id),
  ["en", "ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"]
);
assert.equal(new Set(locales.map(({ catalogue }) => catalogue)).size, locales.length);

assert.equal(new Set(demo.ICON_NAMES).size, demo.ICON_NAMES.length);
for (const family of Object.values(demo.OBJECT_TYPE_ICON_LUT)) {
  for (const definition of Object.values(family)) {
    if (definition.iconName) {
      assert.ok(demo.ICON_NAMES.includes(definition.iconName));
      continue;
    }
    assert.equal(definition.mode, "image");
    const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
    assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
  }
}
for (const definition of Object.values(demo.OBJECT_FAMILY_ICON_LUT)) {
  assert.ok(["image", "mask"].includes(definition.mode));
  const iconInfo = await stat(resolve(demoRoot, definition.iconSrc));
  assert.ok(iconInfo.size > 0, `${definition.iconSrc} must exist and be non-empty`);
}
assert.equal(Object.keys(demo.VOUCHER_ICON_LUT).length, 8);
assert.equal(Object.keys(demo.PERMISSION_ICON_LUT).length, 8);
for (const voucher of demo.INITIAL_WALLET_FIXTURES[0].vouchers) assert.ok(demo.VOUCHER_ICON_LUT[voucher.kind]);
for (const permission of demo.INITIAL_WALLET_FIXTURES[0].permissions) assert.ok(demo.PERMISSION_ICON_LUT[permission.kind]);

const navigationValidation = demo.assertNavigationModel();
assert.equal(navigationValidation.valid, true);
assert.equal(demo.NAVIGATION_NODES.length > demo.PORT_CONTRACT.routes.length, true);
assert.deepEqual(
  Array.from(demo.NAVIGATION_NODES)
    .filter(({ target }) => ["route", "workspace"].includes(target.kind))
    .map(({ target }) => target.routeId)
    .sort(),
  Array.from(demo.PORT_CONTRACT.routes).sort()
);
assert.equal(
  new Set(demo.NAVIGATION_NODES.filter(({ target }) => ["route", "workspace"].includes(target.kind)).map(({ helpTopicId }) => helpTopicId)).size,
  demo.PORT_CONTRACT.routes.length
);
assert.ok(demo.NAVIGATION_NODES.filter(({ target }) => target.kind === "branch").every(({ parentId }) => parentId === null));
assert.equal(demo.navigationNode("wallet.overview"), null);
assert.equal(demo.navigationNode("wallet.assets-rights").target.kind, "workspace");
assert.equal(demo.navigationNode("wallet.assets-rights").target.routeId, "wallet.assets");
assert.equal(demo.navigationNode("wallet.vouchers").parentId, "wallet.assets-rights");
assert.equal(demo.navigationNode("wallet.permissions").parentId, "wallet.assets-rights");
assert.equal(demo.navigationNode("wallet.quarantine").parentId, "wallet.assets-rights");
assert.equal(demo.navigationNode("wallet.settings").target.kind, "workspace");
assert.equal(demo.navigationNode("wallet.staking").target.kind, "workspace");
assert.equal(demo.navigationNode("wallet.staking").target.routeId, "wallet.staking.stake");
assert.equal(demo.navigationNode("wallet.staking.unstake").parentId, "wallet.staking");
assert.deepEqual(
  Array.from(demo.workspaceLocalDestinations("wallet.staking"), ({ routeId }) => routeId),
  ["wallet.staking.stake", "wallet.staking.unstake"]
);
const workspaceNodes = demo.NAVIGATION_NODES.filter(({ target }) => target.kind === "workspace");
assert.equal(workspaceNodes.length, 8);
for (const workspace of workspaceNodes) {
  assert.equal(demo.navigationNode(workspace.parentId).target.kind, "branch");
  const localDestinations = demo.workspaceLocalDestinations(workspace.id);
  assert.ok(Object.isFrozen(localDestinations));
  assert.equal(localDestinations.length, demo.navigationChildren(workspace.id).length + 1);
  assert.equal(localDestinations[0].routeId, workspace.target.routeId);
  assert.equal(localDestinations[0].labelKey, workspace.target.defaultLabelKey);
  assert.equal(localDestinations[0].iconId, workspace.target.defaultIconId);
  assert.ok(localDestinations.every(Object.isFrozen));
  assert.ok(demo.navigationChildren(workspace.id).every(({ target }) => target.kind === "route"));
}
assert.deepEqual(
  Array.from(demo.workspaceLocalDestinations("wallet.assets-rights")).map(({ routeId }) => routeId),
  ["wallet.assets", "wallet.vouchers", "wallet.permissions"]
);
assert.deepEqual(
  Array.from(demo.workspaceLocalDestinations("wallet.settings")).map(({ routeId }) => routeId),
  ["wallet.settings.general", "wallet.settings.security", "wallet.settings.backup", "wallet.settings.policies", "wallet.settings.advanced"]
);
for (const [workspaceId, defaultRoute, childCount] of [
  ["telemetry.reticulum", "telemetry.reticulum.overview", 7],
  ["telemetry.onionnet", "telemetry.onionnet.overview", 6],
  ["telemetry.aggregators", "telemetry.aggregators.overview", 5],
  ["telemetry.watchers", "telemetry.watchers.overview", 5],
  ["telemetry.explorer", "telemetry.explorer.overview", 4],
]) {
  const workspace = demo.navigationNode(workspaceId);
  assert.equal(workspace.parentId, "telemetry");
  assert.equal(workspace.target.kind, "workspace");
  assert.equal(workspace.target.routeId, defaultRoute);
  const children = demo.navigationChildren(workspaceId);
  assert.equal(children.length, childCount);
  assert.ok(children.every(({ parentId, target }) => parentId === workspaceId && target.kind === "route"));
}
assert.equal(demo.NAVIGATION_NODES.filter(({ target }) => target.kind === "group").length, 0);
assert.deepEqual(
  Array.from(demo.navigationChildren(), ({ id }) => id),
  ["wallet", "telemetry", "dapps", "messenger", "data-storage", "contacts.list", "settings", "help", "about", "logout"]
);
assert.ok(demo.ICON_NAMES.includes("message"));
assert.ok(demo.ICON_NAMES.includes("storage"));
assert.ok(demo.ICON_NAMES.includes("info"));
assert.deepEqual(
  Array.from(demo.navigationChildren("data-storage"), ({ target }) => target.routeId),
  ["data-storage.disk-usage", "data-storage.network-usage"]
);
assert.deepEqual(
  Array.from(demo.navigationChildren("settings"), ({ target }) => target.routeId),
  ["settings.general", "settings.notifications", "settings.appearance"]
);
assert.equal(demo.navigationNode("about").target.routeId, "about");
assert.equal(demo.navigationNode("about").isVisible, true);
assert.ok(Array.from(demo.navigationChildren("wallet"), ({ id }) => id).every(
  (id) => !["wallet.swap", "wallet.exchange"].includes(id)
));
assert.deepEqual(
  Array.from(demo.navigationChildren("dapps"), ({ target }) => target.routeId),
  ["dapps.discover", "dapps.installed", "dapps.connections", "dapps.permissions", "wallet.swap", "wallet.exchange"]
);
assert.equal(context.window.Z00ZHelpRegistry.topic("wallet.swap").pagePath.join("/"), "dapps/swap");
assert.equal(context.window.Z00ZHelpRegistry.topic("wallet.exchange").pagePath.join("/"), "dapps/exchange");
assert.deepEqual(
  Array.from(demo.navigationChildren("messenger"), ({ target }) => target.routeId),
  ["messenger.inbox", "messenger.sent", "messenger.conversations"]
);
assert.equal(context.window.Z00ZHelpRegistry.hasTopic("logout"), false);
assert.equal(context.window.Z00ZHelpRegistry.globalTopic(), "app");
assert.equal(demo.capabilityProfile("messenger").presentationMode, "roadmap_preview");
assert.equal(demo.capabilityProfile("telemetry.watchers").evidenceSource, "fixture");
assert.deepEqual(
  Array.from(demo.PORT_CONTRACT.telemetryResultStates),
  ["loading", "success", "degraded", "unavailable", "empty", "malformed", "error"]
);
const telemetryGateway = demo.createMockTelemetryGateway();
assert.equal(telemetryGateway.contractVersion, demo.PORT_CONTRACT.version);
assert.deepEqual(Array.from(telemetryGateway.scenarioIds), Array.from(demo.PORT_CONTRACT.telemetryResultStates));
const telemetryObservations = Object.fromEntries(
  telemetryGateway.scenarioIds.map((scenario) => [
    scenario,
    telemetryGateway.readObservation({
      capabilityId: "telemetry.aggregators",
      routeId: "telemetry.aggregators.publication",
      scenario,
      generation: 3
    })
  ])
);
for (const [scenario, observation] of Object.entries(telemetryObservations)) {
  assert.equal(observation.status, scenario);
  assert.equal(observation.request.generation, 3);
  assert.equal(observation.request.requestKey, "telemetry:telemetry.aggregators.publication");
  assert.equal(observation.capability.maturity, "live");
  assert.equal(observation.capability.presentationMode, "product");
  assert.ok(demo.PORT_CONTRACT.availability.includes(observation.capability.availability));
  assert.ok(demo.PORT_CONTRACT.evidenceSources.includes(observation.capability.evidenceSource));
  assert.ok(demo.PORT_CONTRACT.freshness.includes(observation.capability.freshness));
  assert.ok(Object.isFrozen(observation));
  assert.ok(Object.isFrozen(observation.capability));
}
assert.equal(telemetryObservations.success.capability.availability, "available");
assert.equal(telemetryObservations.success.capability.evidenceSource, "fixture");
assert.equal(telemetryObservations.success.capability.freshness, "timestamp");
assert.equal(telemetryObservations.success.data.total, 2);
assert.equal(telemetryObservations.degraded.capability.availability, "degraded");
assert.equal(telemetryObservations.degraded.capability.freshness, "stale");
assert.equal(telemetryObservations.unavailable.capability.evidenceSource, "none");
assert.equal(telemetryObservations.empty.data.total, 0);
assert.equal(telemetryObservations.malformed.data, null);
assert.equal(telemetryObservations.error.retryable, true);
const watcherFixture = telemetryGateway.readObservation({
  capabilityId: "telemetry.watchers",
  routeId: "telemetry.watchers.alerts",
  scenario: "success"
});
assert.equal(watcherFixture.capability.maturity, "live");
assert.equal(watcherFixture.capability.presentationMode, "roadmap_preview");
assert.equal(watcherFixture.capability.evidenceSource, "fixture");
const watcherAlerts = telemetryGateway.readWatcherView({
  routeId: "telemetry.watchers.alerts",
  scenario: "success",
  sourceId: "runtime_projection",
  generation: 4,
  filters: { severity: "critical", kind: "MissingBlob" }
});
assert.equal(watcherAlerts.data.total, 1);
assert.equal(watcherAlerts.data.records[0].recordType, "watcher_alert");
assert.equal(watcherAlerts.data.records[0].kind, "MissingBlob");
assert.equal(watcherAlerts.data.records[0].severity, "critical");
assert.equal(watcherAlerts.data.records[0].subject.kind, "published_batch");
assert.equal(watcherAlerts.data.records[0].observedAt, "2026-07-26T12:00:00.000Z");
assert.equal(watcherAlerts.data.records[0].explorerAction.kind, "open_explorer");
assert.equal(watcherAlerts.data.records[0].explorerAction.label, "Open DA evidence in Explorer");
assert.equal(watcherAlerts.data.records[0].explorerAction.publicId, "da_ref_72be91");
assert.equal(watcherAlerts.source.datasetId, "runtime_projection");
assert.equal(watcherAlerts.request.generation, 4);
const watcherDegraded = telemetryGateway.readWatcherView({
  routeId: "telemetry.watchers.providers",
  scenario: "degraded",
  sourceId: "evidence_archive"
});
assert.equal(watcherDegraded.status, "degraded");
assert.equal(watcherDegraded.data.total, 1);
assert.equal(watcherDegraded.source.datasetId, "evidence_archive");
for (const scenario of ["loading", "unavailable", "malformed", "error"]) {
  const failedWatcherView = telemetryGateway.readWatcherView({
    routeId: "telemetry.watchers.overview",
    scenario
  });
  assert.equal(failedWatcherView.status, scenario);
  assert.equal(failedWatcherView.data, null);
}
assert.equal(telemetryGateway.readWatcherView({
  routeId: "telemetry.watchers.evidence",
  scenario: "empty"
}).data.total, 0);
const watcherExport = telemetryGateway.prepareWatcherEvidenceExport({
  alertId: "watcher-alert-002",
  sourceId: "runtime_projection"
});
assert.equal(watcherExport.schemaVersion, "watcher-evidence-export-v1");
assert.equal(watcherExport.alert.kind, "MissingBlob");
assert.equal(watcherExport.evidence.batchId, "batch_a13d9e22");
assert.equal(watcherExport.evidenceSource, "fixture");
assert.ok(Object.isFrozen(watcherExport));
for (const forbiddenValue of ["Everyday", "Savings", "Travel", "receiver", "counterparty", "memo", "seed_phrase", "private_key"]) {
  assert.equal(JSON.stringify(watcherExport).toLowerCase().includes(forbiddenValue.toLowerCase()), false);
}
const explorerFixture = telemetryGateway.readExplorerView({
  routeId: "telemetry.explorer.checkpoints",
  scenario: "success",
  generation: 9
});
assert.equal(explorerFixture.capability.maturity, "target");
assert.equal(explorerFixture.capability.presentationMode, "roadmap_preview");
assert.equal(explorerFixture.capability.evidenceSource, "fixture");
assert.equal(explorerFixture.source.datasetId, "public_evidence_fixture");
assert.equal(explorerFixture.source.authority, "storage public proof surface");
assert.equal(explorerFixture.request.generation, 9);
assert.equal(explorerFixture.data.total, 3);
assert.equal(explorerFixture.data.records[0].recordType, "checkpoint");
assert.equal(explorerFixture.data.records[0].lifecycleStatus, "finalized");
assert.equal(explorerFixture.data.records[0].publicId, "checkpoint_000184");
const explorerEvidence = telemetryGateway.readExplorerView({
  routeId: "telemetry.explorer.evidence",
  scenario: "success",
  filters: { kind: "proof" }
});
assert.equal(explorerEvidence.data.total, 3);
assert.ok(explorerEvidence.data.records.every(({ recordType }) => recordType === "proof"));
assert.equal(telemetryGateway.readExplorerView({
  routeId: "telemetry.explorer.evidence",
  scenario: "empty"
}).data.total, 0);
assert.equal(telemetryGateway.readExplorerView({
  routeId: "telemetry.explorer.batches",
  scenario: "degraded"
}).data.total, 1);
for (const scenario of ["loading", "unavailable", "malformed", "error"]) {
  const failedExplorerView = telemetryGateway.readExplorerView({
    routeId: "telemetry.explorer.overview",
    scenario
  });
  assert.equal(failedExplorerView.status, scenario);
  assert.equal(failedExplorerView.data, null);
}
for (const [publicId, publicKind] of [
  ["checkpoint_000184", "checkpoint"],
  ["batch_4f91c7a0", "batch"],
  ["publication_6f840184", "publication"],
  ["proof_92840184", "proof"],
  ["da_ref_72be91", "da_reference"],
]) {
  const searchResult = telemetryGateway.searchExplorerPublicId({ query: publicId, generation: 10 });
  assert.equal(searchResult.status, "found");
  assert.equal(searchResult.publicId, publicId);
  assert.equal(searchResult.publicKind, publicKind);
  assert.equal(searchResult.record.publicId, publicId);
  assert.equal(searchResult.request.generation, 10);
  assert.ok(Object.isFrozen(searchResult));
  assert.ok(Object.isFrozen(searchResult.record));
}
for (const [query, expectedStatus] of [
  ["", "malformed"],
  ["wallet_everyday", "private"],
  ["receiver_secret_001", "private"],
  ["checkpoint_18", "malformed"],
  ["tx_deadbeef", "unsupported"],
  ["checkpoint_999999", "unknown"],
  ["checkpoint_000183", "stale"],
]) {
  const rejected = telemetryGateway.searchExplorerPublicId({ query });
  assert.equal(rejected.status, expectedStatus);
  assert.equal(rejected.publicId, null);
  assert.equal(rejected.record, null);
  if (query) assert.equal(JSON.stringify(rejected).includes(query), false);
}
const degradedExplorerSearch = telemetryGateway.searchExplorerPublicId({
  query: "checkpoint_000184",
  scenario: "degraded"
});
assert.equal(degradedExplorerSearch.status, "degraded");
assert.equal(degradedExplorerSearch.record, null);
for (const [publicId, publicKind, routeId] of [
  ["checkpoint_000184", "checkpoint", "telemetry.explorer.checkpoints"],
  ["batch_72bc108f", "batch", "telemetry.explorer.batches"],
  ["da_ref_72be91", "da_reference", "telemetry.explorer.evidence"],
]) {
  const deepLink = telemetryGateway.resolveExplorerDeepLink({ publicId });
  assert.deepEqual(Object.keys(deepLink), ["ok", "publicId", "publicKind", "routeId"]);
  assert.equal(deepLink.ok, true);
  assert.equal(deepLink.publicId, publicId);
  assert.equal(deepLink.publicKind, publicKind);
  assert.equal(deepLink.routeId, routeId);
  assert.ok(Object.isFrozen(deepLink));
}
const rejectedDeepLink = telemetryGateway.resolveExplorerDeepLink({ publicId: "receiver_secret_001" });
assert.equal(rejectedDeepLink.ok, false);
assert.equal(rejectedDeepLink.publicId, null);
assert.equal(rejectedDeepLink.routeId, null);
assert.equal(JSON.stringify(rejectedDeepLink).includes("receiver_secret_001"), false);
for (const forbiddenValue of ["Everyday", "Savings", "Travel", "receiver", "counterparty", "memo", "seed_phrase", "private_key", "inbox", "route_path"]) {
  const explorerPayloads = JSON.stringify([
    demo.EXPLORER_CHECKPOINTS,
    demo.EXPLORER_BATCHES,
    demo.EXPLORER_PUBLIC_EVIDENCE,
    explorerFixture,
    explorerEvidence
  ]).toLowerCase();
  assert.equal(explorerPayloads.includes(forbiddenValue.toLowerCase()), false);
}
const privateWalletCanaries = [...new Set(demo.INITIAL_WALLET_FIXTURES.flatMap((wallet) => [
  wallet.name,
  wallet.address,
  wallet.fullAddress,
  ...wallet.activities.flatMap((activity) => Object.values(activity.titleValues || {}))
]))].filter(Boolean);
const privateRendererCanaries = [
  ...privateWalletCanaries,
  "z00z1telemetry-private-receiver",
  "telemetry-private-memo-canary",
  "/home/vadim/Projects/z00z",
  "/tmp/z00z-private-route",
  "inbox-record-private",
  "messenger.inbox",
  "seed_phrase",
  "private_key",
  "session_token",
  "raw_signed_package",
  "arbitrary_filesystem_path"
];
const telemetryPayloadMatrix = [];
for (const routeId of demo.PORT_CONTRACT.telemetryRoutes) {
  const capabilityId = routeId.split(".").slice(0, 2).join(".");
  for (const scenario of telemetryGateway.scenarioIds) {
    const observation = capabilityId === "telemetry.watchers"
      ? telemetryGateway.readWatcherView({ routeId, scenario })
      : capabilityId === "telemetry.explorer"
        ? telemetryGateway.readExplorerView({ routeId, scenario })
        : telemetryGateway.readObservation({ capabilityId, routeId, scenario });
    telemetryPayloadMatrix.push(observation);
  }
}
for (const alert of demo.WATCHER_ROUTE_RECORDS["telemetry.watchers.alerts"]) {
  telemetryPayloadMatrix.push(telemetryGateway.prepareWatcherEvidenceExport({ alertId: alert.id }));
  telemetryPayloadMatrix.push(telemetryGateway.resolveExplorerDeepLink({
    publicId: alert.explorerAction.publicId
  }));
}
const serializedTelemetryMatrix = JSON.stringify(telemetryPayloadMatrix).toLowerCase();
for (const canary of privateRendererCanaries) {
  assert.equal(
    serializedTelemetryMatrix.includes(canary.toLowerCase()),
    false,
    `Telemetry payload matrix must redact ${canary}`
  );
}
assert.equal(telemetryPayloadMatrix.length, (demo.PORT_CONTRACT.telemetryRoutes.length * 7) + 6);
assert.equal(
  JSON.stringify(telemetryObservations.success),
  JSON.stringify(telemetryGateway.readObservation({
    capabilityId: "telemetry.aggregators",
    routeId: "telemetry.aggregators.publication",
    scenario: "success",
    generation: 3
  }))
);
assert.throws(
  () => telemetryGateway.readObservation({
    capabilityId: "telemetry.aggregators",
    routeId: "telemetry.watchers.overview",
    scenario: "success"
  }),
  /does not belong/
);
assert.throws(
  () => telemetryGateway.readObservation({
    capabilityId: "telemetry.aggregators",
    routeId: "telemetry.aggregators.overview",
    scenario: "invented"
  }),
  /Unknown telemetry scenario/
);
for (const forbiddenValue of ["Everyday", "Savings", "Travel", "receiver", "counterparty", "memo", "seed_phrase", "private_key"]) {
  assert.equal(JSON.stringify(telemetryObservations).toLowerCase().includes(forbiddenValue.toLowerCase()), false);
}
for (const [capabilityId, expected] of Object.entries({
  "wallet.swap": ["live", "unavailable", "fixture", "not_applicable", "product"],
  "wallet.exchange": ["target", "unavailable", "none", "not_applicable", "product"],
  "wallet.staking": ["live", "unavailable", "fixture", "not_applicable", "product"],
})) {
  const capability = demo.capabilityProfile(capabilityId);
  assert.ok(capability, `${capabilityId} capability profile must exist`);
  assert.equal(capability.maturity, expected[0]);
  assert.equal(capability.availability, expected[1]);
  assert.equal(capability.evidenceSource, expected[2]);
  assert.equal(capability.freshness, expected[3]);
  assert.equal(capability.presentationMode, expected[4]);
}

const duplicateNodeValidation = demo.validateNavigationModel({
  nodes: [...demo.NAVIGATION_NODES, demo.NAVIGATION_NODES[0]]
});
assert.equal(duplicateNodeValidation.valid, false);
assert.ok(duplicateNodeValidation.errors.some((error) => error.startsWith("duplicate node ID:")));

const missingIconValidation = demo.validateNavigationModel({
  nodes: demo.NAVIGATION_NODES.map((entry) => entry.id === "messenger" ? { ...entry, iconId: "missing" } : entry)
});
assert.equal(missingIconValidation.valid, false);
assert.ok(missingIconValidation.errors.some((error) => error.startsWith("missing icon:")));

const nestedWorkspaceValidation = demo.validateNavigationModel({
  nodes: demo.NAVIGATION_NODES.map((entry) => (
    entry.id === "wallet.assets-rights" ? { ...entry, parentId: "telemetry.reticulum" } : entry
  ))
});
assert.equal(nestedWorkspaceValidation.valid, false);
assert.ok(nestedWorkspaceValidation.errors.includes("workspace must be a first-level branch leaf: wallet.assets-rights"));

const nonRouteWorkspaceChildValidation = demo.validateNavigationModel({
  nodes: demo.NAVIGATION_NODES.map((entry) => (
    entry.id === "telemetry.reticulum.node"
      ? { ...entry, target: { kind: "action", actionId: "logout" } }
      : entry
  ))
});
assert.equal(nonRouteWorkspaceChildValidation.valid, false);
assert.ok(nonRouteWorkspaceChildValidation.errors.includes("workspace child must be a local route: telemetry.reticulum.node"));

let shell = demo.defaultShellState(demo.resolveInitialNavigation("?view=wallet&wallet=assets"));
assert.equal(shell.activeRoute, "wallet.assets");
assert.deepEqual(Array.from(shell.expandedBranchIds), ["wallet"]);
const openedTelemetry = demo.reduceShellState(shell, { type: "toggle_branch", nodeId: "telemetry" });
assert.deepEqual(Array.from(openedTelemetry.expandedBranchIds), ["telemetry", "wallet"]);
const nestedToggleIgnored = demo.reduceShellState(openedTelemetry, { type: "toggle_branch", nodeId: "wallet.settings" });
assert.deepEqual(Array.from(nestedToggleIgnored.expandedBranchIds), ["telemetry", "wallet"]);
const closedOne = demo.reduceShellState(nestedToggleIgnored, { type: "toggle_branch", nodeId: "telemetry" });
assert.deepEqual(Array.from(closedOne.expandedBranchIds), ["wallet"]);
assert.equal(closedOne.activeRoute, "wallet.assets");

const leafSelection = demo.reduceShellState(
  { ...closedOne, drawerOpen: true },
  { type: "select_leaf", nodeId: "telemetry.reticulum.links" }
);
assert.equal(leafSelection.activeRoute, "telemetry.reticulum.links");
assert.equal(leafSelection.drawerOpen, false);
assert.ok(leafSelection.expandedBranchIds.includes("telemetry"));

const restoredRoute = demo.reduceShellState(leafSelection, {
  type: "restore_route",
  routeId: "wallet.settings.advanced"
});
assert.equal(restoredRoute.activeRoute, "wallet.settings.advanced");
assert.ok(restoredRoute.expandedBranchIds.includes("wallet"));
const switchedWallet = demo.reduceShellState(restoredRoute, { type: "switch_wallet", walletId: "travel" });
assert.equal(switchedWallet.activeWalletId, "travel");
assert.equal(switchedWallet.activeRoute, "wallet.settings.advanced");
const switchedFromTelemetry = demo.reduceShellState(leafSelection, { type: "switch_wallet", walletId: "travel" });
assert.equal(switchedFromTelemetry.activeRoute, "wallet.assets");

const requestStarted = demo.reduceShellState(switchedWallet, { type: "begin_request", requestKey: "telemetry.watchers" });
assert.equal(requestStarted.requestGenerations["telemetry.watchers"], 1);
const requestRestarted = demo.reduceShellState(requestStarted, { type: "begin_request", requestKey: "telemetry.watchers" });
assert.equal(requestRestarted.requestGenerations["telemetry.watchers"], 2);
const requestCancelled = demo.reduceShellState(requestRestarted, { type: "cancel_request", requestKey: "telemetry.watchers" });
assert.deepEqual(Array.from(requestCancelled.cancelledRequestKeys), ["telemetry.watchers"]);
const lockedShell = demo.reduceShellState({ ...requestCancelled, drawerOpen: true }, { type: "lock" });
assert.equal(lockedShell.locked, true);
assert.equal(lockedShell.drawerOpen, false);
const loggedOutShell = demo.reduceShellState(lockedShell, { type: "logout" });
assert.equal(loggedOutShell.activeRoute, "wallet.assets");
assert.equal(loggedOutShell.activeWalletId, null);
assert.equal(loggedOutShell.locked, true);
assert.deepEqual(Object.keys(loggedOutShell.requestGenerations), []);

const scopedRequestKey = demo.createRequestKey({
  domain: "telemetry",
  routeId: "telemetry.watchers.alerts",
  walletId: "travel",
  scope: "severity-all"
});
const scopedShell = demo.reduceShellState({
  ...switchedWallet,
  activeRoute: "telemetry.watchers.alerts"
}, {
  type: "begin_request",
  requestKey: scopedRequestKey
});
const scopedResponse = {
  requestKey: scopedRequestKey,
  generation: 1,
  routeId: "telemetry.watchers.alerts",
  walletId: "travel"
};
assert.equal(demo.requestResultIsCurrent(scopedShell, scopedResponse), true);
assert.equal(demo.requestResultIsCurrent(
  { ...scopedShell, activeRoute: "telemetry.explorer.search" },
  scopedResponse
), false);
assert.equal(demo.requestResultIsCurrent(
  { ...scopedShell, activeWalletId: "everyday" },
  scopedResponse
), false);
assert.equal(demo.requestResultIsCurrent(scopedShell, {
  ...scopedResponse,
  generation: 0
}), false);
assert.equal(demo.requestResultIsCurrent(
  demo.reduceShellState(scopedShell, { type: "cancel_request", requestKey: scopedRequestKey }),
  scopedResponse
), false);
assert.equal(demo.requestResultIsCurrent(
  demo.reduceShellState(scopedShell, { type: "lock" }),
  scopedResponse
), false);
assert.throws(
  () => demo.createRequestKey({
    domain: "telemetry",
    routeId: "telemetry.watchers.alerts",
    walletId: "../wallet",
    scope: "all"
  }),
  /bounded identifiers/
);

const gatewayCalls = [];
const gatewaySentinel = Object.freeze({ call: () => gatewayCalls.push("called") });
void gatewaySentinel;
demo.reduceShellState(shell, { type: "toggle_branch", nodeId: "telemetry" });
assert.deepEqual(gatewayCalls, []);

console.log("Production-port contract tests passed.");
