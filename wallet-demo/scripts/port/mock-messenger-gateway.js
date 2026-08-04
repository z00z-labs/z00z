"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.MESSENGER_MESSAGES) {
    throw new Error("Z00Z Messenger fixtures must load before the mock Messenger gateway.");
  }

  const NOW = "2026-07-26T12:00:00.000Z";
  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };
  const ok = (data) => deepFreeze({ ok: true, data });
  const fail = (code, message) => deepFreeze({ ok: false, error: { code, message } });
  const REVIEW_BY_MESSAGE_ID = new Map();
  const REVIEW_BY_ID = new Map();

  for (const message of demo.MESSENGER_MESSAGES.filter(({ request }) => Boolean(request))) {
    const review = deepFreeze({
      schemaVersion: "messenger-request-review-v1",
      reviewId: `review_${message.id}`,
      messageId: message.id,
      senderLabel: message.senderLabel,
      subject: message.subject,
      createdAt: message.createdAt,
      expiresAt: message.expiresAt,
      expired: Date.parse(message.expiresAt) <= Date.parse(NOW),
      request: message.request,
      boundary: {
        advisoryOnly: true,
        walletMutation: false,
        settlementMutation: false,
        rawProtocolMaterial: false
      }
    });
    REVIEW_BY_MESSAGE_ID.set(message.id, review);
    REVIEW_BY_ID.set(review.reviewId, review);
  }

  const MESSENGER_REQUEST_REVIEWS = deepFreeze([...REVIEW_BY_ID.values()]);

  function createMockMessengerGateway() {
    function listMessages({ folder = "inbox", deletedIds = [], blockedSenders = [] } = {}) {
      const deleted = new Set(deletedIds);
      const blocked = new Set(blockedSenders);
      const items = demo.MESSENGER_MESSAGES.filter((message) => (
        message.folders.includes(folder)
        && !deleted.has(message.id)
        && !blocked.has(message.senderLabel)
      ));
      return ok({
        schemaVersion: "messenger-list-v1",
        folder,
        items,
        source: "bundled_local_fixture",
        walletMutation: false,
        settlementMutation: false
      });
    }

    function readMessage({ messageId } = {}) {
      const message = demo.messengerMessage(String(messageId || ""));
      return message
        ? ok({ schemaVersion: "messenger-message-detail-v1", message })
        : fail("unknown_message", "The local advisory message is unavailable.");
    }

    function readRequestReview({ messageId } = {}) {
      const review = REVIEW_BY_MESSAGE_ID.get(String(messageId || ""));
      return review
        ? ok(review)
        : fail("not_a_request", "This advisory item has no Wallet-bound request.");
    }

    function decideRequest({ reviewId, decision } = {}) {
      const review = REVIEW_BY_ID.get(String(reviewId || ""));
      if (!review) return fail("unknown_review", "The local request review is unavailable.");
      if (!["accepted", "rejected"].includes(decision)) {
        return fail("unknown_decision", "Choose Accept or Reject for the reviewed request.");
      }
      if (review.expired && decision === "accepted") {
        return fail("request_expired", "The advisory request expired and cannot enter Wallet review.");
      }
      return ok({
        schemaVersion: "messenger-request-decision-v1",
        decisionId: `decision_${review.messageId}_${decision}`,
        reviewId: review.reviewId,
        messageId: review.messageId,
        requestType: review.request.type,
        decision,
        decidedAt: NOW,
        walletReviewRequired: decision === "accepted",
        walletMutation: null,
        settlementMutation: null
      });
    }

    function prepareWalletReview({ decision } = {}) {
      const review = REVIEW_BY_ID.get(String(decision?.reviewId || ""));
      if (!review
        || decision?.schemaVersion !== "messenger-request-decision-v1"
        || decision?.decisionId !== `decision_${review.messageId}_${decision?.decision}`
        || decision?.messageId !== review.messageId
        || decision?.requestType !== review.request.type) {
        return fail("invalid_decision", "The Messenger decision failed Wallet handoff validation.");
      }
      if (decision.decision !== "accepted") {
        return fail("decision_not_accepted", "Only an accepted advisory request can enter Wallet review.");
      }
      if (review.expired) return fail("request_expired", "The advisory request expired before Wallet review.");

      const target = review.request.walletTarget;
      const amount = /^(\d+(?:\.\d+)?)\s+Z00Z$/.exec(review.request.value)?.[1] || "";
      return ok({
        schemaVersion: "messenger-wallet-review-handoff-v1",
        handoffId: `handoff_${decision.decisionId}`,
        source: {
          decisionId: decision.decisionId,
          reviewId: review.reviewId,
          messageId: review.messageId,
          requestType: review.request.type
        },
        target,
        request: {
          objectFamily: review.request.objectFamily,
          action: review.request.action,
          exactScope: review.request.exactScope,
          value: review.request.value,
          fee: review.request.fee
        },
        draft: target.flow === "send"
          ? {
              family: "asset",
              itemKey: target.itemKey,
              amount,
              recipient: "",
              memo: ""
            }
          : null,
        constraints: {
          prefillOnly: true,
          recipientRequired: target.flow === "send",
          walletRevalidationRequired: true,
          walletMutation: false,
          settlementMutation: false
        }
      });
    }

    function advisoryAction({ messageId, action } = {}) {
      const message = demo.messengerMessage(String(messageId || ""));
      if (!message) return fail("unknown_message", "The local advisory message is unavailable.");
      if (!["opened", "acknowledged", "deleted", "blocked", "reported"].includes(action)) {
        return fail("unknown_action", "The advisory action is not supported.");
      }
      return ok({
        schemaVersion: "messenger-advisory-action-v1",
        actionId: `action_${message.id}_${action}`,
        messageId: message.id,
        senderLabel: message.senderLabel,
        action,
        recordedAt: NOW,
        localPresentationOnly: true,
        walletMutation: false,
        settlementMutation: false
      });
    }

    function readRelayState({ scenario = "available" } = {}) {
      const relay = demo.MESSENGER_RELAY_STATES.find(({ id }) => id === scenario);
      return relay
        ? ok({ schemaVersion: "messenger-relay-state-v1", ...relay })
        : fail("unknown_relay_state", "The requested local relay scenario is unavailable.");
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,
      listMessages,
      readMessage,
      readRequestReview,
      decideRequest,
      prepareWalletReview,
      advisoryAction,
      readRelayState
    });
  }

  Object.assign(root.Z00ZDemo, {
    MESSENGER_REQUEST_REVIEWS,
    createMockMessengerGateway
  });
})(typeof window === "undefined" ? globalThis : window);
