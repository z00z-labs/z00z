"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.DAPP_CATALOG || !demo.DAPP_CONNECTION_FIXTURES) {
    throw new Error("Z00Z contracts and the local dApp catalogue must load before the mock dApp gateway.");
  }

  const REVIEWED_AT = "2026-07-26T12:00:00.000Z";
  const DECISION_IDS = Object.freeze(["accepted", "rejected"]);
  const WALLET_REVIEW_TARGETS = deepFreeze({
    prepare_offline_payment: {
      routeId: "wallet.send",
      flow: "send",
      family: "asset",
      itemKey: "z00z"
    },
    propose_agent_budget: {
      routeId: "wallet.send",
      flow: "send",
      family: "asset",
      itemKey: "z00z"
    },
    issue_service_credit: {
      routeId: "wallet.permissions",
      flow: "permission_inspection",
      family: "permission",
      itemKey: null
    }
  });

  function deepFreeze(value) {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  }

  const ok = (data) => deepFreeze({ ok: true, data });
  const fail = (code, message) => deepFreeze({
    ok: false,
    error: { code, message }
  });

  function hasValue(value) {
    return value?.amount !== "None" && value?.unit !== "Not applicable";
  }

  function reviewFromConnection(connection) {
    const descriptor = demo.dappDescriptor(connection.descriptorId);
    const valuePresent = hasValue(connection.value);
    const feePresent = hasValue(connection.fee);
    return deepFreeze({
      schemaVersion: "dapp-permission-review-v1",
      reviewId: `review_${connection.id}`,
      connectionId: connection.id,
      descriptorId: descriptor.id,
      appIdentity: {
        id: descriptor.id,
        label: descriptor.label,
        publisher: descriptor.publisher.label,
        provenance: descriptor.publisher.provenance,
        verified: false
      },
      intent: {
        type: descriptor.intentType,
        humanReadable: connection.humanIntent,
        action: connection.action
      },
      permission: {
        objectFamily: connection.objectFamily,
        exactScope: connection.exactScope,
        uses: connection.uses,
        expiry: connection.expiry,
        delegation: connection.delegation
      },
      value: {
        present: valuePresent,
        display: `${connection.value.amount} ${connection.value.unit}`
      },
      fee: {
        present: feePresent,
        display: `${connection.fee.amount} ${connection.fee.unit}`,
        path: connection.fee.path
      },
      disclosures: connection.disclosures,
      revoke: {
        behavior: connection.revokeBehavior
      },
      reauth: {
        required: valuePresent || feePresent,
        authority: "wallet_review_only",
        behavior: connection.reauth
      },
      boundary: {
        genericSigning: false,
        arbitraryPayload: false,
        walletMutation: false,
        remoteExecution: false
      }
    });
  }

  const DAPP_PERMISSION_REVIEWS = deepFreeze(
    demo.DAPP_CONNECTION_FIXTURES.map(reviewFromConnection)
  );
  const REVIEW_BY_ID = new Map(DAPP_PERMISSION_REVIEWS.map((review) => [review.reviewId, review]));
  const REVIEW_BY_CONNECTION_ID = new Map(DAPP_PERMISSION_REVIEWS.map((review) => [review.connectionId, review]));

  function proposalFromReview(review) {
    return deepFreeze({
      schemaVersion: "dapp-intent-proposal-v1",
      connectionId: review.connectionId,
      descriptorId: review.descriptorId,
      intent: {
        type: review.intent.type,
        action: review.intent.action
      },
      permission: {
        objectFamily: review.permission.objectFamily,
        exactScope: review.permission.exactScope,
        uses: review.permission.uses,
        expiry: review.permission.expiry,
        delegation: review.permission.delegation
      },
      value: review.value,
      fee: review.fee,
      boundary: {
        genericSigning: false,
        arbitraryPayload: false,
        remoteResourceLoading: false
      }
    });
  }

  function heldAuthorityFromReview(review) {
    return deepFreeze({
      schemaVersion: "dapp-held-authority-v1",
      connectionId: review.connectionId,
      permission: {
        objectFamily: review.permission.objectFamily,
        exactScope: review.permission.exactScope,
        uses: review.permission.uses,
        expiry: review.permission.expiry,
        delegation: review.permission.delegation
      }
    });
  }

  const DAPP_INTENT_PROPOSALS = deepFreeze(DAPP_PERMISSION_REVIEWS.map(proposalFromReview));
  const DAPP_HELD_AUTHORITIES = deepFreeze(DAPP_PERMISSION_REVIEWS.map(heldAuthorityFromReview));
  const PROPOSAL_BY_CONNECTION_ID = new Map(DAPP_INTENT_PROPOSALS.map((proposal) => [proposal.connectionId, proposal]));
  const HELD_AUTHORITY_BY_CONNECTION_ID = new Map(DAPP_HELD_AUTHORITIES.map((authority) => [authority.connectionId, authority]));

  function recursivelyMatches(value, predicate, key = "") {
    if (predicate({ key, value })) return true;
    if (!value || typeof value !== "object") return false;
    return Object.entries(value).some(([childKey, childValue]) => (
      recursivelyMatches(childValue, predicate, childKey)
    ));
  }

  function exactKeys(value, keys) {
    return value
      && typeof value === "object"
      && !Array.isArray(value)
      && JSON.stringify(Object.keys(value).sort()) === JSON.stringify([...keys].sort());
  }

  function sameRecord(left, right) {
    return JSON.stringify(left) === JSON.stringify(right);
  }

  function validateIntentProposal({ proposal, heldAuthority } = {}) {
    if (!proposal || typeof proposal !== "object" || Array.isArray(proposal)) {
      return fail("invalid_proposal", "A typed local intent proposal is required.");
    }
    if (recursivelyMatches(proposal, ({ key, value }) => (
      (typeof value === "string" && /(?:https?:\/\/|wss?:\/\/|file:\/\/|data:)/i.test(value))
      || (/(?:url|uri|href)$/i.test(key) && Boolean(value))
    ))) {
      return fail("arbitrary_url_forbidden", "Arbitrary URLs and URI-bearing callbacks are forbidden.");
    }
    if (proposal.boundary?.genericSigning === true
      || proposal.boundary?.arbitraryPayload === true
      || recursivelyMatches(proposal, ({ key, value }) => (
        /(?:payloadToSign|messageToSign|signatureRequest|rawPayload)/i.test(key)
        && value !== false
        && value !== null
        && value !== ""
      ))) {
      return fail("generic_signing_forbidden", "Generic signing and arbitrary payload authorization are forbidden.");
    }
    if (proposal.boundary?.remoteResourceLoading === true
      || recursivelyMatches(proposal, ({ key, value }) => (
        /(?:remoteResource|resourceUrl|iframe|bundle|executable|sourceCode)/i.test(key)
        && value !== false
        && value !== null
        && value !== ""
      ))) {
      return fail("remote_resource_forbidden", "Remote resources and executable application material are forbidden.");
    }
    if (!demo.DAPP_INTENT_TYPES.includes(proposal.intent?.type)) {
      return fail("unknown_intent_type", "The proposal intent type is not allowlisted.");
    }

    const review = REVIEW_BY_CONNECTION_ID.get(String(proposal.connectionId || ""));
    const descriptor = demo.dappDescriptor(String(proposal.descriptorId || ""));
    const canonicalProposal = PROPOSAL_BY_CONNECTION_ID.get(String(proposal.connectionId || ""));
    const canonicalHeldAuthority = HELD_AUTHORITY_BY_CONNECTION_ID.get(String(proposal.connectionId || ""));
    if (!review || !descriptor || review.descriptorId !== descriptor.id) {
      return fail("invalid_proposal", "The proposal does not bind to a known local connection and descriptor.");
    }
    if (proposal.intent.type !== descriptor.intentType || proposal.intent.action !== review.intent.action) {
      return fail("intent_mismatch", "The proposal intent does not match the reviewed descriptor and action.");
    }
    if (!exactKeys(proposal, ["schemaVersion", "connectionId", "descriptorId", "intent", "permission", "value", "fee", "boundary"])
      || proposal.schemaVersion !== "dapp-intent-proposal-v1"
      || !exactKeys(proposal.intent, ["type", "action"])
      || !exactKeys(proposal.permission, ["objectFamily", "exactScope", "uses", "expiry", "delegation"])
      || !exactKeys(proposal.value, ["present", "display"])
      || !exactKeys(proposal.fee, ["present", "display", "path"])
      || !exactKeys(proposal.boundary, ["genericSigning", "arbitraryPayload", "remoteResourceLoading"])
      || proposal.boundary.genericSigning !== false
      || proposal.boundary.arbitraryPayload !== false
      || proposal.boundary.remoteResourceLoading !== false) {
      return fail("invalid_proposal", "The proposal must use the exact bounded intent schema.");
    }
    if (!sameRecord(proposal.value, review.value)) {
      return fail("hidden_value_forbidden", "Value presence and display must exactly match the reviewed intent.");
    }
    if (!sameRecord(proposal.fee, review.fee)) {
      return fail("hidden_fee_forbidden", "Fee presence, display, and path must exactly match the reviewed intent.");
    }
    if (!heldAuthority
      || heldAuthority.schemaVersion !== "dapp-held-authority-v1"
      || heldAuthority.connectionId !== review.connectionId
      || !exactKeys(heldAuthority, ["schemaVersion", "connectionId", "permission"])
      || !exactKeys(heldAuthority.permission, ["objectFamily", "exactScope", "uses", "expiry", "delegation"])) {
      return fail("invalid_held_authority", "A typed Wallet-owned held-authority snapshot is required.");
    }
    if (!sameRecord(heldAuthority, canonicalHeldAuthority)) {
      return fail("invalid_held_authority", "The held-authority snapshot does not match the reviewed local fixture.");
    }
    if (!sameRecord(proposal.permission, heldAuthority.permission)) {
      return fail("permission_exceeds_held_authority", "The proposal cannot broaden object family, scope, uses, expiry, or delegation.");
    }

    return ok({
      schemaVersion: "dapp-intent-validation-v1",
      proposal: canonicalProposal,
      heldAuthority: canonicalHeldAuthority,
      reviewId: review.reviewId,
      result: "accepted_for_wallet_review"
    });
  }

  function createMockDappGateway() {
    function readPermissionReview({ connectionId } = {}) {
      const review = REVIEW_BY_CONNECTION_ID.get(String(connectionId || ""));
      return review
        ? ok(review)
        : fail("unknown_connection", "The requested local connection is unavailable.");
    }

    function decidePermissionReview({
      reviewId,
      decision,
      scopeConfirmed = false,
      reauthAcknowledged = false
    } = {}) {
      const review = REVIEW_BY_ID.get(String(reviewId || ""));
      if (!review) return fail("unknown_review", "The requested local permission review is unavailable.");
      if (!DECISION_IDS.includes(decision)) {
        return fail("unknown_decision", "Choose Accept or Reject for this bounded permission review.");
      }
      if (decision === "accepted" && !scopeConfirmed) {
        return fail("scope_confirmation_required", "Confirm the exact displayed scope before accepting.");
      }
      if (decision === "accepted" && review.reauth.required && !reauthAcknowledged) {
        return fail("reauth_acknowledgement_required", "Acknowledge that Wallet re-auth is required before the value or fee path.");
      }

      return ok({
        schemaVersion: "dapp-permission-decision-v1",
        decisionId: `decision_${review.connectionId}_${decision}`,
        reviewId: review.reviewId,
        connectionId: review.connectionId,
        descriptorId: review.descriptorId,
        decision,
        decidedAt: REVIEWED_AT,
        intentReference: {
          type: review.intent.type,
          descriptorId: review.descriptorId,
          reviewId: review.reviewId
        },
        walletReviewRequired: decision === "accepted" && (review.value.present || review.fee.present),
        walletMutation: null
      });
    }

    function readIntentProposal({ connectionId } = {}) {
      const proposal = PROPOSAL_BY_CONNECTION_ID.get(String(connectionId || ""));
      return proposal
        ? ok(proposal)
        : fail("unknown_connection", "The requested local intent proposal is unavailable.");
    }

    function readHeldAuthority({ connectionId } = {}) {
      const authority = HELD_AUTHORITY_BY_CONNECTION_ID.get(String(connectionId || ""));
      return authority
        ? ok(authority)
        : fail("unknown_connection", "The requested held-authority fixture is unavailable.");
    }

    function prepareWalletReview({ decision, proposal, heldAuthority } = {}) {
      const review = REVIEW_BY_ID.get(String(decision?.reviewId || ""));
      if (!review
        || decision?.schemaVersion !== "dapp-permission-decision-v1"
        || decision?.decisionId !== `decision_${review.connectionId}_${decision?.decision}`
        || decision?.connectionId !== review.connectionId
        || decision?.descriptorId !== review.descriptorId
        || decision?.intentReference?.type !== review.intent.type) {
        return fail("invalid_decision", "The accepted dApp decision failed Wallet handoff validation.");
      }
      if (decision.decision !== "accepted") {
        return fail("decision_not_accepted", "Only an accepted bounded intent can continue to Wallet review.");
      }
      const target = WALLET_REVIEW_TARGETS[review.intent.type];
      if (!target) {
        return fail("unsupported_wallet_review", "This bounded intent has no allowlisted Wallet review route.");
      }
      const proposalValidation = validateIntentProposal({
        proposal: proposal || PROPOSAL_BY_CONNECTION_ID.get(review.connectionId),
        heldAuthority: heldAuthority || HELD_AUTHORITY_BY_CONNECTION_ID.get(review.connectionId)
      });
      if (!proposalValidation.ok) return proposalValidation;
      const validatedProposal = proposalValidation.data.proposal;
      const exactValue = /^(\d+(?:\.\d+)?)\s+[A-Z0-9-]+$/.exec(validatedProposal.value.display)?.[1] || "";

      return ok({
        schemaVersion: "dapp-wallet-review-handoff-v1",
        handoffId: `handoff_${decision.decisionId}`,
        source: {
          decisionId: decision.decisionId,
          reviewId: review.reviewId,
          descriptorId: review.descriptorId,
          intentType: review.intent.type,
          proposalSchemaVersion: validatedProposal.schemaVersion
        },
        target,
        draft: target.flow === "send"
          ? {
              family: target.family,
              itemKey: target.itemKey,
              amount: exactValue,
              recipient: "",
              memo: ""
            }
          : null,
        constraints: {
          prefillOnly: true,
          recipientRequired: target.flow === "send",
          walletReviewRequired: true,
          walletMutation: false
        }
      });
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,
      reviewSchemaVersion: "dapp-permission-review-v1",
      decisionSchemaVersion: "dapp-permission-decision-v1",
      readPermissionReview,
      readIntentProposal,
      readHeldAuthority,
      decidePermissionReview,
      validateIntentProposal,
      prepareWalletReview
    });
  }

  Object.assign(root.Z00ZDemo, {
    DAPP_PERMISSION_REVIEWS,
    DAPP_INTENT_PROPOSALS,
    DAPP_HELD_AUTHORITIES,
    WALLET_REVIEW_TARGETS,
    validateDappIntentProposal: validateIntentProposal,
    createMockDappGateway
  });
})(typeof window === "undefined" ? globalThis : window);
