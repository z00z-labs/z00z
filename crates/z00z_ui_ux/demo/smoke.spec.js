const { expect, test } = require("playwright/test");
const { readFile } = require("node:fs/promises");
const path = require("node:path");

const demoUrl = process.env.Z00Z_WALLET_DEMO_URL || "http://127.0.0.1:4173/index.html";
const demoDir = __dirname;

async function openStandaloneHelp(page, trigger) {
  const popupPromise = page.waitForEvent("popup");
  await trigger.click();
  const helpPage = await popupPromise;
  await helpPage.waitForLoadState("domcontentloaded");
  return helpPage;
}

async function selectCanonicalRoute(page, routeId, { mobile = false } = {}) {
  const navigation = mobile
    ? page.locator("#mobile-popup-menu")
    : page.locator(".sidebar");
  const route = navigation.locator(`[data-navigation-route="${routeId}"]`);
  await expect(route).toHaveCount(1);

  const ancestors = await route.evaluate((node) => {
    const branchIds = [];
    let current = node.parentElement;
    while (current) {
      if (current.classList.contains("navigation-tree-children")) {
        const toggle = document.getElementById(current.getAttribute("aria-labelledby"));
        if (toggle?.dataset.navigationBranch) branchIds.push(toggle.dataset.navigationBranch);
      }
      current = current.parentElement;
    }
    return branchIds.reverse();
  });

  for (const branchId of ancestors) {
    const branch = navigation.locator(`[data-navigation-branch="${branchId}"]`);
    if (await branch.getAttribute("aria-expanded") === "false") await branch.click();
  }
  await route.click();
}

async function expectNoViewportOverflow(page, label = "responsive geometry") {
  const geometry = await page.evaluate(() => ({
    viewport: window.innerWidth,
    document: document.documentElement.scrollWidth,
    rootRect: document.documentElement.getBoundingClientRect().toJSON(),
    bodyRect: document.body.getBoundingClientRect().toJSON(),
    bodyScrollWidth: document.body.scrollWidth,
    main: document.querySelector("#main-content, #help-main")?.getBoundingClientRect().toJSON(),
    offenders: [...document.querySelectorAll("body *")]
      .map((node) => ({ node, rect: node.getBoundingClientRect() }))
      .filter(({ rect }) => rect.width > 0 && (rect.left < -1 || rect.right > window.innerWidth + 1))
      .slice(0, 20)
      .map(({ node, rect }) => ({
        selector: `${node.tagName.toLowerCase()}${node.id ? `#${node.id}` : ""}${[...node.classList].slice(0, 3).map((name) => `.${name}`).join("")}`,
        left: Math.round(rect.left),
        right: Math.round(rect.right),
        width: Math.round(rect.width),
      })),
    scrollOffenders: [...document.querySelectorAll("body *")]
      .filter((node) => node.scrollWidth > node.clientWidth + 1 && getComputedStyle(node).overflowX === "visible")
      .slice(0, 20)
      .map((node) => ({
        selector: `${node.tagName.toLowerCase()}${node.id ? `#${node.id}` : ""}${[...node.classList].slice(0, 3).map((name) => `.${name}`).join("")}`,
        clientWidth: node.clientWidth,
        scrollWidth: node.scrollWidth,
      })),
  }));
  expect(
    geometry.document,
    `${label}: document width; diagnostics=${JSON.stringify({
      offenders: geometry.offenders,
      scrollOffenders: geometry.scrollOffenders,
      rootRect: geometry.rootRect,
      bodyRect: geometry.bodyRect,
      bodyScrollWidth: geometry.bodyScrollWidth,
    })}`,
  ).toBeLessThanOrEqual(geometry.viewport + 1);
  expect(geometry.main.left, `${label}: main left edge`).toBeGreaterThanOrEqual(-1);
  expect(geometry.main.right, `${label}: main right edge`).toBeLessThanOrEqual(geometry.viewport + 1);
}

async function visibleContextNavigation(page, desktopSelector = ".workspace-layout > .context-rail") {
  const mobile = await page.evaluate(() => window.matchMedia("(max-width: 768px)").matches);
  return page.locator(mobile ? "#mobile-topbar-context" : desktopSelector);
}

test("canonical navigation replaces global tabs and has no stale hierarchy styles", async ({ page }) => {
  const [index, app, components] = await Promise.all([
    readFile(path.join(demoDir, "index.html"), "utf8"),
    readFile(path.join(demoDir, "app.js"), "utf8"),
    readFile(path.join(demoDir, "styles/components.css"), "utf8"),
  ]);

  expect(index).toContain('id="app-navigation-tree"');
  expect(index).toContain('id="app-navigation-terminal"');
  expect(index).toContain('class="desktop-topbar-brand brand"');
  expect(index).not.toContain('id="wallet-tabs"');
  expect(app).not.toContain('closest(".system-nav")');
  expect(app).not.toContain('closest("#network-nav")');
  expect(components).not.toMatch(/\.wallet-tabs\b|\.wallet-tab\b|\.system-nav\b/);

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  await expect(page.locator("#wallet-tabs, #network-nav")).toHaveCount(0);
  await expect(page.locator(".desktop-topbar-brand .brand-mark")).toBeVisible();
  await expect(page.locator("#app-navigation-tree > .navigation-tree-branch > [data-navigation-branch]")).toHaveCount(5);
  await expect(page.locator('#app-navigation-tree [data-navigation-route="wallet.overview"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-branch="wallet.assets-rights"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-route="wallet.assets"]')).toContainText("Assets");
  await expect(page.locator('#app-navigation-tree [data-navigation-route="wallet.vouchers"], #app-navigation-tree [data-navigation-route="wallet.permissions"], #app-navigation-tree [data-navigation-route="wallet.quarantine"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="wallet.settings"]')).toBeVisible();
  await expect(page.locator('#app-navigation-tree [data-navigation-route^="wallet.settings."]')).toHaveCount(1);
  await expect(page.locator('#app-navigation-tree > [data-navigation-route="contacts.list"]')).toHaveCount(1);
  await expect(page.locator('#app-navigation-tree [data-navigation-branch="settings"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-terminal > .navigation-tree-branch [data-navigation-branch="settings"]')).toContainText("Settings");
  await expect(page.locator("#app-navigation-terminal > .navigation-tree-terminal")).toHaveText(["Help", "About", "Log out"]);
  await expect(page.locator('#app-navigation-terminal [data-navigation-route="about"]')).toBeVisible();
  await expect(page.locator("#app-navigation-terminal .app-version")).toHaveText("Version 0.1.0");

  const walletPlaceholderGeometry = await page.evaluate(() => {
    const topbar = document.querySelector(".topbar").getBoundingClientRect();
    const label = document.querySelector(".sidebar-label").getBoundingClientRect();
    return {
      topbarBottom: topbar.bottom,
      labelTop: label.top,
    };
  });
  expect(walletPlaceholderGeometry.labelTop - walletPlaceholderGeometry.topbarBottom).toBeCloseTo(30, 0);

  const desktopTypography = await page.locator('#app-navigation-tree [data-navigation-branch="wallet"]').evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      family: style.fontFamily,
      size: style.fontSize,
      weight: style.fontWeight,
      lineHeight: style.lineHeight,
    };
  });
  expect(desktopTypography.family).toContain("Geist");
  expect(desktopTypography).toMatchObject({
    size: "16px",
    weight: "700",
    lineHeight: "20px",
  });

  const topbarTypography = await page.locator("#page-title").evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      family: style.fontFamily,
      size: style.fontSize,
    };
  });
  expect(topbarTypography.family).toContain("Geist");
  expect(topbarTypography.size).toBe("28px");

  await page.goto(`${demoUrl}?route=telemetry.reticulum.overview`);
  await expect(page.locator("#wallet-identity")).toBeVisible();
  await expect(page.locator(".wallet-identity-address")).toHaveText("ZxChpo…2Mj8Pt");
  await expect(page.locator(".wallet-identity-name")).toHaveText("Everyday wallet");
  await expect(page.locator("#page-title")).toHaveText("Reticulum");
  await expect(page.locator("#page-context")).toBeHidden();
  const desktopTopbarOrder = await page.evaluate(() => {
    const brand = document.querySelector(".desktop-topbar-brand").getBoundingClientRect();
    const logo = document.querySelector(".desktop-topbar-brand .brand-mark").getBoundingClientRect();
    const wallet = document.querySelector("#wallet-identity").getBoundingClientRect();
    const heading = document.querySelector(".topbar-address-group").getBoundingClientRect();
    const topbarStyle = getComputedStyle(document.querySelector(".topbar"));
    return {
      brand: { left: brand.left, right: brand.right, center: brand.left + brand.width / 2 },
      logo: { left: logo.left, width: logo.width, height: logo.height, centerY: logo.top + logo.height / 2 },
      wallet: { left: wallet.left, right: wallet.right },
      heading: { left: heading.left },
      topbar: { centerY: brand.top + brand.height / 2, borderBottomWidth: topbarStyle.borderBottomWidth },
    };
  });
  expect(desktopTopbarOrder.wallet.left).toBeGreaterThanOrEqual(desktopTopbarOrder.brand.right - 1);
  expect(desktopTopbarOrder.heading.left).toBeGreaterThanOrEqual(desktopTopbarOrder.wallet.right - 1);
  expect(desktopTopbarOrder.logo).toMatchObject({ width: 52, height: 52 });
  expect(desktopTopbarOrder.logo.left - desktopTopbarOrder.brand.left).toBe(18);
  expect(desktopTopbarOrder.logo.centerY).toBeCloseTo(desktopTopbarOrder.topbar.centerY, 0);
  expect(desktopTopbarOrder.topbar.borderBottomWidth).toBe("0px");
});

test("desktop tree keeps root accordions independent and opens sublevels inside the workspace", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  const wallet = page.locator('#app-navigation-tree [data-navigation-branch="wallet"]');
  const telemetry = page.locator('#app-navigation-tree [data-navigation-branch="telemetry"]');
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await expect(telemetry).toHaveAttribute("aria-expanded", "false");
  await telemetry.click();
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("#page-title")).toHaveText("Assets");
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "false");
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("#page-title")).toHaveText("Assets");
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await telemetry.click();
  await expect(telemetry).toHaveAttribute("aria-expanded", "false");
  await expect(wallet).toHaveAttribute("aria-expanded", "true");

  await telemetry.click();
  await expect(page.locator('#app-navigation-tree [data-navigation-branch="telemetry.reticulum"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.reticulum"]')).toBeVisible();
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.onionnet"]')).toBeVisible();
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.aggregators"]')).toBeVisible();
  await expect(page.locator('#app-navigation-tree [data-navigation-route="telemetry.reticulum.node"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-route^="telemetry.onionnet."]')).toHaveCount(1);
  await expect(page.locator('#app-navigation-tree [data-navigation-route^="telemetry.aggregators."]')).toHaveCount(1);
  await page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.reticulum"]').click();
  await expect(page.locator(".telemetry-workspace-layout > .context-rail")).toBeVisible();
  await expect(page.locator(".telemetry-workspace-context [data-workspace-route]")).toHaveCount(8);
  await page.locator('[data-workspace-route="telemetry.reticulum.node"]').click();
  await expect(page.locator("#page-title")).toHaveText("Node");
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.reticulum"]')).toHaveAttribute("aria-current", "page");
  await expect(page.locator('#app-navigation-tree [aria-current="page"]')).toHaveCount(1);

  await page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.watchers"]').click();
  await expect(page.locator(".telemetry-workspace-context [data-workspace-route]")).toHaveCount(6);
  await page.locator('[data-workspace-route="telemetry.watchers.alerts"]').click();
  await expect(page.locator("#page-title")).toHaveText("Alerts");
  await expect(page.locator('[data-watcher-screen="alerts"]')).toBeVisible();
  await expect(page.locator(".route-preview")).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-route="telemetry.watchers.alerts"]')).toHaveCount(0);
  await expect(page.locator('#app-navigation-tree [data-navigation-workspace="telemetry.watchers"]')).toHaveAttribute("aria-current", "page");

  await selectCanonicalRoute(page, "wallet.send");
  await expect(page.locator('#app-navigation-tree [data-navigation-route="wallet.send"]')).toHaveAttribute("aria-current", "page");
  await expect(page.locator("#page-title")).toHaveText("Send");
  await expect(page.locator("#main-content")).toContainText("Send");

  await selectCanonicalRoute(page, "settings.appearance");
  await expect(page.locator("#page-title")).toHaveText("Appearance");
  await expect(page.locator('#main-content [data-palette="z00z-default"]')).toHaveCount(1);
  await expect(page.locator('#main-content [data-palette="z00z-corporate"]')).toHaveCount(1);
});

test("wallet profiles and object families remain selected through the canonical tree", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  await page.locator('[data-wallet-id="savings"]').click();
  await expect(page.locator("#wallet-identity")).toContainText("Savings");
  await expect(page.locator('#wallet-nav [data-wallet-id="savings"]')).toHaveAttribute("aria-current", "page");

  await page.locator(".asset-identity-button").first().click();
  await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  await page.locator('[data-wallet-id="everyday"]').click();
  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  await page.locator('[data-wallet-id="savings"]').click();
  await expect(page.locator(".claim-row")).toHaveCount(0);
  await expect(page.locator(".object-empty-state")).toBeVisible();
  await page.locator('[data-wallet-id="everyday"]').click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  await page.locator('[data-wallet-section="permissions"]').click();
  await expect(page.locator(".permission-row")).toHaveCount(8);
  await expect(page.locator('[aria-label="Permission filters"] button')).toHaveText(["Held", "Delegated", "Used"]);
});

test("deep links, drafts, pending state, and native Back preserve route-overlay order", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    await expect(page.locator("#wallet-statusbar")).toContainText("Pending in960.00 Z00Z");
    await expect(page.locator("#wallet-statusbar")).toContainText("Pending out240.00 Z00Z");

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.send", { mobile: viewport.mobile });
    await page.locator("#send-recipient").fill("z00z1private-request");
    await page.locator("#send-amount").fill("42.75");
    await page.locator("#send-memo").fill("Preserve this draft");

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.receive", { mobile: viewport.mobile });
    await expect(page).toHaveURL(/route=wallet\.receive/);
    await page.goBack();
    await expect(page).toHaveURL(/route=wallet\.send/);
    await expect(page.locator("#send-recipient")).toHaveValue("z00z1private-request");
    await expect(page.locator("#send-amount")).toHaveValue("42.75");
    await expect(page.locator("#send-memo")).toHaveValue("Preserve this draft");

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.assets", { mobile: viewport.mobile });
    await page.locator(".asset-identity-button").first().click();
    await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();

    await page.goBack();
    await expect(page.locator("#flow-dialog")).not.toBeVisible();
    await expect(page).toHaveURL(/route=wallet\.assets/);
    await expect(page.locator("#page-title")).toHaveText("Assets");

    await page.goForward();
    await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#flow-dialog")).not.toBeVisible();
    await expect(page).toHaveURL(/route=wallet\.assets/);

    await page.goBack();
    await expect(page).toHaveURL(/route=wallet\.send/);
    await expect(page.locator("#send-memo")).toHaveValue("Preserve this draft");

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.history", { mobile: viewport.mobile });
    await expect(page.locator(".activity-row .status-badge.is-settling").first()).toHaveText("Settling");
    await expect(page.locator("#wallet-statusbar")).toContainText("Pending in960.00 Z00Z");
    await expect(page.locator("#wallet-statusbar")).toContainText("Pending out240.00 Z00Z");
  }
});

test("wallet actions are reached from the tree without recreating tab controls", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  await selectCanonicalRoute(page, "wallet.send");
  await expect(page.locator("#send-entry")).toBeVisible();
  await expect(page.locator("#send-entry")).toContainText("Recipient or private request");

  await selectCanonicalRoute(page, "wallet.receive");
  await expect(page.locator("#main-content")).toContainText("Receive");
  await expect(page.locator(".mock-qr")).toBeVisible();

  await selectCanonicalRoute(page, "wallet.history");
  await expect(page.locator(".activity-row")).toHaveCount(7);

  await selectCanonicalRoute(page, "wallet.backup");
  await expect(page.locator("#main-content")).toContainText("Backup");
});

test("capability profiles stay typed without redundant boundary cards", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);

    for (const capability of [
      { route: "wallet.swap", maturity: "live", evidence: "fixture" },
      { route: "wallet.staking.stake", capabilityId: "wallet.staking", maturity: "live", evidence: "fixture" },
      { route: "wallet.exchange", maturity: "target", evidence: "none" },
    ]) {
      await page.goto(`${demoUrl}?route=${capability.route}`);
      const profile = await page.evaluate(
        (capabilityId) => window.Z00ZDemo.capabilityProfile(capabilityId),
        capability.capabilityId || capability.route,
      );
      expect(profile).toMatchObject({
        maturity: capability.maturity,
        availability: "unavailable",
        evidenceSource: capability.evidence,
        freshness: "not_applicable",
        presentationMode: "product",
      });
      await expect(page.locator(".capability-boundary")).toHaveCount(0);
      await expectNoViewportOverflow(page);
    }

    await page.goto(`${demoUrl}?route=wallet.swap`);
    await expect(page.getByRole("button", { name: "Preview experimental recipe" })).toBeVisible();
    await page.getByRole("button", { name: "Preview experimental recipe" }).click();
    await expect(page.locator("#toast-region")).toContainText("needs a verified quote");

    await page.goto(`${demoUrl}?route=wallet.staking.stake`);
    await expect(page.locator(".staking-summary")).not.toContainText("0.00 Z00Z");
    await expect(page.locator(".staking-summary")).toContainText("Unavailable");
    await page.getByRole("button", { name: "Review stake" }).click();
    await expect(page.locator("#toast-region")).toContainText("needs validator and lock-up terms");
    await page.locator('[data-workspace-route="wallet.staking.unstake"]').click();
    await expect(page.locator("#main-content")).toContainText("Unstake");
    await expect(page.locator("#unstake-position")).toBeVisible();
    await page.locator('[data-demo-action="prepare-unstake"]').click();
    await expect(page.locator("#toast-region")).toContainText("needs an authoritative staked balance and unlock terms");

    await page.goto(`${demoUrl}?route=wallet.exchange`);
    await page.locator("#exchange-amount").fill("10");
    await page.locator("#exchange-recipient").fill("target-recipient");
    await page.locator("#exchange-refund").fill("target-refund");
    await page.getByRole("button", { name: "Review target request" }).click();
    await expect(page.locator(".exchange-unavailable-grid strong")).toHaveText([
      "Unavailable",
      "Unavailable",
      "Unavailable",
      "Unavailable",
      "Unavailable",
      "Unavailable",
      "Unavailable",
    ]);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await expectNoViewportOverflow(page);
  }
});

test("Send reconciles an unknown native outcome without duplicating the operation", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.send&operationScenario=timeout_unknown_outcome`);
    await page.locator("#send-recipient").fill("z00z1native-boundary");
    await page.locator("#send-amount").fill("12.50");
    await page.locator("#send-memo").fill("Keep across native timeout");
    await page.locator("#send-entry").evaluate((form) => form.requestSubmit());
    await expect(page.locator('[data-send-action="submit"]')).toBeVisible();
    await page.locator('[data-send-action="submit"]').click();

    await expect(page.locator(".operation-progress-state")).toBeVisible();
    await expect(page.locator(".operation-progress-list li")).toHaveCount(3);
    await expectNoViewportOverflow(page);

    const errorState = page.locator(".operation-error-state");
    await expect(errorState).toBeVisible();
    await expect(errorState).toContainText("Reconcile this operation before any retry");
    await expect(page.locator(".send-panel-body")).toContainText("Preserved for this wallet");
    await expect(page.locator(".send-panel-body .mono")).toHaveText(/^payment-everyday-\d+$/);
    await expectNoViewportOverflow(page);

    await page.locator('[data-send-action="reconcile"]').click();
    await expect(page.locator("#send-panel-title")).toHaveText("Reconciling operation");
    await expect(page.locator(".operation-progress-state")).toBeVisible();
    await expect(page.locator(".operation-progress-list .is-done")).toHaveCount(2);
    await expectNoViewportOverflow(page);

    await expect(page.locator(".result-state")).toBeVisible();
    await expect(page.locator(".receipt-ref")).toHaveText(/^payment-everyday-\d+$/);
    await expect(page.locator(".send-panel-body")).toContainText("Submitted · pending confirmation");
    await page.locator('[data-send-action="history"]').click();
    await expect(page.locator(".activity-row")).toHaveCount(8);
    await expect(page.locator(".activity-row").first()).toContainText("Z00Z sent");
    await expectNoViewportOverflow(page);
  }
});

test("Default and Corporate palettes switch immediately with one ACTIVE marker", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=settings.appearance`);

    const palettes = page.locator("#main-content [data-palette]");
    const defaultPalette = page.locator('#main-content [data-palette="z00z-default"]');
    const corporatePalette = page.locator('#main-content [data-palette="z00z-corporate"]');
    await expect(palettes).toHaveCount(2);
    expect(await palettes.evaluateAll((cards) => cards.map((card) => card.dataset.palette))).toEqual([
      "z00z-default",
      "z00z-corporate",
    ]);
    await expect(defaultPalette).toHaveAttribute("aria-pressed", "true");
    await expect(defaultPalette.getByText("Active", { exact: true })).toBeVisible();
    await expect(defaultPalette.locator(".palette-card-heading em")).toHaveCSS("text-transform", "uppercase");
    await expect(corporatePalette).toHaveAttribute("aria-pressed", "false");
    await expect(page.locator(".palette-card-heading em")).toHaveCount(1);
    await expect(page.locator(".palette-card small")).toHaveCount(0);
    await expect(page.locator("#main-content")).not.toContainText("Current dark Z00Z application palette");
    await expect(page.locator("#main-content")).not.toContainText("Light Corporate palette");
    await expect(page.locator("[data-palette-apply], [data-palette-cancel], [data-palette-reset], .palette-preview-status")).toHaveCount(0);

    await corporatePalette.click();
    await expect(page.locator("html")).toHaveAttribute("data-palette", "z00z-corporate");
    await expect(corporatePalette).toHaveAttribute("aria-pressed", "true");
    await expect(corporatePalette.getByText("Active", { exact: true })).toBeVisible();
    await expect(defaultPalette).toHaveAttribute("aria-pressed", "false");
    await expect(page.locator(".palette-card-heading em")).toHaveCount(1);

    await defaultPalette.click();
    await expect(page.locator("html")).toHaveAttribute("data-palette", "z00z-default");
    await expect(defaultPalette).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator("html")).not.toHaveAttribute("data-theme", /./);
    await expectNoViewportOverflow(page, `Appearance at ${viewport.width}px`);
  }
});

test("dApps roadmap stays local, navigable, bounded, and responsive", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    const walletAssetSnapshot = await page.locator(".asset-row").allInnerTexts();
    await page.goto(`${demoUrl}?route=wallet.history`);
    const walletHistoryCount = await page.locator(".activity-row").count();
    await page.goto(`${demoUrl}?route=dapps.discover`);

    await expect(page.locator(".dapp-roadmap")).toBeVisible();
    await expect(page.locator(".route-preview")).toHaveCount(0);
    await expect(page.locator("[data-dapp-card]")).toHaveCount(6);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await expect(page.locator(".navigation-tree-badge")).toHaveCount(0);
    await expect(page.locator(viewport.mobile ? ".mobile-nav-brand img" : ".desktop-topbar-brand .brand-mark")).toBeVisible();
    await expect(page.locator("#page-title")).toHaveText("Discover");
    await expect(page.locator("#route-breadcrumb")).toHaveText("dApps / Discover");

    const navigation = viewport.mobile
      ? page.locator("#mobile-popup-menu .mobile-navigation-tree")
      : page.locator("#app-navigation-tree");
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await expect(navigation.locator('[data-navigation-branch="dapps"]')).toHaveAttribute("aria-expanded", "true");
    const dappRoutes = navigation.locator('[data-navigation-branch="dapps"] + .navigation-tree-children > [data-navigation-route]');
    await expect(dappRoutes).toHaveCount(6);
    await expect(dappRoutes).toHaveText(["Discover", "Installed", "Connections", "Permissions", "Swap", "Exchange"]);
    await expect(navigation.locator('[data-navigation-branch="wallet"] + .navigation-tree-children > [data-navigation-route="wallet.swap"], [data-navigation-branch="wallet"] + .navigation-tree-children > [data-navigation-route="wallet.exchange"]')).toHaveCount(0);
    await expect(navigation.locator('[data-navigation-branch="dapps"] + .navigation-tree-children [data-navigation-branch]')).toHaveCount(0);
    await expect(navigation.locator('[data-navigation-route="dapps.discover"]')).toHaveAttribute("aria-current", "page");
    if (viewport.mobile) await page.keyboard.press("Escape");

    await page.locator('[data-dapp-card="external-asset-locker"] [data-dapp-action="open"]').click();
    await expect(page.locator('[data-dapp-detail="external-asset-locker"]')).toBeVisible();
    await expect(page.locator('[data-dapp-detail="external-asset-locker"]')).toContainText("Typed intent only · no wallet bridge");
    await expectNoViewportOverflow(page);
    await page.locator('[data-dapp-action="back"]').click();

    await page.goto(`${demoUrl}?route=dapps.connections`);
    await expect(page.locator("[data-dapp-connection]")).toHaveCount(3);
    await page.locator('[data-dapp-connection="connection_offline_pay"] [data-dapp-action="review"]').click();
    const review = page.locator('[data-dapp-review="connection_offline_pay"]');
    await expect(review).toBeVisible();
    await expect(review.locator(".dapp-review-grid > div")).toHaveCount(12);
    await expect(review).toContainText("Value");
    await expect(review).toContainText("Fee path");
    await expect(review).toContainText("Wallet object");
    await expect(review.locator('input[type="password"], [data-secure-entry]')).toHaveCount(0);
    await expectNoViewportOverflow(page);
    const acceptIntent = review.getByRole("button", { name: "Accept bounded intent" });
    await acceptIntent.click();
    await expect(review.locator("#dapp-review-error")).toContainText("Confirm the exact displayed scope");
    await review.locator('input[name="scopeConfirmed"]').check();
    await acceptIntent.click();
    await expect(review.locator("#dapp-review-error")).toContainText("Acknowledge that Wallet re-auth is required");
    await review.locator('input[name="reauthAcknowledged"]').check();
    await acceptIntent.click();
    await expect(page.locator('[data-dapp-outcome-route="intent_accepted"]')).toBeVisible();
    await expect(page.locator('[data-dapp-outcome-route="intent_accepted"]')).toContainText("Wallet state unchanged");
    await expect(page.locator('[data-dapp-action="activity"], [data-navigation-route="dapps.activity"]')).toHaveCount(0);

    await page.goto(`${demoUrl}?route=dapps.connections`);
    await page.locator('[data-dapp-connection="connection_offline_pay"] [data-dapp-action="review"]').click();
    const walletReview = page.locator('[data-dapp-review="connection_offline_pay"]');
    await walletReview.locator('input[name="scopeConfirmed"]').check();
    await walletReview.locator('input[name="reauthAcknowledged"]').check();
    await walletReview.getByRole("button", { name: "Accept bounded intent" }).click();
    await page.locator('[data-dapp-action="wallet-review"]').click();
    await expect(page.locator("#page-title")).toHaveText("Send");
    await expect(page.locator('[data-dapp-wallet-handoff]')).toContainText("Prepared from Offline Pay");
    await expect(page.locator("#send-recipient")).toHaveValue("");
    await expect(page.locator("#send-item")).toHaveValue("z00z");
    await expect(page.locator("#send-amount")).toHaveValue("24.00");
    await expect(page.locator("#send-panel-title")).toHaveText("Send privately");

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.assets", { mobile: viewport.mobile });
    expect(await page.locator(".asset-row").allInnerTexts()).toEqual(walletAssetSnapshot);

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.history", { mobile: viewport.mobile });
    await expect(page.locator(".activity-row")).toHaveCount(walletHistoryCount);

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.send", { mobile: viewport.mobile });
    await expect(page.locator('[data-dapp-wallet-handoff]')).toBeVisible();
    await expect(page.locator("#send-amount")).toHaveValue("24.00");
    await page.locator("#send-recipient").fill("z00z1wallet-review-demo");
    await page.locator('button[form="send-entry"]').click();
    await expect(page.locator("#send-panel-title")).toHaveText("Review send");
    await expect(page.locator(".review-hero")).toContainText("24.00 Z00Z");
    await expect(page.locator(".review-hero")).toContainText("z00z1wallet-review-demo");

    await page.goto(`${demoUrl}?route=dapps.permissions`);
    await expect(page.locator("[data-dapp-permission]")).toHaveCount(3);
    await expect(page.locator('[data-dapp-permission="permission_service_credits"]')).toContainText("Expired");
    await page.locator('[data-dapp-permission="permission_scoped_expenses"] [data-dapp-action="revoke"]').click();
    await expect(page.locator('[data-dapp-outcome-route="permission_revoked"]')).toBeVisible();
    await page.locator('[data-dapp-action="outcome-back"]').click();
    await expect(page.locator('[data-dapp-permission="permission_scoped_expenses"]')).toContainText("Revoked");

    await page.goto(`${demoUrl}?route=dapps.installed`);
    await expect(page.locator("[data-dapp-card]")).toHaveCount(3);
    await expect(page.locator(".notice")).toContainText("No third-party executable or remote service was loaded");
    await expectNoViewportOverflow(page);
  }
});

test("Messenger keeps advisory actions local and revalidates payment inside Wallet on desktop and mobile", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    const walletAssetSnapshot = await page.locator(".asset-row").allInnerTexts();
    await page.goto(`${demoUrl}?route=wallet.history`);
    const walletHistoryCount = await page.locator(".activity-row").count();

    await page.goto(`${demoUrl}?route=messenger.inbox`);
    await expect(page.locator(".messenger-roadmap")).toBeVisible();
    await expect(page.locator(".route-preview")).toHaveCount(0);
    await expect(page.locator("[data-messenger-message]")).toHaveCount(8);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await expect(page.locator(".messenger-relay-control")).toBeVisible();
    await expect(page.locator(viewport.mobile ? ".mobile-nav-brand img" : ".desktop-topbar-brand .brand-mark")).toBeVisible();

    const navigation = viewport.mobile
      ? page.locator("#mobile-popup-menu .mobile-navigation-tree")
      : page.locator("#app-navigation-tree");
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await expect(navigation.locator('[data-navigation-branch="messenger"] + .navigation-tree-children > [data-navigation-route^="messenger."]')).toHaveCount(3);
    await expect(navigation.locator('[data-navigation-branch="messenger"] + .navigation-tree-children > [data-navigation-route^="messenger."] .navigation-tree-label')).toHaveText(["Inbox", "Sent", "Conversations"]);
    await expect(navigation.locator('[data-navigation-branch="messenger"] + .navigation-tree-children [data-navigation-branch]')).toHaveCount(0);
    if (viewport.mobile) await page.keyboard.press("Escape");

    await page.goto(`${demoUrl}?route=messenger.sent`);
    await expect(page.locator("#page-title")).toHaveText("Sent");
    await expect(page.locator("[data-messenger-sent]")).toHaveCount(5);
    await page.goto(`${demoUrl}?route=messenger.conversations`);
    await expect(page.locator("[data-messenger-conversation]")).toHaveCount(2);
    await page.goto(`${demoUrl}?route=messenger.inbox`);

    await page.locator('[data-messenger-message="message_advisory_001"] [data-messenger-action="open"]').click();
    await expect(page.locator('[data-messenger-detail="message_advisory_001"]')).toContainText("Wallet object");
    await page.locator('[data-messenger-action="acknowledge"]').click();
    await expect(page.locator('[data-messenger-detail="message_advisory_001"]')).toContainText("Acknowledged");
    await page.locator('[data-messenger-action="delete"]').click();
    await expect(page.locator("[data-messenger-message]")).toHaveCount(7);

    await page.locator('[data-messenger-action="relay-unavailable"]').click();
    await expect(page.locator('[data-messenger-relay="unavailable"]')).toContainText("Relay unavailable");
    await page.locator('[data-messenger-action="relay-recover"]').click();
    await expect(page.locator('[data-messenger-relay="recovering"]')).toContainText("Recovery check is local-only");

    await page.goto(`${demoUrl}?route=messenger.inbox`);
    await expect(page.locator("[data-messenger-message]")).toHaveCount(8);
    await page.locator('[data-messenger-message="message_expired_001"] [data-messenger-action="open"]').click();
    await page.locator('[data-messenger-action="review"]').click();
    await expect(page.locator('[data-messenger-review="review_message_expired_001"]')).toContainText("Expired");
    await expect(page.locator('[data-messenger-action="accept-request"]')).toBeDisabled();
    await page.locator('[data-messenger-action="detail"]').click();
    await page.locator('[data-messenger-action="back"]').click();

    await page.locator('[data-messenger-message="message_payment_001"] [data-messenger-action="open"]').click();
    await page.locator('[data-messenger-action="review"]').click();
    await expect(page.locator('[data-messenger-review="review_message_payment_001"]')).toContainText("18.50 Z00Z");
    await page.locator('[data-messenger-action="accept-request"]').click();
    await expect(page.locator('[data-messenger-outcome="accepted"]')).toContainText("Wallet state unchanged");
    await page.locator('[data-messenger-action="wallet-review"]').click();
    await expect(page.locator("#page-title")).toHaveText("Send");
    await expect(page.locator("[data-messenger-wallet-handoff]")).toContainText("Revalidated Messenger payment request");
    await expect(page.locator("#send-recipient")).toHaveValue("");
    await expect(page.locator("#send-item")).toHaveValue("z00z");
    await expect(page.locator("#send-amount")).toHaveValue("18.50");
    await expectNoViewportOverflow(page);

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.assets", { mobile: viewport.mobile });
    expect(await page.locator(".asset-row").allInnerTexts()).toEqual(walletAssetSnapshot);
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.history", { mobile: viewport.mobile });
    await expect(page.locator(".activity-row")).toHaveCount(walletHistoryCount);
  }
});

test("Contacts stays wallet-local across search, import, identity review, actions, and removal", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    const walletAssetSnapshot = await page.locator(".asset-row").allInnerTexts();
    await page.goto(`${demoUrl}?route=wallet.history`);
    const walletHistoryCount = await page.locator(".activity-row").count();
    await page.goto(`${demoUrl}?route=contacts.list`);

    await expect(page.locator(".contacts-roadmap")).toBeVisible();
    await expect(page.locator("#page-title")).toHaveText("Contacts");
    await expect(page.locator(".contacts-roadmap")).toContainText("Address book");
    await expect(page.locator("[data-contact]")).toHaveCount(6);
    await expect(page.locator(".contact-book-row")).toHaveCount(6);
    await expect(page.locator(".contact-card")).toHaveCount(0);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await page.locator("[data-contact-status-filter]").selectOption("identity_changed");
    await expect(page.locator("[data-contact]")).toHaveCount(1);
    await page.locator("[data-contact-status-filter]").selectOption("all");
    await page.locator("#contacts-query").fill("voucher");
    await page.locator("#contacts-search-form").evaluate((form) => form.requestSubmit());
    await expect(page.locator("[data-contact]")).toHaveCount(1);
    await page.locator("#contacts-query").fill("");
    await page.locator("#contacts-search-form").evaluate((form) => form.requestSubmit());
    await page.locator("[data-contact-sort]").selectOption("nickname");
    expect(await page.locator("[data-contact]").evaluateAll((cards) => cards.map((card) => card.dataset.contact))).toEqual([
      "contact_ada",
      "contact_ben",
      "contact_community",
      "contact_old_service",
      "contact_ops",
      "contact_revoked",
    ]);
    await page.locator("[data-contact-sort]").selectOption("date");
    expect(await page.locator("[data-contact]").evaluateAll((cards) => cards.map((card) => card.dataset.contact))).toEqual([
      "contact_ada",
      "contact_ben",
      "contact_community",
      "contact_ops",
      "contact_old_service",
      "contact_revoked",
    ]);

    await page.locator('[data-contact="contact_ops"] [data-contact-action="open"]').click();
    await expect(page.locator('[data-contact-detail="contact_ops"]')).toContainText("Identity Changed");
    await page.locator('[data-contact-action="identity-review"]').click();
    await expect(page.locator('[data-contact-identity-review="contact_ops"]')).toContainText("None; local confirmation only");
    await page.locator('[data-contact-action="identity-accept"]').click();
    await expect(page.locator('[data-contact-outcome="identity_accepted"]')).toContainText("no public trust claim");
    await page.locator('[data-contact-action="back"]').click();

    await page.locator('[data-contact-action="add"]').click();
    await page.locator('[data-source-id="qr_scan"]').click();
    await expect(page.locator('[data-contact-import="qr_scan"]')).toContainText("Native boundary unavailable");
    await page.locator('[data-source-id="manual"]').click();
    await page.locator("#contact-import-label").fill("Local test");
    await page.locator("#contact-import-note").fill("Browser-free local record");
    await page.locator("#contact-import-form").evaluate((form) => form.requestSubmit());
    await expect(page.locator('[data-contact-outcome="added"]')).toContainText("without a contact upload");
    await page.locator('[data-contact-action="back"]').click();
    await expect(page.locator("[data-contact]")).toHaveCount(7);
    await page.locator('[data-contact="contact_local_0007"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="edit"]').click();
    await page.locator("#contact-edit-label").fill("Local revised");
    await page.locator("#contact-edit-form").evaluate((form) => form.requestSubmit());
    await expect(page.locator('[data-contact-outcome="edited"]')).toContainText("remote state remained unchanged");
    await page.locator('[data-contact-action="back"]').click();
    await page.locator('[data-contact="contact_local_0007"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="remove"]').click();
    await expect(page.locator('[data-contact-outcome="removed"]')).toContainText("not revoked or erased");

    await page.goto(`${demoUrl}?route=contacts.list`);
    await page.locator('[data-contact="contact_ada"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="pay"]').click();
    await expect(page.locator("#page-title")).toHaveText("Send");
    await expect(page.locator("[data-contact-wallet-handoff]")).toContainText("Revalidated Pay action for Ada");
    await expect(page.locator("#send-recipient")).toHaveValue("");
    await expectNoViewportOverflow(page);

    await page.goto(`${demoUrl}?route=contacts.list`);
    await page.locator('[data-contact="contact_community"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="message"]').click();
    await expect(page.locator("#page-title")).toHaveText("Conversations");
    await expect(page.locator("[data-contact-messenger-handoff]")).toContainText("Prepared for Community desk");
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await expect(page.locator("[data-contact-messenger-handoff]")).toContainText("must revalidate");
    await expectNoViewportOverflow(page);

    await page.goto(`${demoUrl}?route=wallet.assets`);
    expect(await page.locator(".asset-row").allInnerTexts()).toEqual(walletAssetSnapshot);
    await page.goto(`${demoUrl}?route=wallet.history`);
    await expect(page.locator(".activity-row")).toHaveCount(walletHistoryCount);
  }
});

test("desktop Help is a separate application and keeps the selected palette", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=settings.appearance`);
  await page.locator('[data-palette="z00z-corporate"]').click();

  const helpPage = await openStandaloneHelp(page, page.locator("#app-navigation-terminal [data-help-topic]"));
  await expect(helpPage.locator("#help-document")).toBeVisible();
  await expect(helpPage.locator("html")).toHaveAttribute("data-palette", "z00z-corporate");
  await expect(helpPage.locator(".help-sidebar")).toBeVisible();
  await helpPage.close();
  await expect(page.locator("#main-content")).toBeVisible();
});

test("version, destructive Log out, Data & Storage, Notifications, and About work on desktop and mobile", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 390, height: 844, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    const terminal = viewport.mobile
      ? page.locator(".mobile-navigation-terminal")
      : page.locator("#app-navigation-terminal");
    if (viewport.mobile) await terminal.scrollIntoViewIfNeeded();
    const logout = terminal.locator('[data-demo-action="logout"]');
    const terminalSettings = terminal.locator(':scope > .navigation-tree-branch [data-navigation-branch="settings"]');
    await expect(terminalSettings).toContainText("Settings");
    await expect(terminal.locator(":scope > .navigation-tree-terminal")).toHaveText(["Help", "About", "Log out"]);
    await expect(terminal.locator('[data-navigation-route="about"]')).toBeVisible();
    const primaryNavigation = viewport.mobile
      ? page.locator(".mobile-navigation-tree")
      : page.locator("#app-navigation-tree");
    const primaryOrder = await primaryNavigation.evaluate((navigation) => [...navigation.children].map((child) => {
      const target = child.matches("[data-navigation-branch], [data-navigation-route]")
        ? child
        : child.querySelector(":scope > [data-navigation-branch], :scope > [data-navigation-route]");
      return target?.dataset.navigationBranch || target?.dataset.navigationRoute || "";
    }));
    expect(primaryOrder.indexOf("data-storage")).toBeLessThan(primaryOrder.indexOf("contacts.list"));
    await expect(terminalSettings).toHaveAttribute("aria-expanded", "false");
    await terminalSettings.click();
    await expect(terminalSettings).toHaveAttribute("aria-expanded", "true");
    await expect(terminal.locator(':scope > .navigation-tree-branch > .navigation-tree-children [data-navigation-route^="settings."]')).toHaveCount(3);
    await expect(terminal.locator(".app-version")).toHaveText("Version 0.1.0");
    expect(await logout.evaluate((element) => getComputedStyle(element).color)).toBe(
      await page.locator(".nav-item-danger").evaluate((element) => getComputedStyle(element).color),
    );
    expect(await logout.evaluate((element) => element.nextElementSibling?.classList.contains("app-version"))).toBe(true);

    await terminal.locator('[data-navigation-route="about"]').click();
    await expect(page).toHaveURL(/route=about/);
    if (viewport.mobile) await expect(page.locator("#mobile-popup-menu")).toBeHidden();
    await expect(page.locator("#page-title")).toHaveText("About");
    const about = page.locator(".about-surface");
    await expect(about).toContainText("Z00Z Wallet v0.1.0");
    await expect(about.locator("a")).toHaveText([
      "Privacy Policy",
      "Terms of Use",
      "Visit Z00Z Website",
      "Visit Z00Z GitHub repository",
    ]);
    await expect(about.locator('a[href="https://z00z.io/docs/legal/privacy"]')).toHaveAttribute("target", "_blank");
    await expect(about.locator('a[href="https://z00z.io/docs/legal/terms"]')).toHaveAttribute("rel", "noopener noreferrer");
    await expect(about.locator('a[href="https://z00z.io/"]')).toBeVisible();
    await expect(about.locator('a[href="https://github.com/z00z-labs/z00z"]')).toBeVisible();
    await expect(about.locator(":scope > :last-child")).toHaveAttribute("data-demo-action", "check-for-updates");
    await expect(page.locator(".about-card, .about-metadata")).toHaveCount(0);
    await page.locator('[data-demo-action="check-for-updates"]').click();
    await expect(page.locator(".update-check-status")).toContainText("current demo version 0.1.0");

    await page.goto(`${demoUrl}?route=data-storage.disk-usage`);
    await expect(page.locator(".data-storage-view")).toContainText("Disk Usage");
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "data-storage.network-usage", { mobile: viewport.mobile });
    await expect(page.locator(".data-storage-view")).toContainText("Network Usage");

    await page.goto(`${demoUrl}?route=settings.notifications`);
    await expect(page.locator('[data-config-control="vibrate"]')).toHaveValue("messages-and-alerts");
    await expect(page.locator('[data-config-control="ringtone"]')).toHaveValue("z00z-pulse");
    await page.locator('[data-config-control="vibrate"]').selectOption("alerts-only");
    await page.locator('[data-config-control="ringtone"]').selectOption("soft-chime");
    await expect(page.locator('[data-config-control="vibrate"]')).toHaveValue("alerts-only");
    await expect(page.locator('[data-config-control="ringtone"]')).toHaveValue("soft-chime");

    await page.goto(`${demoUrl}?route=settings.general`);
    await expect(page.locator('[data-config-control="language"] option').first()).toContainText("🇬🇧");
    await expectNoViewportOverflow(page);
  }
});

test("Help reuses App section icons, language flags, and searchable localized content", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const helpPage = await openStandaloneHelp(page, page.locator("#app-navigation-terminal [data-help-topic]"));

  await expect(helpPage.locator('[data-help-group="wallets"] use').first()).toHaveAttribute("href", "#i-wallet");
  await expect(helpPage.locator('[data-help-group="telemetry"] use').first()).toHaveAttribute("href", "#i-network");
  await expect(helpPage.locator('[data-help-group="data-storage"] use').first()).toHaveAttribute("href", "#i-storage");
  await expect(helpPage.locator(".help-language-icon")).toBeVisible();
  await expect(helpPage.locator(".help-language-icon use")).toHaveAttribute("href", "#i-language");
  await expect(helpPage.locator("#i-language")).toHaveAttribute("data-icon-source", "material-symbols-light:language");
  await expect(helpPage.locator("#help-language-label")).toHaveClass(/visually-hidden/);
  await expect(helpPage.locator("#help-language")).toHaveAttribute("aria-label", "Language");
  await expect(helpPage.locator("#help-language option").first()).toContainText("🇬🇧");
  await helpPage.locator("#help-search").fill("signed release manifest");
  await expect(helpPage.locator('[data-help-search-topic="about"]')).toBeVisible();
  await helpPage.locator('[data-help-search-topic="about"]').click();
  await expect(helpPage.locator("#help-title")).toContainText("About Z00Z");

  await helpPage.setViewportSize({ width: 390, height: 844 });
  await helpPage.locator("#help-menu-button").click();
  await helpPage.locator("#help-search").fill("public evidence");
  await expect(helpPage.locator(".help-search-result").first()).toBeVisible();
  await expectNoViewportOverflow(helpPage);
  await helpPage.close();
});

test("all root Help articles are selectable on desktop and mobile", async ({ page }) => {
  const articles = [
    ["about", "About Z00Z"],
    ["help.faq", "Frequently asked questions"],
    ["help.how-to", "How to use the demo"],
    ["help.report-issues", "Report issues"],
    ["help.tips-and-tricks", "Tips and tricks"],
    ["help.video-tutorials", "Video tutorials"],
  ];

  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 390, height: 844, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    const trigger = viewport.mobile
      ? page.locator(".mobile-navigation-terminal [data-help-topic]")
      : page.locator("#app-navigation-terminal [data-help-topic]");
    if (viewport.mobile) await trigger.scrollIntoViewIfNeeded();
    const helpPage = await openStandaloneHelp(page, trigger);
    await helpPage.setViewportSize(viewport);

    for (const [topicId, title] of articles) {
      if (
        viewport.mobile
        && !await helpPage.locator("#help-sidebar").evaluate((sidebar) => sidebar.classList.contains("is-open"))
      ) {
        await helpPage.locator("#help-menu-button").click();
      }
      const appGroup = helpPage.locator('[data-help-group="app"]');
      if (await appGroup.getAttribute("aria-expanded") !== "true") await appGroup.click();
      const articleLink = helpPage.locator(`[data-help-topic-link="${topicId}"]`);
      await expect(articleLink).toBeVisible();
      await articleLink.click();
      await expect(helpPage.locator("#help-title")).toHaveText(title);
      await expect(helpPage).toHaveURL(new RegExp(`topic=${topicId.replaceAll(".", "\\.")}`));
      await expect(helpPage.locator("#help-context-tabs")).toBeHidden();
      const titlePosition = await helpPage.evaluate(() => ({
        headerTop: document.querySelector(".help-site-header").getBoundingClientRect().top,
        headerBottom: document.querySelector(".help-site-header").getBoundingClientRect().bottom,
        titleTop: document.querySelector("#help-title").getBoundingClientRect().top,
      }));
      expect(Math.abs(titlePosition.headerTop)).toBeLessThanOrEqual(1);
      expect(titlePosition.titleTop).toBeGreaterThanOrEqual(titlePosition.headerBottom);
    }
    await expectNoViewportOverflow(helpPage);
    await helpPage.close();
  }
});

test("context navigation stays vertical on desktop and moves into the mobile topbar beside a persistent wallet identity", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const desktopRail = page.locator(".wallet-assets-layout > .context-rail");
  const desktopPanel = page.locator(".wallet-assets-layout > .workspace-panel");
  await expect(desktopRail).toBeVisible();
  await expect(desktopRail.locator("[data-wallet-section]")).toHaveCount(3);
  const desktopGeometry = await Promise.all([
    desktopRail.boundingBox(),
    desktopPanel.boundingBox(),
  ]);
  expect(desktopGeometry[0].x + desktopGeometry[0].width).toBeLessThanOrEqual(desktopGeometry[1].x);

  await page.setViewportSize({ width: 320, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const mobileRail = page.locator(".wallet-assets-layout > .context-rail");
  const mobileTopbarContext = page.locator("#mobile-topbar-context");
  const mobileTabs = mobileTopbarContext.locator("[data-wallet-section]");
  const mobileWalletIdentity = page.locator("#mobile-active-wallet");
  await expect(mobileRail).toBeHidden();
  await expect(mobileTopbarContext).toBeVisible();
  await expect(mobileTabs).toHaveCount(3);
  await expect(mobileWalletIdentity).toBeVisible();
  await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
  await expect(mobileWalletIdentity).toContainText("Everyday wallet");
  const mobileGeometry = await page.evaluate(() => {
    const topbar = document.querySelector(".topbar").getBoundingClientRect();
    const walletIdentity = document.querySelector("#mobile-active-wallet").getBoundingClientRect();
    const tabs = [...document.querySelectorAll("#mobile-topbar-context [data-wallet-section]")]
      .map((tab) => tab.getBoundingClientRect());
    return {
      topbarTop: topbar.top,
      topbarBottom: topbar.bottom,
      walletIdentityTop: walletIdentity.top,
      tabBounds: tabs.map(({ top, bottom }) => ({ top, bottom })),
    };
  });
  expect(Math.abs(mobileGeometry.walletIdentityTop - mobileGeometry.topbarBottom)).toBeLessThanOrEqual(1);
  for (const tab of mobileGeometry.tabBounds) {
    expect(tab.top).toBeGreaterThanOrEqual(mobileGeometry.topbarTop);
    expect(tab.bottom).toBeLessThanOrEqual(mobileGeometry.topbarBottom);
  }

  await mobileTabs.filter({ hasText: "Vouchers" }).click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  await expect(mobileTopbarContext.locator('[data-wallet-section="vouchers"]')).toHaveAttribute("aria-current", "page");
  await expect(page.locator('#app-navigation-tree [data-navigation-route="wallet.vouchers"]')).toHaveCount(0);
  await expectNoViewportOverflow(page);

  await page.goto(`${demoUrl}?route=wallet.send`);
  await expect(page.locator(".send-workspace-layout > .context-rail")).toBeHidden();
  await expect(mobileTopbarContext.locator("[data-send-family]")).toHaveCount(3);

  await page.goto(`${demoUrl}?route=telemetry.reticulum.overview`);
  const telemetryRail = page.locator(".telemetry-workspace-layout > .context-rail");
  await expect(telemetryRail).toBeHidden();
  await expect(mobileTopbarContext.locator("[data-workspace-route]")).toHaveCount(8);
  await mobileTopbarContext.locator('[data-workspace-route="telemetry.reticulum.node"]').click();
  await expect(page.locator("#page-title")).toHaveText("Node");
  await expect(page.locator("#mobile-popup-menu")).toBeHidden();

  await page.goto(`${demoUrl}?route=contacts.list`);
  await expect(mobileTopbarContext).toBeHidden();
  await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
  await page.locator("#mobile-menu-button").click();
  await page.locator('[data-mobile-wallet-id="savings"]').click();
  await expect(mobileWalletIdentity).toContainText("ZxR5vK…8Ee1Qm");
  await expect(mobileWalletIdentity).toContainText("Savings wallet");

  for (const routeId of [
    "telemetry.reticulum.overview",
    "dapps.discover",
    "messenger.inbox",
    "contacts.list",
    "data-storage.disk-usage",
    "settings.general",
    "about",
  ]) {
    await page.goto(`${demoUrl}?route=${routeId}`);
    await expect(mobileWalletIdentity).toBeVisible();
    await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
  }
  await expectNoViewportOverflow(page);
});

test("all Telemetry components keep deeper routes inside desktop rails and mobile tabs", async ({ page }) => {
  const workspaces = [
    {
      id: "telemetry.reticulum",
      route: "telemetry.reticulum.overview",
      childRoute: "telemetry.reticulum.links",
      childTitle: "Links",
      localCount: 8,
    },
    {
      id: "telemetry.onionnet",
      route: "telemetry.onionnet.overview",
      childRoute: "telemetry.onionnet.queues",
      childTitle: "Queues",
      localCount: 7,
    },
    {
      id: "telemetry.aggregators",
      route: "telemetry.aggregators.overview",
      childRoute: "telemetry.aggregators.publication",
      childTitle: "Publication",
      localCount: 6,
    },
    {
      id: "telemetry.watchers",
      route: "telemetry.watchers.overview",
      childRoute: "telemetry.watchers.evidence",
      childTitle: "Evidence export",
      localCount: 6,
    },
    {
      id: "telemetry.explorer",
      route: "telemetry.explorer.overview",
      childRoute: "telemetry.explorer.evidence",
      childTitle: "Public evidence",
      localCount: 5,
    },
  ];

  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    for (const workspace of workspaces) {
      await page.goto(`${demoUrl}?route=${workspace.route}`);
      const rail = await visibleContextNavigation(page, ".telemetry-workspace-layout > .context-rail");
      await expect(rail).toBeVisible();
      await expect(rail.locator("[data-workspace-route]")).toHaveCount(workspace.localCount);
      await rail.locator(`[data-workspace-route="${workspace.childRoute}"]`).click();
      await expect(page.locator("#page-title")).toHaveText(workspace.childTitle);
      await expect(rail.locator(`[data-workspace-route="${workspace.childRoute}"]`)).toHaveAttribute("aria-current", "page");
      if (["telemetry.aggregators", "telemetry.watchers", "telemetry.explorer"].includes(workspace.id)) {
        if (workspace.id === "telemetry.watchers") {
          await expect(page.locator('[data-watcher-screen="evidence"]')).toBeVisible();
        } else if (workspace.id === "telemetry.explorer") {
          await expect(page.locator('[data-explorer-screen="evidence"]')).toBeVisible();
        } else {
          await expect(page.locator('[data-aggregator-screen="publication"]')).toBeVisible();
        }
        await expect(page.locator(".route-preview")).toHaveCount(0);
      }

      if (viewport.mobile) {
        const tabTops = await rail.locator("[data-workspace-route]").evaluateAll((tabs) => (
          tabs.map((tab) => Math.round(tab.getBoundingClientRect().top))
        ));
        expect(new Set(tabTops).size).toBe(1);
        await page.locator("#mobile-menu-button").click();
        const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
        await expect(drawer.locator(`[data-navigation-workspace="${workspace.id}"]`)).toHaveAttribute("aria-current", "page");
        await expect(drawer.locator(`[data-navigation-route="${workspace.childRoute}"]`)).toHaveCount(0);
        await page.keyboard.press("Escape");
      } else {
        const [railBox, panelBox] = await Promise.all([
          rail.boundingBox(),
          page.locator(".telemetry-workspace-layout > .workspace-panel").boundingBox(),
        ]);
        expect(railBox.x + railBox.width).toBeLessThanOrEqual(panelBox.x);
        await expect(page.locator(`#app-navigation-tree [data-navigation-workspace="${workspace.id}"]`)).toHaveAttribute("aria-current", "page");
        await expect(page.locator(`#app-navigation-tree [data-navigation-route="${workspace.childRoute}"]`)).toHaveCount(0);
      }
      await expectNoViewportOverflow(page);
    }
  }
});

test("every canonical workspace projects deeper routes only inside the main window", async ({ page }) => {
  const workspaces = [
    ["wallet.assets-rights", "wallet.assets", "wallet.permissions", 3],
    ["wallet.staking", "wallet.staking.stake", "wallet.staking.unstake", 2],
    ["wallet.settings", "wallet.settings.general", "wallet.settings.advanced", 5],
    ["telemetry.reticulum", "telemetry.reticulum.overview", "telemetry.reticulum.links", 8],
    ["telemetry.onionnet", "telemetry.onionnet.overview", "telemetry.onionnet.ingress", 7],
    ["telemetry.aggregators", "telemetry.aggregators.overview", "telemetry.aggregators.recovery", 6],
    ["telemetry.watchers", "telemetry.watchers.overview", "telemetry.watchers.evidence", 6],
    ["telemetry.explorer", "telemetry.explorer.overview", "telemetry.explorer.evidence", 5],
  ];

  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    for (const [workspaceId, defaultRoute, childRoute, localCount] of workspaces) {
      await page.goto(`${demoUrl}?route=${defaultRoute}`);
      const localNavigation = await visibleContextNavigation(page);
      await expect(localNavigation).toBeVisible();
      await expect(localNavigation.locator("button")).toHaveCount(localCount);

      if (viewport.mobile) {
        await page.locator("#mobile-menu-button").click();
        const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
        await expect(drawer.locator(`[data-navigation-workspace="${workspaceId}"]`)).toHaveCount(1);
        await expect(drawer.locator(`[data-navigation-route="${childRoute}"]`)).toHaveCount(0);
        await page.keyboard.press("Escape");
        const topCoordinates = await localNavigation.locator("button").evaluateAll((buttons) => (
          buttons.map((button) => Math.round(button.getBoundingClientRect().top))
        ));
        expect(new Set(topCoordinates).size).toBe(1);
      } else {
        await expect(page.locator(`#app-navigation-tree [data-navigation-workspace="${workspaceId}"]`)).toHaveCount(1);
        await expect(page.locator(`#app-navigation-tree [data-navigation-route="${childRoute}"]`)).toHaveCount(0);
        const [railBox, panelBox] = await Promise.all([
          localNavigation.boundingBox(),
          page.locator(".workspace-layout > .workspace-panel, .wallet-settings-view .settings-detail").first().boundingBox(),
        ]);
        expect(railBox.x + railBox.width).toBeLessThanOrEqual(panelBox.x);
      }
      await expectNoViewportOverflow(page);
    }
  }
});

test("Watchers roadmap completes typed alert to sanitized evidence across desktop and mobile states", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=telemetry.watchers.alerts`);

    const screen = page.locator('[data-watcher-screen="alerts"]');
    await expect(screen).toBeVisible();
    await expect(page.locator(".route-preview")).toHaveCount(0);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    await expect(screen.locator("[data-watcher-alert]")).toHaveCount(3);

    await screen.locator('[data-watcher-alert="watcher-alert-002"]').click();
    const alertDetail = page.locator(".watcher-alert-detail");
    await expect(alertDetail).toContainText("MissingBlob");
    await expect(alertDetail).toContainText("published_batch · batch_a13d9e22");
    await expect(alertDetail).toContainText("2026-07-26T12:00:00.000Z");
    await expect(alertDetail).toContainText("watchers::da_health");
    await expect(alertDetail).toContainText("da_ref_72be91");

    const explorerLink = alertDetail.locator('[data-watcher-action="open-explorer"]');
    await expect(explorerLink).toHaveAttribute("data-public-id", "da_ref_72be91");
    await explorerLink.click();
    await expect(page.locator("#page-title")).toHaveText("Public evidence");
    await expect(page.locator('[data-explorer-screen="evidence"]')).toBeVisible();
    await expect(page.locator('[data-explorer-detail="da_ref_72be91"]')).toContainText("Opaque provider ref");
    await expect(page.locator(".route-preview")).toHaveCount(0);
    await page.goBack();
    await expect(page.locator("#page-title")).toHaveText("Alerts");
    await expect(page.locator('[data-watcher-alert="watcher-alert-002"]')).toHaveAttribute("aria-current", "true");

    await alertDetail.locator('[data-watcher-action="inspect-evidence"]').click();
    await expect(page.locator("#page-title")).toHaveText("Evidence export");
    const evidenceScreen = page.locator('[data-watcher-screen="evidence"]');
    await expect(evidenceScreen).toBeVisible();
    await expect(evidenceScreen.locator(".watcher-evidence-card.is-selected")).toContainText("MissingBlob");
    await evidenceScreen.locator('[data-watcher-action="export-evidence"][data-alert-id="watcher-alert-002"]').click();
    const exportResult = evidenceScreen.locator(".watcher-export-result");
    await expect(exportResult).toContainText("watcher-evidence-export-v1");
    await expect(exportResult).toContainText("batch_a13d9e22");
    await expect(exportResult).toContainText("private addressing and communication fields excluded");
    for (const privateValue of ["Everyday", "Savings", "Travel", "z00z1native-boundary"]) {
      await expect(page.locator("#main-content")).not.toContainText(privateValue);
    }

    await page.goto(`${demoUrl}?route=telemetry.watchers.alerts`);
    await page.locator('[data-watcher-control="severity"]').selectOption("critical");
    await expect(page.locator("[data-watcher-alert]")).toHaveCount(1);
    await expect(page.locator("[data-watcher-alert]")).toContainText("MissingBlob");
    await page.locator('[data-watcher-control="source"]').selectOption("evidence_archive");
    await expect(page.locator('[data-watcher-control="source"]')).toHaveValue("evidence_archive");

    for (const scenario of ["loading", "degraded", "empty", "malformed", "error", "unavailable"]) {
      await page.locator('[data-watcher-control="scenario"]').selectOption(scenario);
      await expect(page.locator(".watcher-roadmap")).toHaveAttribute("data-watcher-result", scenario);
      if (scenario === "degraded") {
        await expect(page.locator(".watcher-state-notice")).toBeVisible();
        await expect(page.locator("[data-watcher-alert]")).toHaveCount(1);
      } else {
        await expect(page.locator(`[data-watcher-state="${scenario}"]`)).toBeVisible();
      }
    }
    await page.locator('[data-watcher-action="recover"]').click();
    await expect(page.locator(".watcher-roadmap")).toHaveAttribute("data-watcher-result", "success");

    const localTabs = (await visibleContextNavigation(page, ".telemetry-workspace-layout > .context-rail")).locator("[data-workspace-route]");
    await expect(localTabs).toHaveCount(6);
    if (viewport.width === 320) {
      const tabTops = await localTabs.evaluateAll((tabs) => tabs.map((tab) => Math.round(tab.getBoundingClientRect().top)));
      expect(new Set(tabTops).size).toBe(1);
    }
    await expectNoViewportOverflow(page);
  }
});

test("Explorer roadmap accepts only public typed IDs and keeps detail inside desktop and mobile workspaces", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=telemetry.explorer.search`);

    const screen = page.locator('[data-explorer-screen="search"]');
    await expect(screen).toBeVisible();
    await expect(page.locator(".route-preview")).toHaveCount(0);
    await expect(page.locator(".capability-boundary")).toHaveCount(0);

    const submitSearch = async (publicId) => {
      await page.locator("#explorer-public-id").fill(publicId);
      await page.locator("#explorer-public-search").evaluate((form) => form.requestSubmit());
    };
    for (const [publicId, kind] of [
      ["checkpoint_000184", "checkpoint"],
      ["batch_4f91c7a0", "batch"],
      ["publication_6f840184", "publication"],
      ["proof_92840184", "proof"],
      ["da_ref_72be91", "da reference"],
    ]) {
      await submitSearch(publicId);
      const detail = page.locator(`[data-explorer-detail="${publicId}"]`);
      await expect(detail).toBeVisible();
      await expect(detail).toContainText(kind);
      await expect(detail).toContainText(publicId);
    }

    await page.locator('[data-explorer-action="technical"]').click();
    await expect(page.locator(".explorer-technical-json")).toContainText("CheckpointDaReferenceV1");
    await page.locator('[data-explorer-action="summary"]').click();

    for (const [query, status] of [
      ["receiver_secret_001", "private"],
      ["checkpoint_18", "malformed"],
      ["tx_deadbeef", "unsupported"],
      ["checkpoint_999999", "unknown"],
      ["checkpoint_000183", "stale"],
    ]) {
      await submitSearch(query);
      await expect(page.locator(`[data-explorer-search-status="${status}"]`)).toBeVisible();
      await expect(page.locator("#explorer-public-id")).toHaveValue("");
      await expect(page.locator("#main-content")).not.toContainText(query);
    }

    const contextRail = await visibleContextNavigation(page, ".telemetry-workspace-layout > .context-rail");
    const localTabs = contextRail.locator("[data-workspace-route]");
    await expect(localTabs).toHaveCount(5);
    await contextRail.locator('[data-workspace-route="telemetry.explorer.checkpoints"]').click();
    await expect(page.locator('[data-explorer-screen="checkpoints"] [data-explorer-record]')).toHaveCount(3);
    await page.locator('[data-explorer-record="checkpoint_000184"]').click();
    const checkpointDetail = page.locator('[data-explorer-detail="checkpoint_000184"]');
    await expect(checkpointDetail).toContainText("finalized");
    await expect(checkpointDetail).toContainText("root_84f2d18a");
    await checkpointDetail.locator('[data-explorer-open-id="publication_6f840184"]').click();
    await expect(page.locator("#page-title")).toHaveText("Public evidence");
    await expect(page.locator('[data-explorer-detail="publication_6f840184"]')).toContainText("Route generation");

    await page.locator('[data-explorer-control="kind"]').selectOption("proof");
    await expect(page.locator("[data-explorer-record]")).toHaveCount(3);
    await expect(page.locator("[data-explorer-record]").first()).toContainText("Proof envelope");

    await contextRail.locator('[data-workspace-route="telemetry.explorer.checkpoints"]').click();
    for (const scenario of ["loading", "degraded", "empty", "malformed", "error", "unavailable"]) {
      await page.locator('[data-explorer-control="scenario"]').selectOption(scenario);
      await expect(page.locator(".explorer-roadmap")).toHaveAttribute("data-explorer-result", scenario);
      if (scenario === "degraded") {
        await expect(page.locator(".watcher-state-notice")).toBeVisible();
        await expect(page.locator("[data-explorer-record]")).toHaveCount(1);
      } else {
        await expect(page.locator(`[data-explorer-state="${scenario}"]`)).toBeVisible();
      }
    }
    await page.locator('[data-explorer-action="recover"]').click();
    await expect(page.locator(".explorer-roadmap")).toHaveAttribute("data-explorer-result", "success");

    if (viewport.width === 320) {
      const tabTops = await localTabs.evaluateAll((tabs) => tabs.map((tab) => Math.round(tab.getBoundingClientRect().top)));
      expect(new Set(tabTops).size).toBe(1);
      await page.locator("#mobile-menu-button").click();
      const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
      await expect(drawer.locator('[data-navigation-workspace="telemetry.explorer"]')).toHaveAttribute("aria-current", "page");
      await expect(drawer.locator('[data-navigation-route="telemetry.explorer.search"]')).toHaveCount(0);
      await page.keyboard.press("Escape");
    }
    await expectNoViewportOverflow(page);
  }
});

test("Telemetry renderer redacts wallet, receiver, memo, path, inbox, and secret canaries on every route", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.send`);
    await page.locator("#send-recipient").fill("z00z1telemetry-private-receiver");
    await page.locator("#send-memo").fill("telemetry-private-memo-canary");

    const { telemetryRoutes, fixtureCanaries } = await page.evaluate(() => ({
      telemetryRoutes: [...window.Z00ZDemo.PORT_CONTRACT.telemetryRoutes],
      fixtureCanaries: [...new Set(window.Z00ZDemo.INITIAL_WALLET_FIXTURES.flatMap((wallet) => [
        wallet.name,
        wallet.address,
        wallet.fullAddress,
        ...wallet.activities.flatMap((activity) => Object.values(activity.titleValues || {})),
      ]))].filter(Boolean),
    }));
    const privateCanaries = [
      ...fixtureCanaries,
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
      "arbitrary_filesystem_path",
    ].map((value) => value.toLowerCase());

    for (const routeId of telemetryRoutes) {
      await page.evaluate((route) => {
        const url = new URL(window.location.href);
        url.searchParams.set("route", route);
        window.history.pushState({ z00zRoute: route }, "", url);
        window.dispatchEvent(new PopStateEvent("popstate", { state: { z00zRoute: route } }));
      }, routeId);
      await page.waitForFunction((route) => (
        [...document.querySelectorAll("[data-workspace-route][aria-current='page']")]
          .some((item) => item.dataset.workspaceRoute === route)
      ), routeId);
      const rendered = await page.locator("#main-content").evaluate((main) => (
        `${main.innerText}\n${main.innerHTML}`.toLowerCase()
      ));
      for (const canary of privateCanaries) {
        expect(rendered.includes(canary), `${routeId} rendered private canary: ${canary}`).toBe(false);
      }
    }

    await page.evaluate(() => {
      const route = "wallet.send";
      const url = new URL(window.location.href);
      url.searchParams.set("route", route);
      window.history.pushState({ z00zRoute: route }, "", url);
      window.dispatchEvent(new PopStateEvent("popstate", { state: { z00zRoute: route } }));
    });
    await expect(page.locator("#send-recipient")).toHaveValue("z00z1telemetry-private-receiver");
    await expect(page.locator("#send-memo")).toHaveValue("telemetry-private-memo-canary");
  }
});

test("Aggregator concept screens keep runtime contracts without redundant boundary cards", async ({ page }) => {
  const screens = [
    ["overview", "Admission"],
    ["ingress", "WorkPayload::Tx"],
    ["planning", "BatchPlanned"],
    ["placement", "SecondaryState"],
    ["publication", "quorum digests"],
    ["recovery", "ShardExecState"],
  ];

  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    for (const [screenId, contractField] of screens) {
      await page.goto(`${demoUrl}?route=telemetry.aggregators.${screenId}`);
      const screen = page.locator(`[data-aggregator-screen="${screenId}"]`);
      await expect(screen).toBeVisible();
      await expect(page.locator(".route-preview")).toHaveCount(0);
      await expect(page.locator(".capability-boundary")).toHaveCount(0);
      await expect(screen.locator(".aggregator-contract-card").first()).toContainText(contractField);
      await expect(screen.locator(".network-summary-grid strong")).toHaveText([
        "Unavailable",
        "Unavailable",
        "Unavailable",
        "Unavailable",
      ]);
      await expectNoViewportOverflow(page);
    }
  }
});

test("mobile drawer uses the same root-only accordion tree and preserves the topbar logo", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  await expect(page.locator(".mobile-nav-brand img")).toBeVisible();
  await expect(page.locator(".desktop-topbar-brand")).not.toBeVisible();

  await page.locator("#mobile-menu-button").click();
  const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
  await expect(drawer).toBeVisible();
  await expect(drawer.locator(".mobile-navigation-scroll-region")).toBeVisible();
  await expect(drawer.locator('[data-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  const telemetry = drawer.locator('[data-navigation-branch="telemetry"]');
  await expect(telemetry).toHaveAttribute("aria-expanded", "false");
  await telemetry.click();
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(drawer.locator('[data-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("#page-title")).toHaveText("Assets");
  const wallet = drawer.locator('[data-navigation-branch="wallet"]');
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "false");
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(page.locator("#page-title")).toHaveText("Assets");
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(drawer.locator('[data-navigation-branch="telemetry.reticulum"]')).toHaveCount(0);
  await expect(drawer.locator('[data-navigation-workspace="telemetry.reticulum"]')).toBeVisible();
  await expect(drawer.locator('[data-navigation-workspace="telemetry.onionnet"]')).toBeVisible();
  await expect(drawer.locator('[data-navigation-workspace="telemetry.aggregators"]')).toBeVisible();
  await expect(drawer.locator('[data-navigation-route="telemetry.reticulum.node"]')).toHaveCount(0);
  const dapps = drawer.locator('[data-navigation-branch="dapps"]');
  if (await dapps.getAttribute("aria-expanded") === "false") await dapps.click();
  const terminal = drawer.locator(".mobile-navigation-terminal");
  const mobileScrollContract = await drawer.locator(".mobile-navigation-scroll-region").evaluate((region) => {
    const terminal = region.querySelector(".mobile-navigation-terminal");
    const regionRect = region.getBoundingClientRect();
    region.scrollTop = region.scrollHeight;
    const terminalRect = terminal?.getBoundingClientRect();
    return {
      terminalIsInsideScrollRegion: terminal?.parentElement === region,
      clientHeight: region.clientHeight,
      scrollHeight: region.scrollHeight,
      scrollTop: region.scrollTop,
      terminalFullyReachable: Boolean(terminalRect
        && terminalRect.top >= regionRect.top - 1
        && terminalRect.bottom <= regionRect.bottom + 1),
    };
  });
  expect(mobileScrollContract.terminalIsInsideScrollRegion).toBe(true);
  expect(mobileScrollContract.scrollHeight).toBeGreaterThan(mobileScrollContract.clientHeight);
  expect(mobileScrollContract.scrollTop).toBeGreaterThan(0);
  expect(mobileScrollContract.terminalFullyReachable).toBe(true);
  await expect(terminal.getByRole("button", { name: "Settings", exact: true })).toBeVisible();
  await expect(terminal.getByRole("button", { name: "Help", exact: true })).toBeVisible();
  await expect(terminal.getByRole("button", { name: "About", exact: true })).toBeVisible();
  const mobileTypography = await drawer.locator('[data-navigation-branch="telemetry"]').evaluate((element) => {
    const style = getComputedStyle(element);
    return {
      family: style.fontFamily,
      size: style.fontSize,
      weight: style.fontWeight,
      lineHeight: style.lineHeight,
    };
  });
  expect(mobileTypography.family).toContain("Geist");
  expect(mobileTypography).toMatchObject({
    size: "16px",
    weight: "700",
    lineHeight: "20px",
  });
  await telemetry.click();
  await expect(telemetry).toHaveAttribute("aria-expanded", "false");
  await expect(drawer.locator('[data-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");

  await selectCanonicalRoute(page, "wallet.send", { mobile: true });
  await expect(drawer).toBeHidden();
  await expect(page.locator("#page-title")).toHaveText("Send");
  await expectNoViewportOverflow(page);
});

test("mobile profile selection, Help, and drawer focus are independent of desktop navigation", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  await page.locator("#mobile-menu-button").click();
  const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
  await drawer.locator('[data-mobile-wallet-id="travel"]').click();
  await expect(drawer).toBeHidden();
  await expect(page.locator("#wallet-identity")).toContainText("Travel");

  await page.locator("#mobile-menu-button").click();
  await expect(page.locator("#app-body")).toHaveJSProperty("inert", true);
  await expect(drawer.locator("[data-mobile-popup-close]")).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(drawer.getByRole("button", { name: "Log out", exact: true })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(drawer.locator("[data-mobile-popup-close]")).toBeFocused();
  await page.locator("#mobile-menu-backdrop").click({ position: { x: 380, y: 400 } });
  await expect(drawer).toBeHidden();
  await expect(page.locator("#app-body")).toHaveJSProperty("inert", false);
  await expect(page.locator("#mobile-menu-button")).toBeFocused();

  await page.locator("#mobile-menu-button").click();
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await expect(page.locator("#mobile-menu-button")).toBeFocused();

  await page.locator("#mobile-menu-button").click();
  const helpPage = await openStandaloneHelp(page, drawer.getByRole("button", { name: "Help", exact: true }));
  await helpPage.setViewportSize({ width: 390, height: 844 });
  await expect(helpPage.locator("#help-document")).toBeVisible();
  await expect(helpPage.locator("#help-context-tabs")).toBeHidden();
  await expect(helpPage.locator("[data-help-context-topic]")).toHaveCount(1);
  await helpPage.close();
  await expect(drawer).toBeHidden();
  await expectNoViewportOverflow(page);
});

test("mobile contextual Help exposes sibling topics as top tabs without opening its drawer", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const helpPage = await openStandaloneHelp(
    page,
    page.getByRole("button", { name: "Help for this view" }),
  );
  await helpPage.setViewportSize({ width: 390, height: 844 });

  const contextTabs = helpPage.locator("#help-context-tabs");
  await expect(contextTabs).toBeVisible();
  await expect(contextTabs.locator("[data-help-context-topic]")).toHaveCount(3);
  await expect(contextTabs.locator('[data-help-context-topic="wallet.assets"]')).toHaveAttribute("aria-current", "page");
  await expect(helpPage.locator('#help-tree [data-help-topic-link="wallet.assets"]')).toContainText("Assets");
  await expect(helpPage.locator('#help-tree [data-help-topic-link="wallet.vouchers"]')).toHaveCount(0);
  await expect(helpPage.locator('#help-tree [data-help-topic-link="asset.details"]')).toHaveCount(0);
  await expect(helpPage.locator('[data-help-group="telemetry"]')).toContainText("Telemetry");
  await expect(helpPage.locator("#help-sidebar")).not.toHaveClass(/is-open/);
  await expect(helpPage.locator("#help-menu-button")).toHaveAttribute("aria-expanded", "false");

  await contextTabs.locator('[data-help-context-topic="wallet.vouchers"]').click();
  await expect(contextTabs.locator('[data-help-context-topic="wallet.vouchers"]')).toHaveAttribute("aria-current", "page");
  await expect(helpPage.locator("#help-title")).toContainText("Vouchers");
  await expect(helpPage).toHaveURL(/topic=wallet\.vouchers/);
  await helpPage.close();

  await page.goto(`${demoUrl}?route=wallet.staking.stake`);
  const stakingHelpPage = await openStandaloneHelp(
    page,
    page.getByRole("button", { name: "Help for this view" }),
  );
  await stakingHelpPage.setViewportSize({ width: 390, height: 844 });
  const stakingTabs = stakingHelpPage.locator("#help-context-tabs");
  await expect(stakingTabs.locator("[data-help-context-topic]")).toHaveCount(2);
  await expect(stakingTabs.locator('[data-help-context-topic="wallet.staking.stake"]')).toHaveAttribute("aria-current", "page");
  await stakingTabs.locator('[data-help-context-topic="wallet.staking.unstake"]').click();
  await expect(stakingHelpPage.locator("#help-title")).toContainText("Unstake");
  await expect(stakingHelpPage).toHaveURL(/topic=wallet\.staking\.unstake/);
  await stakingHelpPage.close();
});

test("standalone Help keeps root accordions independent and projects workspace topics responsively", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=dapps.discover`);
  const helpPage = await openStandaloneHelp(
    page,
    page.getByRole("button", { name: "Help for this view" }),
  );
  await helpPage.setViewportSize({ width: 1280, height: 800 });

  const rootGroups = helpPage.locator("[data-help-group]");
  await expect(rootGroups).toHaveCount(8);
  await expect(helpPage.locator(".help-tree-items [data-help-group]")).toHaveCount(0);
  const walletGroup = helpPage.locator('[data-help-group="wallets"]');
  const telemetryGroup = helpPage.locator('[data-help-group="telemetry"]');
  await walletGroup.click();
  await telemetryGroup.click();
  await expect(walletGroup).toHaveAttribute("aria-expanded", "true");
  await expect(telemetryGroup).toHaveAttribute("aria-expanded", "true");
  await walletGroup.click();
  await expect(walletGroup).toHaveAttribute("aria-expanded", "false");
  await expect(telemetryGroup).toHaveAttribute("aria-expanded", "true");

  const desktopTopics = helpPage.locator("#help-context-tabs [data-help-context-topic]");
  await expect(desktopTopics).toHaveCount(6);
  await expect(desktopTopics).toHaveText(["Discover", "Installed", "Connections", "Permissions", "Swap", "Exchange"]);
  const desktopGeometry = await Promise.all([
    helpPage.locator("#help-context-tabs").boundingBox(),
    helpPage.locator("#help-document").boundingBox(),
  ]);
  expect(desktopGeometry[0].x + desktopGeometry[0].width).toBeLessThanOrEqual(desktopGeometry[1].x);

  await helpPage.setViewportSize({ width: 390, height: 844 });
  const mobileGeometry = await helpPage.evaluate(() => {
    const header = document.querySelector(".help-site-header").getBoundingClientRect();
    const tabs = document.querySelector("#help-context-tabs").getBoundingClientRect();
    return { headerBottom: header.bottom, tabsTop: tabs.top };
  });
  expect(Math.abs(mobileGeometry.tabsTop - mobileGeometry.headerBottom)).toBeLessThanOrEqual(1);
  await helpPage.locator("#help-menu-button").click();
  await expect(helpPage.locator("#help-sidebar")).toHaveClass(/is-open/);
  await expect(telemetryGroup).toHaveAttribute("aria-expanded", "true");
  await walletGroup.click();
  await expect(walletGroup).toHaveAttribute("aria-expanded", "true");
  await telemetryGroup.click();
  await expect(telemetryGroup).toHaveAttribute("aria-expanded", "false");
  await expect(walletGroup).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator("#help-sidebar-close").click();
  await expect(helpPage.locator("#help-sidebar")).not.toHaveClass(/is-open/);
  await helpPage.close();
});

test("global and contextual Help reuse one named surface and preserve application state", async ({ page, context }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.send`);
  await page.locator("#send-recipient").fill("z00z1-help-state-preserved");
  await page.locator("#send-amount").fill("17");

  const helpPage = await openStandaloneHelp(page, page.locator("#app-navigation-terminal [data-help-topic]"));
  await expect(helpPage).toHaveURL(/topic=app/);
  expect(await helpPage.evaluate(() => window.name)).toBe("z00z-help");
  expect(context.pages()).toHaveLength(2);

  await page.bringToFront();
  await page.getByRole("button", { name: "Help for this view" }).click();
  await expect(helpPage).toHaveURL(/topic=wallet\.send/);
  expect(context.pages()).toHaveLength(2);
  await expect(page.locator("#send-recipient")).toHaveValue("z00z1-help-state-preserved");
  await expect(page.locator("#send-amount")).toHaveValue("17");

  await helpPage.close();
  await expect(page.locator("#send-recipient")).toHaveValue("z00z1-help-state-preserved");
  await expect(page.locator("#send-amount")).toHaveValue("17");
});

test("context Help follows detail and review state instead of the containing route", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });

  await page.goto(`${demoUrl}?route=dapps.discover`);
  await page.locator("[data-dapp-card] [data-dapp-action='open']").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "dapps.detail");
  await page.locator("[data-dapp-action='back']").click();
  await page.goto(`${demoUrl}?route=dapps.connections`);
  await page.locator("[data-dapp-connection] [data-dapp-action='review']").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "dapps.permission-review");

  await page.goto(`${demoUrl}?route=messenger.inbox`);
  await page.locator("[data-messenger-message] [data-messenger-action='open']").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "messenger.detail");

  await page.goto(`${demoUrl}?route=contacts.list`);
  await page.locator("[data-contact] [data-contact-action='open']").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "contacts.detail");

  await page.goto(`${demoUrl}?route=telemetry.watchers.alerts`);
  await page.locator("[data-watcher-alert]").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "telemetry.watchers.alert-detail");

  await page.goto(`${demoUrl}?route=telemetry.explorer.checkpoints`);
  await page.locator("[data-explorer-record]").first().click();
  await expect(page.locator(".context-help-button")).toHaveAttribute("data-help-topic", "telemetry.explorer.detail");
});

test("compact route context, status, privacy, attention, and lock utilities keep the shell singular", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  await expect(page.locator("#route-breadcrumb")).toContainText("Wallet");
  await expect(page.locator("#page-context")).toBeHidden();
  const contextualHelpGap = await page.evaluate(() => {
    const helpButton = document.querySelector(".context-help-button").getBoundingClientRect();
    const statusbar = document.querySelector("#wallet-statusbar").getBoundingClientRect();
    return Math.round(statusbar.top - helpButton.bottom);
  });
  expect(contextualHelpGap).toBe(20);
  await expect(page.locator("#wallet-statusbar")).toContainText("Available");
  await expect(page.locator("#wallet-tabs")).toHaveCount(0);

  const privacyButton = page.locator('[data-demo-action="toggle-balance"]').first();
  await privacyButton.click();
  await expect(privacyButton).toHaveAttribute("aria-label", "Show sensitive amounts");
  await page.locator('[data-demo-action="notifications"]').click();
  await expect(page.getByText("One item needs attention", { exact: true })).toBeVisible();
  await page.keyboard.press("Escape");

  await selectCanonicalRoute(page, "wallet.settings.general");
  await expect(page.locator(".wallet-settings-context [data-wallet-settings-section]")).toHaveCount(5);
  await page.locator('[data-wallet-settings-section="security"]').click();
  await page.locator('[data-demo-action="lock"]').click();
  await expect(page.locator("#lock-screen")).toBeVisible();
  await expect(page.locator("#lock-screen .brand")).toBeVisible();
  await expect(page.locator("#app-shell")).toBeHidden();
  await expect(page.locator("#unlock-password")).toBeFocused();
  await page.locator("#unlock-password").fill("demo");
  await page.locator("#unlock-form").evaluate((form) => form.requestSubmit());
  await expect(page.locator("#lock-screen")).toBeHidden();
  await expect(page.locator("#app-shell")).toBeVisible();
});

test("desktop and mobile maintain responsive geometry on canonical routes", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=settings.appearance`);
    await expect(page.locator("#main-content")).toBeVisible();
    await expectNoViewportOverflow(page);
  }
});

test("active-route mounting and workspace failures stay isolated from the branded shell", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 320, height: 800, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=telemetry.watchers.overview&workspaceFailure=telemetry.watchers.overview`);

    await expect(page.locator("#main-content")).toHaveAttribute("data-mounted-route", "telemetry.watchers.overview");
    await expect(page.locator('.workspace-error-boundary[data-workspace-error="telemetry.watchers.overview"]')).toBeVisible();
    await expect(page.locator(viewport.mobile ? ".mobile-nav-brand img" : ".desktop-topbar-brand img")).toBeVisible();
    await expect(page.locator("#app-shell")).toBeVisible();

    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    await selectCanonicalRoute(page, "wallet.send", { mobile: viewport.mobile });
    await expect(page.locator("#main-content")).toHaveAttribute("data-mounted-route", "wallet.send");
    await expect(page.locator(".workspace-error-boundary")).toHaveCount(0);
    await expect(page.locator("#page-title")).toHaveText("Send");

    await page.goto(`${demoUrl}?route=telemetry.watchers.overview&workspaceFailure=telemetry.watchers.overview`);
    await page.locator('[data-demo-action="retry-workspace"]').click();
    await expect(page).not.toHaveURL(/workspaceFailure=/);
    await expect(page.locator('[data-workspace-id="telemetry.watchers"]')).toBeVisible();
    await expect(page.locator("#main-content")).toHaveAttribute("data-mounted-route", "telemetry.watchers.overview");
  }
});

test("navigation and workspace-local destinations expose screen-reader state without nested accordions", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=telemetry.reticulum.node`);

  const rootBranches = page.locator("#app-navigation-tree > .navigation-tree-branch > [data-navigation-branch]");
  await expect(rootBranches).toHaveCount(5);
  for (const branch of await rootBranches.all()) {
    await expect(branch).toHaveAttribute("aria-expanded", /true|false/);
    const controls = await branch.getAttribute("aria-controls");
    expect(controls).toBeTruthy();
    await expect(page.locator(`#${controls}`)).toHaveCount(1);
  }
  await expect(page.locator("#app-navigation-tree .navigation-tree-children [data-navigation-branch]")).toHaveCount(0);
  await expect(page.locator('[data-navigation-workspace="telemetry.reticulum"]')).toHaveAttribute("aria-current", "page");

  const localNavigation = page.locator('[data-workspace-id="telemetry.reticulum"] .workspace-local-context');
  await expect(localNavigation).toHaveAttribute("aria-label", "Reticulum");
  await expect(localNavigation.locator('[data-workspace-route="telemetry.reticulum.node"]')).toHaveAttribute("aria-current", "page");
  await expect(localNavigation.locator("[aria-current='page']")).toHaveCount(1);

  await page.keyboard.press("Tab");
  await expect(page.locator(":focus")).toBeVisible();
});

test("mobile navigation meets touch-target, safe-area, reduced-motion, and software-keyboard contracts", async ({ page }) => {
  const [components, helpStyles] = await Promise.all([
    readFile(path.join(demoDir, "styles/components.css"), "utf8"),
    readFile(path.join(demoDir, "styles/help.css"), "utf8"),
  ]);
  expect(components).toContain("env(safe-area-inset-top)");
  expect(components).toContain("env(safe-area-inset-bottom)");
  expect(helpStyles).toContain("env(safe-area-inset-top)");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  await page.locator("#mobile-menu-button").click();
  const targets = page.locator(
    '#mobile-popup-menu button:visible, #mobile-popup-menu [data-navigation-route]:visible, #mobile-popup-menu [data-navigation-workspace]:visible',
  );
  expect(await targets.count()).toBeGreaterThan(10);
  const undersized = await targets.evaluateAll((nodes) => nodes
    .map((node) => {
      const rect = node.getBoundingClientRect();
      return { label: node.textContent.trim(), width: rect.width, height: rect.height };
    })
    .filter(({ width, height }) => width < 44 || height < 44));
  expect(undersized).toEqual([]);
  await page.keyboard.press("Escape");

  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.locator("#mobile-menu-button").click();
  const motion = await page.locator("#mobile-popup-menu").evaluate((node) => {
    const style = getComputedStyle(node);
    const seconds = (value) => value.split(",").map((part) => {
      const duration = part.trim();
      return duration.endsWith("ms") ? Number.parseFloat(duration) / 1000 : Number.parseFloat(duration);
    });
    return {
      animations: seconds(style.animationDuration),
      transitions: seconds(style.transitionDuration),
    };
  });
  expect(Math.max(...motion.animations, ...motion.transitions)).toBeLessThanOrEqual(0.00002);
  await page.keyboard.press("Escape");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.send`);
  await page.locator("#send-recipient").focus();
  await page.setViewportSize({ width: 390, height: 420 });
  await page.locator("#send-recipient").scrollIntoViewIfNeeded();
  const activeInput = await page.locator("#send-recipient").boundingBox();
  expect(activeInput.y).toBeGreaterThanOrEqual(0);
  expect(activeInput.y + activeInput.height).toBeLessThanOrEqual(420);
  await expectNoViewportOverflow(page);
});

test("200 percent text zoom preserves canonical routes at desktop and constrained mobile widths", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800 },
    { width: 320, height: 800 },
  ]) {
    for (const routeId of [
      "wallet.assets",
      "wallet.send",
      "telemetry.reticulum.node",
      "telemetry.watchers.alerts",
      "dapps.connections",
      "messenger.inbox",
      "contacts.list",
      "settings.appearance",
    ]) {
      await page.setViewportSize(viewport);
      await page.goto(`${demoUrl}?route=${routeId}`);
      await page.evaluate(() => {
        document.documentElement.style.fontSize = "200%";
      });
      await expect(page.locator(viewport.width <= 767 ? ".mobile-nav-brand img" : ".desktop-topbar-brand img")).toBeVisible();
      await expectNoViewportOverflow(page, `${routeId} at ${viewport.width}px and 200% text zoom`);
    }
  }
});

test("packaged Help invokes one bounded command without wallet or draft data", async ({ page }) => {
  await page.addInitScript(() => {
    Object.defineProperty(window, "__TAURI__", {
      configurable: true,
      value: {
        core: {
          invoke(command, payload) {
            window.__z00zCapturedNativeHelp = { command, payload };
            return Promise.resolve();
          },
        },
      },
    });
  });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.send&palette=z00z-corporate`);
  await page.locator("#send-recipient").fill("receiver_secret_never_forward");
  await page.locator("#send-amount").fill("314.159");
  await page.getByRole("button", { name: "Help for this view" }).click();

  const captured = await page.evaluate(() => window.__z00zCapturedNativeHelp);
  expect(captured.command).toBe("open_or_focus_help");
  expect(Object.keys(captured.payload)).toEqual(["request"]);
  expect(Object.keys(captured.payload.request).sort()).toEqual(["locale", "palette", "section", "topicId"]);
  expect(captured.payload.request).toEqual({
    topicId: "wallet.send",
    locale: "en",
    palette: "z00z-corporate",
    section: "current-view",
  });
  expect(JSON.stringify(captured)).not.toContain("receiver_secret_never_forward");
  expect(JSON.stringify(captured)).not.toContain("314.159");
});

test("768px narrow tablet uses the drawer and keeps its branded header while the tree scrolls", async ({ page }) => {
  await page.setViewportSize({ width: 768, height: 1024 });
  for (const palette of ["z00z-default", "z00z-corporate"]) {
    await page.goto(`${demoUrl}?route=messenger.sent&palette=${palette}`);
    await expect(page.locator(".mobile-nav-brand img")).toBeVisible();
    await expect(page.locator(".desktop-topbar-brand")).not.toBeVisible();
    await expect(page.locator("html")).toHaveAttribute("data-palette", palette);
    await expectNoViewportOverflow(page, `Messenger Sent at 768px in ${palette}`);

    await page.goto(`${demoUrl}?route=telemetry.reticulum.node&palette=${palette}`);
    const localRail = await visibleContextNavigation(page, '[data-workspace-id="telemetry.reticulum"] > .context-rail');
    await expect(localRail).toBeVisible();
    const railGeometry = await localRail.evaluate((node) => ({
      clientWidth: node.clientWidth,
      scrollWidth: node.scrollWidth,
      position: getComputedStyle(node).position,
    }));
    expect(railGeometry.position).toBe("static");
    expect(railGeometry.scrollWidth).toBeGreaterThan(railGeometry.clientWidth);
    await expectNoViewportOverflow(page, `Reticulum local tabs at 768px in ${palette}`);
  }

  await page.locator("#mobile-menu-button").click();
  const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
  const drawerHeader = drawer.locator(".mobile-drawer-header");
  const scrollRegion = drawer.locator(".mobile-navigation-scroll-region");
  await scrollRegion.evaluate((node) => {
    node.scrollTop = node.scrollHeight;
  });
  await expect(drawerHeader).toBeVisible();
  const positions = await Promise.all([
    page.locator(".topbar").boundingBox(),
    drawerHeader.boundingBox(),
  ]);
  expect(Math.abs(positions[0].y + positions[0].height - positions[1].y)).toBeLessThanOrEqual(1);
  await expect(drawer.getByRole("button", { name: "Close" })).toBeVisible();
});
