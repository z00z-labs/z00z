"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.createWalletProfile) {
    throw new Error("Z00Z demo contracts and fixtures must load before the mock gateway.");
  }

  const ok = (data = {}) => Object.freeze({ ok: true, data });
  const fail = (code, message, details = {}) => Object.freeze({
    ok: false,
    error: Object.freeze({ code, message, ...details })
  });

  function createMockWalletGateway(state) {
    const operations = new Map();
    const operationIdByIdempotencyKey = new Map();
    const pendingAssetImports = new Map();
    let operationSequence = 0;
    let assetImportSequence = 0;

    const assetPackageMaxBytes = 64 * 1024;
    const assetPackageFields = new Set([
      "definition",
      "serial_id",
      "amount",
      "commitment",
      "range_proof",
      "nonce",
      "lock_height",
      "is_burned",
      "is_frozen",
      "is_slashed",
      "owner_pub",
      "owner_signature",
      "r_pub",
      "owner_tag",
      "enc_pack",
      "tag16",
      "leaf_ad_id"
    ]);
    const assetDefinitionFields = new Set([
      "id",
      "class",
      "name",
      "symbol",
      "decimals",
      "serials",
      "nominal",
      "domain_name",
      "version",
      "crypto_version",
      "policy_flags",
      "metadata"
    ]);
    const requiredAssetPackageFields = [
      "definition",
      "serial_id",
      "amount",
      "commitment",
      "nonce",
      "is_burned"
    ];
    const requiredAssetDefinitionFields = [...assetDefinitionFields].filter((field) => field !== "metadata");
    const isRecord = (value) => Boolean(value) && typeof value === "object" && !Array.isArray(value);
    const isOptionalString = (value) => value === undefined || value === null || typeof value === "string";
    const isOptionalInteger = (value) => value === undefined || value === null
      || (Number.isInteger(value) && value >= 0);
    const isHex = (value, bytes) => typeof value === "string"
      && value.length === bytes * 2
      && /^[0-9a-f]+$/i.test(value);
    const shortIdentifier = (value) => {
      const text = String(value ?? "");
      return text.length > 22 ? `${text.slice(0, 12)}…${text.slice(-8)}` : text;
    };
    const bech32Alphabet = "qpzry9x8gf2tvdw0s3jn54khce6mua7l";

    function stablePublicHex(seed, byteLength) {
      let stateValue = 0x811c9dc5;
      for (const character of String(seed)) {
        stateValue = Math.imul(stateValue ^ character.charCodeAt(0), 0x01000193) >>> 0;
      }
      return Array.from({ length: byteLength }, (_, index) => {
        stateValue ^= stateValue << 13;
        stateValue ^= stateValue >>> 17;
        stateValue ^= stateValue << 5;
        stateValue = (stateValue + Math.imul(index + 1, 0x9e3779b1)) >>> 0;
        return (stateValue & 0xff).toString(16).padStart(2, "0");
      }).join("");
    }

    function bech32Polymod(values) {
      const generators = [0x3b6a57b2, 0x26508e6d, 0x1ea119fa, 0x3d4233dd, 0x2a1462b3];
      let checksum = 1;
      for (const value of values) {
        const top = checksum >>> 25;
        checksum = ((checksum & 0x1ffffff) << 5) ^ value;
        generators.forEach((generator, index) => {
          if ((top >>> index) & 1) checksum ^= generator;
        });
      }
      return checksum >>> 0;
    }

    function bech32mOwnerHandle(ownerHandle) {
      const hrp = "z00z";
      const bytes = ownerHandle.match(/.{2}/g).map((part) => Number.parseInt(part, 16));
      const words = [];
      let accumulator = 0;
      let bitCount = 0;
      for (const byte of bytes) {
        accumulator = (accumulator << 8) | byte;
        bitCount += 8;
        while (bitCount >= 5) {
          bitCount -= 5;
          words.push((accumulator >>> bitCount) & 31);
        }
      }
      if (bitCount > 0) words.push((accumulator << (5 - bitCount)) & 31);
      const hrpValues = [
        ...Array.from(hrp, (character) => character.charCodeAt(0) >>> 5),
        0,
        ...Array.from(hrp, (character) => character.charCodeAt(0) & 31)
      ];
      const checksumBase = [...hrpValues, ...words, 0, 0, 0, 0, 0, 0];
      const checksumValue = bech32Polymod(checksumBase) ^ 0x2bc830a3;
      const checksumWords = Array.from({ length: 6 }, (_, index) => (
        (checksumValue >>> (5 * (5 - index))) & 31
      ));
      return `${hrp}1${[...words, ...checksumWords].map((value) => bech32Alphabet[value]).join("")}`;
    }

    function importValidation(message, reason = "IMPORT_MALFORMED_JSON") {
      return fail("validation", message, { reason });
    }

    function unknownFields(record, allowed) {
      return Object.keys(record).filter((field) => !allowed.has(field));
    }

    function inspectAssetPackageImpl({ walletId, fileName, assetData }) {
      const wallet = walletById(walletId);
      if (!wallet) return importValidation("Wallet profile no longer exists.", "IMPORT_SESSION_INVALID");
      const source = String(assetData ?? "");
      const byteLength = new TextEncoder().encode(source).byteLength;
      if (!source || byteLength > assetPackageMaxBytes) {
        return importValidation(
          byteLength > assetPackageMaxBytes
            ? `Asset package exceeds the 64 KiB public JSON limit (${byteLength} bytes).`
            : "Choose a non-empty public asset package.",
        );
      }

      let pkg;
      try {
        pkg = JSON.parse(source);
      } catch {
        return importValidation("Asset package is not valid JSON.");
      }
      if (!isRecord(pkg)) return importValidation("Asset package must be a JSON object.");
      if (Object.hasOwn(pkg, "secret")) {
        return importValidation(
          "Public asset packages must not contain a secret field.",
          "IMPORT_SECRET_FIELD_FORBIDDEN",
        );
      }
      const packageUnknown = unknownFields(pkg, assetPackageFields);
      if (packageUnknown.length) {
        return importValidation(`Unknown asset package field: ${packageUnknown[0]}.`);
      }
      const missingPackageField = requiredAssetPackageFields.find((field) => !Object.hasOwn(pkg, field));
      if (missingPackageField) return importValidation(`Missing asset package field: ${missingPackageField}.`);

      const definition = pkg.definition;
      if (!isRecord(definition)) return importValidation("definition must be a JSON object.");
      const definitionUnknown = unknownFields(definition, assetDefinitionFields);
      if (definitionUnknown.length) {
        return importValidation(`Unknown asset definition field: ${definitionUnknown[0]}.`);
      }
      const missingDefinitionField = requiredAssetDefinitionFields.find((field) => !Object.hasOwn(definition, field));
      if (missingDefinitionField) return importValidation(`Missing asset definition field: ${missingDefinitionField}.`);

      if (!isHex(definition.id, 32)) return importValidation("definition.id must be a 32-byte hexadecimal identifier.");
      if (!["Coin", "Token", "Nft", "Void"].includes(definition.class)) {
        return importValidation("definition.class must be Coin, Token, Nft, or Void.");
      }
      if (typeof definition.name !== "string" || !definition.name.trim()
        || typeof definition.symbol !== "string" || !definition.symbol.trim()
        || typeof definition.domain_name !== "string") {
        return importValidation("Asset name, symbol, and domain_name must be strings.");
      }
      for (const field of ["decimals", "version", "crypto_version", "policy_flags"]) {
        if (!Number.isSafeInteger(definition[field]) || definition[field] < 0 || definition[field] > 0xff) {
          return importValidation(`definition.${field} must be an unsigned 8-bit integer.`);
        }
      }
      if (!Number.isSafeInteger(definition.serials) || definition.serials < 0 || definition.serials > 0xffffffff) {
        return importValidation("definition.serials must be an unsigned 32-bit integer.");
      }
      if (!Number.isInteger(definition.nominal) || definition.nominal < 0) {
        return importValidation("definition.nominal must be a non-negative integer.");
      }
      if (definition.metadata !== undefined && definition.metadata !== null
        && (!isRecord(definition.metadata)
          || Object.values(definition.metadata).some((value) => typeof value !== "string"))) {
        return importValidation("definition.metadata must be a string map or null.");
      }
      if (!Number.isSafeInteger(pkg.serial_id) || pkg.serial_id < 0 || pkg.serial_id > 0xffffffff
        || !Number.isInteger(pkg.amount) || pkg.amount < 0
        || !isHex(pkg.commitment, 32)
        || !isHex(pkg.nonce, 32)
        || typeof pkg.is_burned !== "boolean") {
        return importValidation("serial_id, amount, commitment, nonce, or is_burned has an invalid public DTO type.");
      }
      if (!isOptionalInteger(pkg.lock_height)
        || !isOptionalInteger(pkg.tag16) || (pkg.tag16 !== undefined && pkg.tag16 !== null && pkg.tag16 > 0xffff)
        || (pkg.is_frozen !== undefined && typeof pkg.is_frozen !== "boolean")
        || (pkg.is_slashed !== undefined && typeof pkg.is_slashed !== "boolean")
        || !["range_proof", "owner_pub", "owner_signature", "r_pub", "owner_tag", "enc_pack", "leaf_ad_id"]
          .every((field) => isOptionalString(pkg[field]))) {
        return importValidation("An optional asset package field has an invalid public DTO type.");
      }

      const hasDirectOwner = Boolean(pkg.owner_pub && pkg.owner_signature);
      const stealthParts = ["r_pub", "owner_tag", "enc_pack"].filter((field) => Boolean(pkg[field]));
      const hasStealthOwner = stealthParts.length === 3 && Boolean(pkg.leaf_ad_id);
      if (!hasDirectOwner && !hasStealthOwner) {
        return importValidation(
          "Package needs a complete direct owner signature or a complete stealth owner binding.",
          stealthParts.length ? "IMPORT_STEALTH_INCONSISTENT" : "IMPORT_CRYPTO_VERIFY_FAILED",
        );
      }

      const reviewToken = `asset-import-review-${wallet.id}-${++assetImportSequence}`;
      const preview = Object.freeze({
        schemaVersion: "asset-package-review-v1",
        file: Object.freeze({
          name: String(fileName || "asset-package.json").split(/[\\/]/).at(-1),
          bytes: byteLength
        }),
        target: Object.freeze({
          walletId: wallet.id,
          walletName: wallet.name,
          chainId: wallet.chainId
        }),
        asset: Object.freeze({
          definitionId: shortIdentifier(definition.id),
          name: definition.name.trim(),
          symbol: definition.symbol.trim(),
          class: definition.class,
          serialId: pkg.serial_id,
          amount: pkg.amount,
          decimals: definition.decimals,
          serials: definition.serials,
          nominal: definition.nominal,
          metadataEntryCount: definition.metadata ? Object.keys(definition.metadata).length : 0,
          domainName: definition.domain_name || "Not declared",
          lockHeight: pkg.lock_height ?? null,
          tag16: pkg.tag16 ?? null,
          flags: Object.freeze({
            burned: pkg.is_burned,
            frozen: Boolean(pkg.is_frozen),
            slashed: Boolean(pkg.is_slashed)
          })
        }),
        ownership: Object.freeze({
          mode: hasStealthOwner ? "Stealth receiver binding" : "Direct owner signature",
          ownerReference: shortIdentifier(hasStealthOwner ? pkg.owner_tag : pkg.owner_pub),
          leafAdId: shortIdentifier(pkg.leaf_ad_id || "")
        }),
        cryptography: Object.freeze({
          commitment: shortIdentifier(pkg.commitment),
          nonce: shortIdentifier(pkg.nonce),
          rangeProofPresent: Boolean(pkg.range_proof),
          definitionVersion: definition.version,
          cryptoVersion: definition.crypto_version,
          policyFlags: definition.policy_flags
        }),
        checks: Object.freeze({
          schemaAccepted: true,
          withinSizeLimit: true,
          secretFieldAbsent: true,
          ownershipShapeComplete: true
        })
      });
      pendingAssetImports.set(reviewToken, Object.freeze({
        walletId: wallet.id,
        assetData: source,
        preview
      }));
      return ok({ reviewToken, preview });
    }

    function walletById(walletId) {
      return state.wallets.find((wallet) => wallet.id === walletId);
    }

    function nextObjectId(wallet, family) {
      const entries = family === "voucher" ? wallet.vouchers : wallet.permissions;
      return `${family}-${wallet.id}-${entries.length + 1}`;
    }

    function transferObjectImpl({ walletId, family, objectId, recipient }) {
      const wallet = walletById(walletId);
      if (!wallet) return fail("validation", "Wallet profile no longer exists.");
      if (!["voucher", "permission"].includes(family)) return fail("validation", "Unsupported wallet object family.");
      const entries = family === "voucher" ? wallet.vouchers : wallet.permissions;
      const entry = entries.find((candidate) => candidate.id === objectId);
      if (!entry || !entry.transferable) return fail("conflict", "This wallet object is no longer transferable.");
      const normalizedRecipient = String(recipient ?? "").trim();
      if (normalizedRecipient.length < 3) return fail("validation", "Enter a valid recipient address.");
      entry.transferable = false;
      entry.status = "Sent";
      entry.tone = "settling";
      entry.recipient = normalizedRecipient;
      entry.detail = `Sent to ${normalizedRecipient} · waiting to settle`;
      return ok({ entry, family, recipient: normalizedRecipient });
    }

    function operationResult(operation) {
      return {
        operationId: operation.operationId,
        status: operation.status,
        completed: { ...operation.completed }
      };
    }

    function revalidateExternalReviewHandoff({ walletId, handoff } = {}) {
      const wallet = walletById(walletId);
      if (!wallet) return fail("validation", "Wallet profile no longer exists.");
      if (handoff?.schemaVersion === "contact-action-handoff-v1") {
        if (!handoff.handoffId
          || handoff.action !== "pay"
          || handoff.target?.routeId !== "wallet.send"
          || handoff.target?.flow !== "send"
          || handoff.target?.referenceDomain !== "wallet_recipient"
          || !/^wallet_receiver_ref_[a-z0-9_]+(?:…[a-z0-9]+)?$/i.test(String(handoff.reference || ""))
          || handoff.constraints?.revalidationRequired !== true
          || handoff.constraints?.prefillOnly !== true
          || handoff.constraints?.networkRequest !== false
          || handoff.constraints?.walletMutation !== false
          || handoff.constraints?.settlementMutation !== false) {
          return fail("external_review_rejected", "The Contact action failed Wallet schema and authority validation.");
        }
        return ok(Object.freeze({
          schemaVersion: "wallet-external-review-validation-v1",
          handoffId: handoff.handoffId,
          walletId: wallet.id,
          target: handoff.target,
          draft: null,
          result: "accepted_for_wallet_entry",
          walletMutation: false,
          settlementMutation: false
        }));
      }
      if (!handoff
        || handoff.schemaVersion !== "messenger-wallet-review-handoff-v1"
        || !handoff.handoffId
        || handoff.source?.requestType !== "payment_request"
        || handoff.target?.routeId !== "wallet.send"
        || handoff.target?.flow !== "send"
        || handoff.request?.objectFamily !== "asset"
        || handoff.constraints?.prefillOnly !== true
        || handoff.constraints?.walletRevalidationRequired !== true
        || handoff.constraints?.walletMutation !== false
        || handoff.constraints?.settlementMutation !== false) {
        return fail("external_review_rejected", "The external advisory handoff failed Wallet schema and authority validation.");
      }
      const draft = handoff.draft;
      const asset = demo.ASSET_CATALOG.find(({ key }) => key === draft?.itemKey);
      const amount = Number(draft?.amount);
      if (!asset
        || !wallet.assetKeys.includes(asset.key)
        || draft?.family !== "asset"
        || draft?.recipient !== ""
        || !Number.isFinite(amount)
        || amount <= 0
        || amount > Number((asset.key === "z00z" ? wallet.summary.available : asset.demoBalance || "0").replaceAll(",", ""))) {
        return fail("external_review_rejected", "Wallet rejected the stale, unsupported, or over-broad advisory prefill.");
      }
      return ok(Object.freeze({
        schemaVersion: "wallet-external-review-validation-v1",
        handoffId: handoff.handoffId,
        walletId: wallet.id,
        target: handoff.target,
        draft: handoff.draft,
        result: "accepted_for_wallet_entry",
        walletMutation: false,
        settlementMutation: false
      }));
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,
      revalidateExternalReviewHandoff,

      inspectAssetPackage({ walletId, fileName, assetData }) {
        return inspectAssetPackageImpl({ walletId, fileName, assetData });
      },

      prepareAssetImport({ walletId, reviewToken }) {
        const pending = pendingAssetImports.get(String(reviewToken ?? ""));
        if (!pending || pending.walletId !== walletId) {
          return importValidation(
            "The reviewed package is stale or belongs to another wallet.",
            "IMPORT_SESSION_INVALID",
          );
        }
        pendingAssetImports.delete(reviewToken);
        return ok(Object.freeze({
          rpcMethod: "wallet.asset.import_asset",
          status: "native_verification_required",
          walletId,
          walletMutation: false,
          resultFields: Object.freeze([
            "asset_id",
            "serial_id",
            "symbol",
            "class",
            "success",
            "message",
            "is_inserted",
            "asset_already_exists"
          ])
        }));
      },

      listWallets() {
        return ok({
          wallets: state.wallets.map((wallet) => ({
            ...wallet,
            assetKeys: [...wallet.assetKeys],
            vouchers: wallet.vouchers.map((voucher) => ({ ...voucher })),
            permissions: wallet.permissions.map((permission) => ({ ...permission })),
            activities: [...wallet.activities]
          }))
        });
      },

      getReceiverCard({ walletId }) {
        const wallet = walletById(walletId);
        if (!wallet) return fail("validation", "Wallet profile no longer exists.");
        const ownerHandle = stablePublicHex(`${wallet.id}:receiver-owner`, 32);
        const viewKey = stablePublicHex(`${wallet.id}:receiver-view`, 32);
        const identityKey = stablePublicHex(`${wallet.id}:receiver-identity`, 32);
        const signature = stablePublicHex(`${wallet.id}:receiver-signature`, 64);
        const registryEntryId = stablePublicHex(`${wallet.id}:receiver-registry`, 32);
        return ok(Object.freeze({
          owner_handle: ownerHandle,
          view_key: viewKey,
          identity_key: identityKey,
          signature,
          card_compact: `z00zrc1:${stablePublicHex(`${wallet.id}:receiver-record`, 192)}`,
          registry_entry_id: registryEntryId,
          card_epoch: 0,
          owner_handle_display: bech32mOwnerHandle(ownerHandle)
        }));
      },

      createProfile({ name, chainId = "mainnet", scan = "Scanning" }) {
        const normalized = String(name ?? "").trim();
        if (normalized.length < 2 || normalized.length > 32) {
          return fail("validation", "Wallet name must contain 2–32 characters.");
        }
        if (!demo.PORT_CONTRACT.walletChains.includes(chainId)) {
          return fail("validation", "Choose a supported wallet chain.");
        }
        const wallet = demo.createWalletProfile(state.wallets, normalized, chainId, scan);
        state.wallets.push(wallet);
        return ok({ wallet });
      },

      removeProfiles({ walletIds, selectedWalletId }) {
        const ids = new Set(Array.from(walletIds ?? [], String));
        if (!ids.size) return fail("validation", "Select at least one wallet profile.");
        const selectedIndex = state.wallets.findIndex((wallet) => wallet.id === selectedWalletId);
        const removed = state.wallets.filter((wallet) => ids.has(wallet.id));
        if (!removed.length) return fail("validation", "Selected wallet profiles no longer exist.");
        state.wallets = state.wallets.filter((wallet) => !ids.has(wallet.id));
        removed.forEach((wallet) => delete state.walletPreferences[wallet.id]);
        const nextSelectedWalletId = ids.has(selectedWalletId)
          ? state.wallets[selectedIndex]?.id
            || state.wallets[Math.max(0, selectedIndex - 1)]?.id
            || state.wallets[0]?.id
            || null
          : selectedWalletId;
        return ok({ removed, selectedWalletId: nextSelectedWalletId });
      },

      renameWallet({ walletId, name }) {
        const normalized = String(name ?? "").trim();
        if (normalized.length < 2 || normalized.length > 32) {
          return fail("validation", "Wallet name must contain 2–32 characters.");
        }
        const wallet = state.wallets.find((entry) => entry.id === walletId);
        if (!wallet) return fail("validation", "Wallet profile no longer exists.");
        wallet.name = normalized;
        wallet.initials = normalized.slice(0, 1).toUpperCase();
        return ok({ walletId, name: normalized });
      },

      changePassword({ walletId, currentPassword, newPassword }) {
        if (!state.wallets.some((wallet) => wallet.id === walletId)) {
          return fail("validation", "Wallet profile no longer exists.");
        }
        if (String(currentPassword ?? "").length < 8 || String(newPassword ?? "").length < 8) {
          return fail("validation", "Both passwords must contain at least 8 characters.");
        }
        if (currentPassword === newPassword) {
          return fail("validation", "The new password must differ from the current password.");
        }
        // The concept intentionally validates and discards the supplied strings.
        return ok({ walletId, changed: true });
      },

      createVoucher({ walletId, title, amount, expiry }) {
        const wallet = walletById(walletId);
        const normalizedTitle = String(title ?? "").trim();
        const normalizedAmount = Number(amount);
        if (!wallet) return fail("validation", "Wallet profile no longer exists.");
        if (normalizedTitle.length < 2 || normalizedTitle.length > 48) {
          return fail("validation", "Voucher name must contain 2–48 characters.");
        }
        if (!Number.isFinite(normalizedAmount) || normalizedAmount <= 0) {
          return fail("validation", "Voucher value must be greater than zero.");
        }
        if (!/^\d{4}-\d{2}-\d{2}$/.test(String(expiry ?? ""))) {
          return fail("validation", "Choose a voucher expiry date.");
        }
        const voucher = {
          id: nextObjectId(wallet, "voucher"),
          kind: "refund",
          title: normalizedTitle,
          detail: `Created by ${wallet.name} · ready to transfer`,
          value: `${normalizedAmount.toFixed(2)} Z00Z`,
          status: "Ready",
          tone: "active",
          detailFlow: "voucher-detail",
          expiry: String(expiry),
          transferable: true
        };
        wallet.vouchers.push(voucher);
        return ok({ voucher });
      },

      createPermission({ walletId, title, action, scope, uses, expiry }) {
        const wallet = walletById(walletId);
        const normalizedTitle = String(title ?? "").trim();
        const normalizedScope = String(scope ?? "").trim();
        const useCount = Number(uses);
        if (!wallet) return fail("validation", "Wallet profile no longer exists.");
        if (normalizedTitle.length < 2 || normalizedTitle.length > 48) {
          return fail("validation", "Permission name must contain 2–48 characters.");
        }
        if (normalizedScope.length < 3) return fail("validation", "Enter a bounded permission scope.");
        if (!Number.isInteger(useCount) || useCount < 1 || useCount > 100) {
          return fail("validation", "Permission uses must be between 1 and 100.");
        }
        if (!/^\d{4}-\d{2}-\d{2}$/.test(String(expiry ?? ""))) {
          return fail("validation", "Choose a permission expiry date.");
        }
        const permission = {
          id: nextObjectId(wallet, "permission"),
          kind: "deploy",
          title: normalizedTitle,
          detail: `${action} · ${normalizedScope} · transfer-ready`,
          remaining: `${useCount} ${useCount === 1 ? "use" : "uses"}`,
          classLabel: "Bounded permission",
          action: String(action),
          scope: normalizedScope,
          delegation: "One transfer",
          expiry: String(expiry),
          rightId: `right_${wallet.id}_${wallet.permissions.length + 1}`,
          typeLabel: "bounded_permission",
          status: "Held",
          tone: "active",
          transferable: true
        };
        wallet.permissions.push(permission);
        return ok({ permission });
      },

      transferObject({ walletId, family, objectId, recipient }) {
        return transferObjectImpl({ walletId, family, objectId, recipient });
      },

      submitPayment({ walletId, family, itemKey, amount, recipient, idempotencyKey, scenario = "success" }) {
        const wallet = walletById(walletId);
        if (!wallet) return fail("validation", "Wallet profile no longer exists.");
        const normalizedRecipient = String(recipient ?? "").trim();
        if (normalizedRecipient.length < 3) return fail("validation", "Enter a valid recipient address.");
        const normalizedIdempotencyKey = String(idempotencyKey ?? "").trim();
        if (!normalizedIdempotencyKey) return fail("validation", "A payment idempotency key is required.");

        const existingOperationId = operationIdByIdempotencyKey.get(normalizedIdempotencyKey);
        if (existingOperationId) return ok(operationResult(operations.get(existingOperationId)));

        const operationId = `payment-${wallet.id}-${++operationSequence}`;
        let label;
        let amountLabel;
        let activityType;
        if (family === "asset") {
          const asset = demo.ASSET_CATALOG.find((entry) => entry.key === itemKey);
          const normalizedAmount = Number(amount);
          if (!asset || !wallet.assetKeys.includes(asset.key)) return fail("conflict", "This wallet asset is no longer available.");
          if (!Number.isFinite(normalizedAmount) || normalizedAmount <= 0 || (!asset.divisible && !Number.isInteger(normalizedAmount))) {
            return fail("validation", "Enter a valid amount for the selected asset.");
          }
          label = asset.label;
          amountLabel = `${asset.divisible ? normalizedAmount.toFixed(2) : normalizedAmount} ${asset.unit}`;
          activityType = asset.key === "z00z" ? "money" : "asset";
        } else {
          const objectId = String(itemKey || "").split(":").slice(1).join(":");
          const result = transferObjectImpl({ walletId, family, objectId, recipient: normalizedRecipient });
          if (!result.ok) return result;
          label = result.data.entry.title;
          amountLabel = family === "voucher" ? result.data.entry.value : result.data.entry.remaining;
          activityType = family;
        }

        wallet.activities.unshift({
          id: `${family}-send-${wallet.activities.length + 1}`,
          type: activityType,
          direction: "out",
          title: `${label} sent`,
          detail: `Sent to ${normalizedRecipient} · waiting to settle`,
          amount: family === "permission" ? "" : `− ${amountLabel}`,
          time: "Now",
          status: "settling"
        });

        const recipientLabel = normalizedRecipient.length > 24
          ? `${normalizedRecipient.slice(0, 12)}…${normalizedRecipient.slice(-8)}`
          : normalizedRecipient;
        const operation = Object.freeze({
          operationId,
          status: "pending_confirmation",
          completed: Object.freeze({ family, label, amountLabel, recipientLabel })
        });
        operations.set(operationId, operation);
        operationIdByIdempotencyKey.set(normalizedIdempotencyKey, operationId);

        if (scenario === "timeout_unknown_outcome") {
          return fail(
            "timeout_unknown_outcome",
            "Native submission timed out after handoff. Reconcile this operation before any retry.",
            { operationId }
          );
        }
        return ok(operationResult(operation));
      },

      reconcileOperation({ operationId }) {
        const operation = operations.get(String(operationId ?? ""));
        if (!operation) return fail("conflict", "The operation record is unavailable; do not submit again until native diagnostics resolve it.");
        return ok(operationResult(operation));
      }
    });
  }

  Object.assign(root.Z00ZDemo, { createMockWalletGateway });
})(typeof window === "undefined" ? globalThis : window);
