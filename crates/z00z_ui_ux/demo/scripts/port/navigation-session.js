"use strict";

((root) => {
  const demo = root.Z00ZDemo;
  if (!demo?.navigationChildren) {
    throw new Error("Navigation model must load before navigation session state.");
  }

  const STORAGE_VERSION = 1;
  const STORAGE_PREFIX = "z00z.navigation";
  const rootBranchIds = new Set(
    demo.navigationChildren()
      .filter((node) => node.target.kind === "branch")
      .map(({ id }) => id)
  );

  function normalizeExpandedBranchIds(branchIds) {
    if (!Array.isArray(branchIds)) return null;
    return [...new Set(branchIds.filter((branchId) => rootBranchIds.has(branchId)))].sort();
  }

  function normalizeScrollTop(value) {
    const scrollTop = Number(value);
    return Number.isFinite(scrollTop) && scrollTop >= 0 ? Math.round(scrollTop) : 0;
  }

  function normalizeSnapshot(snapshot) {
    const expandedBranchIds = normalizeExpandedBranchIds(snapshot?.expandedBranchIds);
    if (!expandedBranchIds) return null;
    return {
      activeRoute: demo.PORT_CONTRACT.routes.includes(snapshot?.activeRoute)
        ? snapshot.activeRoute
        : null,
      expandedBranchIds,
      scrollPositions: {
        desktop: normalizeScrollTop(snapshot?.scrollPositions?.desktop),
        mobile: normalizeScrollTop(snapshot?.scrollPositions?.mobile)
      },
      drawerOpen: Boolean(snapshot?.drawerOpen)
    };
  }

  function createNavigationSession(surface, storageOverride) {
    if (!/^[a-z][a-z0-9-]*$/u.test(surface || "")) {
      throw new TypeError("Navigation session surface must be a bounded identifier.");
    }
    const key = `${STORAGE_PREFIX}.${surface}.v${STORAGE_VERSION}`;

    function storage() {
      if (storageOverride) return storageOverride;
      try {
        return root.sessionStorage;
      } catch {
        return null;
      }
    }

    function read() {
      try {
        const value = storage()?.getItem(key);
        if (!value) return null;
        const parsed = JSON.parse(value);
        if (parsed?.version !== STORAGE_VERSION) return null;
        return normalizeSnapshot(parsed);
      } catch {
        return null;
      }
    }

    function write(snapshot) {
      const normalized = normalizeSnapshot(snapshot);
      if (!normalized) return false;
      try {
        storage()?.setItem(key, JSON.stringify({
          version: STORAGE_VERSION,
          ...normalized
        }));
        return true;
      } catch {
        return false;
      }
    }

    return Object.freeze({ key, read, write });
  }

  Object.assign(root.Z00ZDemo, {
    NAVIGATION_SESSION_VERSION: STORAGE_VERSION,
    createNavigationSession
  });
})(typeof window === "undefined" ? globalThis : window);
