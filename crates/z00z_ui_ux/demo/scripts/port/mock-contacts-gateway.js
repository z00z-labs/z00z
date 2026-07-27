"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.CONTACT_FIXTURES) {
    throw new Error("Z00Z Contacts fixtures must load before the mock Contacts gateway.");
  }

  const NOW = "2026-07-26T12:00:00.000Z";
  const deepFreeze = (value) => {
    if (!value || typeof value !== "object" || Object.isFrozen(value)) return value;
    Object.values(value).forEach(deepFreeze);
    return Object.freeze(value);
  };
  const ok = (data) => deepFreeze({ ok: true, data });
  const fail = (code, message) => deepFreeze({ ok: false, error: { code, message } });
  const frozenClone = (value) => deepFreeze(structuredClone(value));

  function createMockContactsGateway(state) {
    function contactById(contactId) {
      return state.contacts.find(({ id }) => id === contactId) || null;
    }

    function listContacts({ query = "", status = "all", sort = "nickname" } = {}) {
      const normalized = String(query || "").trim().toLocaleLowerCase();
      const normalizedSort = ["nickname", "date"].includes(sort) ? sort : "nickname";
      const items = state.contacts.filter((contact) => {
        if (status !== "all" && contact.status !== status) return false;
        if (!normalized) return true;
        return [
          contact.label,
          contact.safeNote,
          contact.fingerprint,
          ...contact.tags
        ].some((value) => String(value).toLocaleLowerCase().includes(normalized));
      }).sort((left, right) => normalizedSort === "date"
        ? new Date(right.lastLocalUseAt).getTime() - new Date(left.lastLocalUseAt).getTime()
        : left.label.localeCompare(right.label, undefined, { sensitivity: "base" }));
      return ok({
        schemaVersion: "contacts-list-v1",
        query: normalized,
        status,
        sort: normalizedSort,
        items: frozenClone(items),
        source: "wallet_local_records",
        networkRequest: false,
        publicPresence: false,
        inferredTrust: false
      });
    }

    function readContact({ contactId } = {}) {
      const contact = contactById(String(contactId || ""));
      return contact
        ? ok({ schemaVersion: "contact-detail-v1", contact: frozenClone(contact) })
        : fail("unknown_contact", "The local contact record is unavailable.");
    }

    function createImportPreview({ sourceId } = {}) {
      const source = demo.CONTACT_IMPORT_PREVIEWS.find(({ id }) => id === sourceId);
      return source
        ? ok({
            schemaVersion: "contact-import-preview-v1",
            source,
            nativeBoundaryRequired: ["qr_scan", "native_share"].includes(source.id),
            networkRequest: false,
            walletMutation: false
          })
        : fail("unknown_import_source", "Choose a supported local contact source.");
    }

    function addContact({ sourceId, label, safeNote = "" } = {}) {
      const source = demo.CONTACT_IMPORT_PREVIEWS.find(({ id }) => id === sourceId);
      const normalizedLabel = String(label || "").trim();
      const normalizedNote = String(safeNote || "").trim();
      if (!source) return fail("unknown_import_source", "Choose a supported local contact source.");
      if (["qr_scan", "native_share"].includes(source.id)) {
        return fail("native_boundary_unavailable", "This concept requires a native camera/share boundary and loads no browser resource.");
      }
      if (normalizedLabel.length < 2 || normalizedLabel.length > 40) {
        return fail("invalid_label", "Contact label must contain 2–40 characters.");
      }
      if (normalizedNote.length > 80) return fail("invalid_note", "Safe note must be 80 characters or fewer.");
      const ordinal = state.contacts.length + 1;
      const suffix = String(ordinal).padStart(4, "0");
      const contact = {
        id: `contact_local_${suffix}`,
        label: normalizedLabel,
        initials: normalizedLabel.split(/\s+/).slice(0, 2).map((part) => part[0]).join("").toUpperCase(),
        safeNote: normalizedNote,
        tags: ["local"],
        fingerprint: `NEW ${suffix}…LOCAL`,
        source: source.label,
        lastLocalUseAt: NOW,
        chainId: "mainnet",
        compatibility: "Confirmation required",
        expiresAt: "2026-08-26T00:00:00Z",
        status: "needs_confirmation",
        pinned: false,
        contactIdentityKey: `contact_key_local_${suffix}`,
        reticulumDestinationRef: `reticulum_ref_local_${suffix}`,
        walletRecipientRef: `wallet_receiver_ref_local_${suffix}`
      };
      state.contacts.push(contact);
      return ok({
        schemaVersion: "contact-local-mutation-v1",
        action: "added",
        contact: frozenClone(contact),
        localOnly: true,
        networkRequest: false,
        walletMutation: false,
        settlementMutation: false
      });
    }

    function editLabel({ contactId, label, safeNote } = {}) {
      const contact = contactById(String(contactId || ""));
      const normalizedLabel = String(label || "").trim();
      const normalizedNote = String(safeNote || "").trim();
      if (!contact) return fail("unknown_contact", "The local contact record is unavailable.");
      if (normalizedLabel.length < 2 || normalizedLabel.length > 40) {
        return fail("invalid_label", "Contact label must contain 2–40 characters.");
      }
      if (normalizedNote.length > 80) return fail("invalid_note", "Safe note must be 80 characters or fewer.");
      contact.label = normalizedLabel;
      contact.safeNote = normalizedNote;
      contact.initials = normalizedLabel.split(/\s+/).slice(0, 2).map((part) => part[0]).join("").toUpperCase();
      return ok({
        schemaVersion: "contact-local-mutation-v1",
        action: "edited",
        contact: frozenClone(contact),
        localOnly: true,
        networkRequest: false,
        walletMutation: false,
        settlementMutation: false
      });
    }

    function reviewIdentityChange({ contactId, decision } = {}) {
      const contact = contactById(String(contactId || ""));
      if (!contact || contact.status !== "identity_changed") {
        return fail("identity_change_unavailable", "No local identity change is awaiting review.");
      }
      if (!["accepted", "rejected"].includes(decision)) {
        return fail("unknown_decision", "Choose Accept or Reject for the identity change.");
      }
      if (decision === "accepted") {
        contact.status = "known_locally";
        contact.compatibility = "Selected wallet compatible";
      }
      return ok({
        schemaVersion: "contact-identity-review-v1",
        contactId: contact.id,
        decision,
        status: contact.status,
        localOnly: true,
        implicitTrust: false,
        walletMutation: false,
        settlementMutation: false
      });
    }

    function removeContact({ contactId } = {}) {
      const contact = contactById(String(contactId || ""));
      if (!contact) return fail("unknown_contact", "The local contact record is unavailable.");
      state.contacts = state.contacts.filter(({ id }) => id !== contact.id);
      return ok({
        schemaVersion: "contact-local-mutation-v1",
        action: "removed",
        contactId: contact.id,
        localOnly: true,
        protocolRevocation: false,
        historyErasure: false,
        networkRequest: false,
        walletMutation: false,
        settlementMutation: false
      });
    }

    function prepareAction({ contactId, action } = {}) {
      const contact = contactById(String(contactId || ""));
      if (!contact) return fail("unknown_contact", "The local contact record is unavailable.");
      if (!["pay", "request", "message", "export"].includes(action)) {
        return fail("unknown_action", "The contact action is not supported.");
      }
      if (["identity_changed", "expired", "revoked"].includes(contact.status) && action !== "export") {
        return fail("contact_revalidation_required", "Confirm current receiver material before using this local contact.");
      }
      const target = {
        pay: { routeId: "wallet.send", flow: "send", referenceDomain: "wallet_recipient" },
        request: { routeId: "messenger.conversations", flow: "request_compose", referenceDomain: "messenger_contact" },
        message: { routeId: "messenger.conversations", flow: "conversation_compose", referenceDomain: "reticulum_destination" },
        export: { routeId: "contacts.list", flow: "public_export", referenceDomain: "contact_identity" }
      }[action];
      const reference = action === "pay"
        ? contact.walletRecipientRef
        : action === "message"
          ? contact.reticulumDestinationRef
          : contact.contactIdentityKey;
      return ok({
        schemaVersion: "contact-action-handoff-v1",
        handoffId: `handoff_${contact.id}_${action}`,
        contactId: contact.id,
        label: contact.label,
        action,
        target,
        reference,
        constraints: {
          revalidationRequired: true,
          prefillOnly: true,
          networkRequest: false,
          walletMutation: false,
          settlementMutation: false
        }
      });
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,
      listContacts,
      readContact,
      createImportPreview,
      addContact,
      editLabel,
      reviewIdentityChange,
      removeContact,
      prepareAction
    });
  }

  Object.assign(root.Z00ZDemo, { createMockContactsGateway });
})(typeof window === "undefined" ? globalThis : window);
