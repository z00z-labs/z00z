"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.PORT_CONTRACT || !demo.createWalletProfile) {
    throw new Error("Z00Z demo contracts and fixtures must load before the mock gateway.");
  }

  const ok = (data = {}) => Object.freeze({ ok: true, data });
  const fail = (code, message) => Object.freeze({
    ok: false,
    error: Object.freeze({ code, message })
  });

  function createMockWalletGateway(state) {
    function walletById(walletId) {
      return state.wallets.find((wallet) => wallet.id === walletId);
    }

    function nextObjectId(wallet, family) {
      const entries = family === "voucher" ? wallet.vouchers : wallet.permissions;
      return `${family}-${wallet.id}-${entries.length + 1}`;
    }

    return Object.freeze({
      contractVersion: demo.PORT_CONTRACT.version,

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
    });
  }

  Object.assign(root.Z00ZDemo, { createMockWalletGateway });
})(typeof window === "undefined" ? globalThis : window);
