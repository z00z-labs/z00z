const { mkdir, readFile, writeFile } = require("node:fs/promises");
const path = require("node:path");
const vm = require("node:vm");
const { test, expect } = require("playwright/test");

test.setTimeout(2_400_000);

const demoUrl = process.env.Z00Z_WALLET_DEMO_URL;
const dappMenuLabels = [
  "Agent Budget",
  "Assets Locker",
  "Bounty",
  "Create Asset",
  "Create Permission",
  "Create Voucher",
  "Digital Goods",
  "Donation",
  "Escrow",
  "Pay",
  "Payroll",
  "Private Agreement",
  "Request",
  "Service Credit",
  "Subscription",
  "Swap",
  "Ticket & Pass",
  "wCoins Gateway",
  "X-Chain Integration",
  "Discover dApps",
];
const dappReviewRoutes = [
  ["dapps.agents-budget", "dapps-agents-budget"],
  ["dapps.assets-locker", "dapps-assets-locker"],
  ["dapps.bounties", "dapps-bounties"],
  ["dapps.create-asset", "dapps-create-asset"],
  ["dapps.create-permission", "dapps-create-permission"],
  ["dapps.create-voucher", "dapps-create-voucher"],
  ["dapps.digital-goods", "dapps-digital-goods"],
  ["dapps.donation", "dapps-donation"],
  ["dapps.escrow", "dapps-escrow"],
  ["dapps.pay", "dapps-pay"],
  ["dapps.payroll", "dapps-payroll"],
  ["dapps.private-contract", "dapps-private-contract"],
  ["dapps.request", "dapps-request"],
  ["dapps.service-credits", "dapps-service-credits"],
  ["dapps.subscription", "dapps-subscription"],
  ["dapps.tickets-passes", "dapps-tickets-passes"],
  ["dapps.wbold-gateway", "dapps-wbold-gateway"],
  ["dapps.xchain-integration", "dapps-xchain-integration"],
];
const reviewRoot = path.resolve(process.env.Z00Z_VISUAL_REVIEW_DIR || path.join(
  __dirname,
  "../../z00z_storage/outputs/checkpoint/phase-110/ui-help-review",
));

const viewports = [
  { name: "desktop-1280", width: 1280, height: 800 },
  { name: "desktop-1024", width: 1024, height: 768 },
  { name: "tablet-768", width: 768, height: 1024 },
  { name: "mobile-390", width: 390, height: 844 },
  { name: "mobile-320", width: 320, height: 800 },
];

async function loadPortContract() {
  const sourcePath = path.resolve(__dirname, "scripts/port/contracts.js");
  const source = await readFile(sourcePath, "utf8");
  const sandbox = { URLSearchParams };
  sandbox.globalThis = sandbox;
  vm.runInNewContext(source, sandbox, { filename: sourcePath });
  return sandbox.Z00ZDemo.PORT_CONTRACT;
}

function routeQuery(parameters) {
  return `?${new URLSearchParams(parameters).toString()}`;
}

function allReviewRoutes(contract) {
  return contract.routes.map((routeId) => ({
    name: routeId.replaceAll(".", "-"),
    routeId,
  }));
}

async function capture(page, name, { fullPage = false } = {}) {
  await expect(page.locator("#main-content")).toBeVisible();
  await page.evaluate(() => document.fonts?.ready);
  await page.waitForTimeout(180);
  await page.screenshot({
    path: path.join(reviewRoot, `${name}.png`),
    fullPage,
  });
}

async function captureLock(page, name, { fullPage = false } = {}) {
  await expect(page.locator("#lock-screen")).toBeVisible();
  await expect(page.locator("#lock-screen .brand")).toBeVisible();
  await page.screenshot({
    path: path.join(reviewRoot, `${name}.png`),
    fullPage,
  });
}

async function captureHelp(page, trigger, viewport, name) {
  const popupPromise = page.waitForEvent("popup");
  await trigger.click();
  const helpPage = await popupPromise;
  await helpPage.setViewportSize({ width: viewport.width, height: viewport.height });
  await expect(helpPage.locator("#help-document")).toBeVisible();
  await helpPage.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
  await helpPage.screenshot({
    path: path.join(reviewRoot, `${name}.png`),
    fullPage: false,
  });
  await helpPage.close();
}

async function selectAppLanguage(page, languageId) {
  await page.locator("[data-language-picker-trigger]").click();
  await page.locator(`[data-language-picker-option="${languageId}"]`).click();
}

async function setBranchExpanded(branch, expanded) {
  const expected = String(expanded);
  if (await branch.getAttribute("aria-expanded") !== expected) await branch.click();
  await expect(branch).toHaveAttribute("aria-expanded", expected);
}

async function settleMainAnimations(page) {
  await page.locator("#main-content").evaluate((main) => (
    Promise.all(main.getAnimations({ subtree: true })
      .filter((animation) => Number.isFinite(animation.effect?.getTiming().iterations ?? 1))
      .map((animation) => animation.finished))
  ));
}

async function auditResponsiveGeometry(page, viewport, route) {
  await page.evaluate(() => document.fonts?.ready);
  await page.waitForTimeout(40);

  return page.evaluate(({ viewportName, routeName }) => {
    const tolerance = 1;
    const ignoredOverflowHost = ".choice-strip, .filter-bar, .context-rail, .yaml-editor, .help-contents";
    const ignoredOverlapParent = /(?:icon|badge|logo|avatar|step|toggle|orb|indicator|glyph|balance-amount|yaml-highlight)/;
    const issues = [];
    const roundedRect = (rect) => ({
      left: Math.round(rect.left * 10) / 10,
      top: Math.round(rect.top * 10) / 10,
      right: Math.round(rect.right * 10) / 10,
      bottom: Math.round(rect.bottom * 10) / 10,
      width: Math.round(rect.width * 10) / 10,
      height: Math.round(rect.height * 10) / 10,
    });
    const visible = (element) => {
      const style = getComputedStyle(element);
      const rect = element.getBoundingClientRect();
      return style.display !== "none"
        && style.visibility !== "hidden"
        && Number(style.opacity) !== 0
        && rect.width > 0
        && rect.height > 0;
    };
    const elementName = (element) => {
      const id = element.id ? `#${element.id}` : "";
      const classes = [...element.classList].slice(0, 3).map((name) => `.${name}`).join("");
      return `${element.tagName.toLowerCase()}${id}${classes}`;
    };

    if (document.documentElement.scrollWidth > window.innerWidth + tolerance) {
      issues.push({
        type: "viewport-overflow",
        scrollWidth: document.documentElement.scrollWidth,
        viewportWidth: window.innerWidth,
      });
    }

    const main = document.querySelector("#main-content");
    if (main) {
      const mainRect = main.getBoundingClientRect();
      if (mainRect.left < -tolerance || mainRect.right > window.innerWidth + tolerance) {
        issues.push({ type: "main-outside-viewport", element: elementName(main), rect: roundedRect(mainRect) });
      }
    }

    document.querySelectorAll("#main-content *").forEach((element) => {
      if (!(element instanceof HTMLElement) || !visible(element) || element.closest(ignoredOverflowHost)) return;
      const style = getComputedStyle(element);
      if (["inline", "contents"].includes(style.display) || ["absolute", "fixed"].includes(style.position)) return;
      const rect = element.getBoundingClientRect();
      if (rect.left < -tolerance || rect.right > window.innerWidth + tolerance) {
        issues.push({ type: "element-outside-viewport", element: elementName(element), rect: roundedRect(rect) });
      }
    });

    document.querySelectorAll("#main-content *").forEach((parent) => {
      if (!(parent instanceof HTMLElement) || ignoredOverlapParent.test(parent.className || "")) return;
      const children = [...parent.children].filter((child) => {
        if (!(child instanceof HTMLElement) || !visible(child)) return false;
        const style = getComputedStyle(child);
        return !["absolute", "fixed", "sticky"].includes(style.position);
      });
      for (let firstIndex = 0; firstIndex < children.length; firstIndex += 1) {
        const first = children[firstIndex];
        const firstRect = first.getBoundingClientRect();
        for (let secondIndex = firstIndex + 1; secondIndex < children.length; secondIndex += 1) {
          const second = children[secondIndex];
          const secondRect = second.getBoundingClientRect();
          const overlapWidth = Math.min(firstRect.right, secondRect.right) - Math.max(firstRect.left, secondRect.left);
          const overlapHeight = Math.min(firstRect.bottom, secondRect.bottom) - Math.max(firstRect.top, secondRect.top);
          if (overlapWidth > 2 && overlapHeight > 2) {
            issues.push({
              type: "sibling-overlap",
              parent: elementName(parent),
              first: elementName(first),
              second: elementName(second),
              overlap: {
                width: Math.round(overlapWidth * 10) / 10,
                height: Math.round(overlapHeight * 10) / 10,
              },
            });
          }
        }
      }
    });

    return {
      viewport: viewportName,
      route: routeName,
      url: location.href,
      viewportWidth: window.innerWidth,
      documentWidth: document.documentElement.scrollWidth,
      issues: issues.slice(0, 50),
    };
  }, { viewportName: viewport.name, routeName: route.name });
}

test("capture Demo layouts and English Help review matrix", async ({ page }) => {
  await mkdir(reviewRoot, { recursive: true });
  const reviewRoutes = allReviewRoutes(await loadPortContract());
  const layoutAudit = [];

  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    for (const palette of ["z00z-default", "z00z-corporate"]) {
      const paletteName = palette === "z00z-default" ? "default" : "corporate";
      for (const route of reviewRoutes) {
        await page.goto(`${demoUrl}${routeQuery({ route: route.routeId, palette })}`);
        await expect(page.locator("html")).toHaveAttribute("data-palette", palette);
        const stateName = `${paletteName}-${route.name}`;
        layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: stateName }));
        await capture(page, `${viewport.name}-${stateName}`, { fullPage: viewport.width <= 768 });
      }
    }

    await page.goto(`${demoUrl}?route=wallet.send&operationScenario=timeout_unknown_outcome`);
    await page.locator("#send-recipient").fill("z00z1visual-review");
    await page.locator("#send-amount").fill("12.50");
    await page.locator("#send-entry").evaluate((form) => form.requestSubmit());
    await page.locator('[data-send-action="submit"]').click();
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "send-operation-submitting" }));
    await capture(page, `${viewport.name}-send-operation-submitting`, { fullPage: viewport.width <= 768 });

    await expect(page.locator(".operation-error-state")).toBeVisible();
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "send-operation-unknown-outcome" }));
    await capture(page, `${viewport.name}-send-operation-unknown-outcome`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-send-action="reconcile"]').click();
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "send-operation-reconciling" }));
    await capture(page, `${viewport.name}-send-operation-reconciling`, { fullPage: viewport.width <= 768 });

    await expect(page.locator(".result-state")).toBeVisible();
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "send-operation-result" }));
    await capture(page, `${viewport.name}-send-operation-result`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=telemetry.watchers.alerts`);
    await page.locator('[data-watcher-alert="watcher-alert-002"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "watchers-alert-detail" }));
    await capture(page, `${viewport.name}-watchers-alert-detail`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-watcher-action="open-explorer"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "watchers-explorer-deep-link" }));
    await capture(page, `${viewport.name}-watchers-explorer-deep-link`, { fullPage: viewport.width <= 768 });
    await page.goBack();
    await expect(page.locator('[data-watcher-screen="alerts"]')).toBeVisible();

    await page.locator('[data-watcher-action="inspect-evidence"]').click();
    await page.locator('[data-watcher-action="export-evidence"][data-alert-id="watcher-alert-002"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "watchers-evidence-export" }));
    await capture(page, `${viewport.name}-watchers-evidence-export`, { fullPage: viewport.width <= 768 });

    for (const watcherScenario of ["loading", "degraded", "empty", "malformed", "error", "unavailable"]) {
      await page.locator('[data-watcher-control="scenario"]').selectOption(watcherScenario);
      await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
      await settleMainAnimations(page);
      layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: `watchers-${watcherScenario}` }));
      await capture(page, `${viewport.name}-watchers-${watcherScenario}`, { fullPage: viewport.width <= 768 });
    }

    await page.goto(`${demoUrl}?route=telemetry.explorer.search`);
    await page.locator("#explorer-public-id").fill("checkpoint_000184");
    await page.locator("#explorer-public-search").evaluate((form) => form.requestSubmit());
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-search-detail" }));
    await capture(page, `${viewport.name}-explorer-search-detail`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-explorer-action="technical"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-search-technical" }));
    await capture(page, `${viewport.name}-explorer-search-technical`, { fullPage: viewport.width <= 768 });

    await page.locator("#explorer-public-id").fill("receiver_secret_001");
    await page.locator("#explorer-public-search").evaluate((form) => form.requestSubmit());
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-private-rejected" }));
    await capture(page, `${viewport.name}-explorer-private-rejected`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=telemetry.explorer.checkpoints`);
    await page.locator('[data-explorer-record="checkpoint_000184"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-checkpoint-detail" }));
    await capture(page, `${viewport.name}-explorer-checkpoint-detail`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-explorer-open-id="publication_6f840184"]').click();
    await page.locator('[data-explorer-action="technical"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-publication-technical" }));
    await capture(page, `${viewport.name}-explorer-publication-technical`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-explorer-control="kind"]').selectOption("proof");
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "explorer-proof-filter" }));
    await capture(page, `${viewport.name}-explorer-proof-filter`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=telemetry.explorer.checkpoints`);
    for (const explorerScenario of ["loading", "degraded", "empty", "malformed", "error", "unavailable"]) {
      await page.locator('[data-explorer-control="scenario"]').selectOption(explorerScenario);
      await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
      await settleMainAnimations(page);
      layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: `explorer-${explorerScenario}` }));
      await capture(page, `${viewport.name}-explorer-${explorerScenario}`, { fullPage: viewport.width <= 768 });
    }

    await page.goto(`${demoUrl}?route=dapps.discover`);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "dapps-discover" }));
    await capture(page, `${viewport.name}-dapps-discover`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-dapp-card="assets-locker"] [data-dapp-action="open"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "dapps-detail" }));
    await capture(page, `${viewport.name}-dapps-detail`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=dapps.discover`);
    await page.locator('[data-dapp-card="pay"] [data-dapp-action="open"]').click();
    await page.locator('[data-dapp-action="review"]').click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "dapps-permission-review" }));
    await capture(page, `${viewport.name}-dapps-permission-review`, { fullPage: viewport.width <= 768 });

    await page.locator('[data-dapp-action="decide"][data-decision="rejected"]').click();
    await page.locator("#toast-region .toast").last().getByRole("button", { name: "Dismiss notification" }).click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "dapps-rejected-outcome" }));
    await capture(page, `${viewport.name}-dapps-rejected-outcome`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=dapps.discover`);
    await page.locator('[data-dapp-card="pay"] [data-dapp-action="open"]').click();
    await page.locator('[data-dapp-action="review"]').click();
    await page.locator('input[name="scopeConfirmed"]').check();
    await page.locator('input[name="reauthAcknowledged"]').check();
    await page.getByRole("button", { name: "Accept bounded intent" }).click();
    await page.locator("#toast-region .toast").last().getByRole("button", { name: "Dismiss notification" }).click();
    await page.locator('[data-dapp-action="wallet-review"]').click();
    await page.locator("#toast-region .toast").last().getByRole("button", { name: "Dismiss notification" }).click();
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await settleMainAnimations(page);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "dapps-wallet-handoff-send" }));
    await capture(page, `${viewport.name}-dapps-wallet-handoff-send`, { fullPage: viewport.width <= 768 });

    await page.goto(`${demoUrl}?route=settings.appearance`);
    await page.locator('#main-content [data-palette="z00z-corporate"]').click();
    await expect(page.locator('#main-content [data-palette="z00z-corporate"]')).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".palette-card-heading em")).toHaveText("Active");
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "settings-appearance-corporate" }));
    await capture(page, `${viewport.name}-settings-appearance-corporate`, { fullPage: viewport.width <= 768 });
    if (viewport.width <= 768) {
      await page.locator("#mobile-menu-button").click();
      await captureHelp(
        page,
        page.getByRole("button", { name: "Help", exact: true }),
        viewport,
        `${viewport.name}-corporate-global-help`,
      );
    } else {
      await captureHelp(
        page,
        page.locator("#app-navigation-terminal [data-help-topic]"),
        viewport,
        `${viewport.name}-corporate-global-help`,
      );
    }

    if (viewport.width > 768) {
      await page.goto(`${demoUrl}?route=wallet.assets`);
      for (const walletId of ["everyday", "savings", "travel"]) {
        await page.locator("#wallet-nav [data-wallet-picker-trigger]").click();
        await page.locator(`#wallet-picker-popup [data-wallet-picker-id="${walletId}"]`).click();
        await capture(page, `${viewport.name}-wallet-header-${walletId}`);
      }
    }

    await page.goto(`${demoUrl}?route=wallet.assets`);
    await page.locator(".asset-identity-button").first().click();
    await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
    await capture(page, `${viewport.name}-asset-details`);
    await captureHelp(page, page.locator(".dialog-help-button"), viewport, `${viewport.name}-asset-details-help`);
    await page.keyboard.press("Escape");

    await captureHelp(
      page,
      page.getByRole("button", { name: "Help for this view" }),
      viewport,
      `${viewport.name}-context-help`,
    );

    await page.goto(`${demoUrl}?route=wallet.settings.security`);
    await page.locator('[data-demo-action="lock"]').click();
    await captureLock(page, `${viewport.name}-lock`, { fullPage: viewport.width <= 768 });
    await page.locator("#unlock-password").fill("demo");
    await page.locator("#unlock-form").evaluate((form) => form.requestSubmit());
    await expect(page.locator("#app-shell")).toBeVisible();

    if (viewport.width <= 768) {
      await page.locator("#mobile-menu-button").click();
      await captureHelp(
        page,
        page.getByRole("button", { name: "Help", exact: true }),
        viewport,
        `${viewport.name}-global-help`,
      );
    } else {
      await captureHelp(page, page.locator("#app-navigation-terminal [data-help-topic]"), viewport, `${viewport.name}-global-help`);
    }

    if (viewport.width <= 768) {
      await page.goto(`${demoUrl}?route=wallet.assets`);
      await page.locator("#mobile-menu-button").click();
      const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
      const wallet = drawer.locator('[data-navigation-branch="wallet"]');
      const telemetry = drawer.locator('[data-navigation-branch="telemetry"]');
      await setBranchExpanded(wallet, true);
      await setBranchExpanded(telemetry, false);
      await setBranchExpanded(telemetry, true);
      await expect(wallet).toHaveAttribute("aria-expanded", "true");
      await capture(page, `${viewport.name}-wallet-telemetry-multi-open`);
      await drawer.locator(".mobile-navigation-scroll-region").evaluate((region) => {
        region.scrollTop = region.scrollHeight;
      });
      await capture(page, `${viewport.name}-wallet-telemetry-multi-open-lower-tree`);

      await wallet.click();
      await expect(wallet).toHaveAttribute("aria-expanded", "false");
      await expect(telemetry).toHaveAttribute("aria-expanded", "true");
      await capture(page, `${viewport.name}-telemetry-one-open`);
      await page.keyboard.press("Escape");
    }

    if (viewport.width === 320) {
      for (const locale of ["ru", "de", "fr", "pt", "tr", "ja", "ko", "zh-Hans"]) {
        await page.goto(`${demoUrl}?route=settings.general`);
        await selectAppLanguage(page, locale);
        await capture(page, `${viewport.name}-locale-${locale}`);
      }
    }

    if ([1280, 320].includes(viewport.width)) {
      await page.goto(`${demoUrl}${routeQuery({ route: "wallet.assets", palette: "z00z-default" })}`);
      await page.evaluate(() => {
        document.documentElement.style.fontSize = "200%";
      });
      layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "default-wallet-assets-zoom-200" }));
      await capture(page, `${viewport.name}-default-wallet-assets-zoom-200`, { fullPage: viewport.width <= 768 });

      await page.emulateMedia({ reducedMotion: "reduce" });
      await page.goto(`${demoUrl}${routeQuery({ route: "telemetry.watchers.alerts", palette: "z00z-default" })}`);
      layoutAudit.push(await auditResponsiveGeometry(page, viewport, { name: "default-watchers-reduced-motion" }));
      await capture(page, `${viewport.name}-default-watchers-reduced-motion`, { fullPage: viewport.width <= 768 });
      await page.emulateMedia({ reducedMotion: "no-preference" });
    }
  }

  const auditPath = path.join(reviewRoot, "responsive-layout-audit.json");
  await writeFile(auditPath, `${JSON.stringify(layoutAudit, null, 2)}\n`);
  const failedRoutes = layoutAudit.filter(({ issues }) => issues.length > 0);
  expect(failedRoutes, `Responsive geometry audit failed; inspect ${auditPath}`).toEqual([]);
});

test("capture Phase 6 desktop and mobile states", async ({ page }) => {
  await mkdir(reviewRoot, { recursive: true });
  const phaseViewports = [
    { name: "desktop-1280", width: 1280, height: 800 },
    { name: "mobile-320", width: 320, height: 800 },
  ];
  const layoutAudit = [];

  const reviewState = async (viewport, name, { fullPage = viewport.width <= 768 } = {}) => {
    const route = { name };
    await expect(page.locator(".capability-boundary")).toHaveCount(0);
    layoutAudit.push(await auditResponsiveGeometry(page, viewport, route));
    await capture(page, `${viewport.name}-phase-6-${name}`, { fullPage });
  };

  for (const viewport of phaseViewports) {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });

    await page.goto(`${demoUrl}?route=settings.appearance`);
    await page.locator('#main-content [data-palette="z00z-corporate"]').click();
    await expect(page.locator('#main-content [data-palette="z00z-corporate"]')).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator(".palette-card-heading em")).toHaveText("Active");
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await reviewState(viewport, "settings-appearance-corporate-active");

    for (const [route, name] of [
      ["wallet.swap", "wallet-swap"],
      ["wallet.staking.stake", "wallet-staking"],
      ["telemetry.aggregators.overview", "aggregators-overview"],
      ["telemetry.watchers.overview", "watchers-overview"],
      ["telemetry.explorer.overview", "explorer-overview"],
      ["dapps.discover", "dapps-discover"],
      ...dappReviewRoutes,
    ]) {
      await page.goto(`${demoUrl}?route=${route}`);
      await reviewState(viewport, name, {
        fullPage: viewport.width <= 768 || (route.startsWith("dapps.") && route !== "dapps.discover"),
      });
    }

    if (viewport.width > 768) {
      await page.goto(`${demoUrl}?route=telemetry.reticulum.overview`);
      await expect(page.locator("#wallet-identity")).toContainText("ZxChpo…2Mj8Pt");
      await expect(page.locator("#page-title")).toHaveText("Reticulum");
      await expect(page.locator("#page-context")).toBeHidden();
      await reviewState(viewport, "topbar-wallet-context");
    }

    await page.goto(`${demoUrl}?route=wallet.assets`);
    if (viewport.width <= 768) {
      const mobileTopbarContext = page.locator("#mobile-topbar-context");
      const mobileWalletIdentity = page.locator("#mobile-active-wallet");
      await expect(mobileTopbarContext.locator("[data-wallet-section]")).toHaveCount(3);
      await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
      await expect(page.locator(".wallet-assets-layout > .context-rail")).toBeHidden();
      await capture(page, `${viewport.name}-phase-6-mobile-topbar-assets`);

      await page.goto(`${demoUrl}?route=wallet.staking.unstake`);
      await expect(mobileTopbarContext.locator("[data-workspace-route]")).toHaveCount(2);
      await expect(mobileWalletIdentity).toContainText("Everyday wallet");
      await capture(page, `${viewport.name}-phase-6-mobile-topbar-staking`);

      await page.goto(`${demoUrl}?route=contacts.list`);
      await expect(mobileTopbarContext).toBeHidden();
      await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
      await capture(page, `${viewport.name}-phase-6-mobile-active-wallet-contacts`);

      await page.goto(`${demoUrl}?route=wallet.assets`);
    }
    if (viewport.width <= 768) {
      await page.locator("#mobile-menu-button").click();
      await page.locator("#mobile-popup-menu").evaluate((drawer) => (
        Promise.all(drawer.getAnimations({ subtree: true }).map((animation) => animation.finished))
      ));
    }
    const assetsNavigation = viewport.width <= 768
      ? page.locator("#mobile-popup-menu")
      : page.locator("#app-navigation-tree");
    await expect(assetsNavigation.locator('[data-navigation-route="wallet.assets"]')).toHaveText("Assets");
    await expect(page.locator("#page-title")).toHaveText("Assets");
    await capture(page, `${viewport.name}-phase-6-assets-navigation`);
    if (viewport.width <= 768) await page.keyboard.press("Escape");

    await page.goto(`${demoUrl}?route=messenger.inbox`);
    await reviewState(viewport, "messenger-inbox");
    await page.locator('[data-messenger-message="message_advisory_001"] [data-messenger-action="open"]').click();
    await reviewState(viewport, "messenger-detail");
    await page.goto(`${demoUrl}?route=messenger.inbox`);
    await page.locator('[data-messenger-message="message_payment_001"] [data-messenger-action="open"]').click();
    await page.locator('[data-messenger-action="review"]').click();
    await reviewState(viewport, "messenger-request-review");
    await page.locator('[data-messenger-action="accept-request"]').click();
    await page.locator('[data-messenger-action="wallet-review"]').click();
    await reviewState(viewport, "messenger-wallet-handoff");
    await page.goto(`${demoUrl}?route=messenger.sent&messengerRelay=unavailable`);
    await reviewState(viewport, "messenger-sent-unavailable");
    await page.goto(`${demoUrl}?route=messenger.conversations`);
    await reviewState(viewport, "messenger-conversations");

    await page.goto(`${demoUrl}?route=contacts.list`);
    await reviewState(viewport, "contacts-list");
    await page.locator('[data-contact="contact_ada"] [data-contact-action="open"]').click();
    await reviewState(viewport, "contact-detail");
    await page.goto(`${demoUrl}?route=contacts.list`);
    await page.locator('[data-contact="contact_ops"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="identity-review"]').click();
    await reviewState(viewport, "contact-identity-review");
    await page.locator('[data-contact-action="detail"]').click();
    await page.locator('[data-contact-action="back"]').click();
    await page.locator('[data-contact-action="add"]').click();
    await page.locator('[data-source-id="qr_scan"]').click();
    await reviewState(viewport, "contact-qr-native-boundary");
    await page.goto(`${demoUrl}?route=contacts.list`);
    await page.locator('[data-contact="contact_ada"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="pay"]').click();
    await reviewState(viewport, "contact-wallet-handoff");

    await page.goto(`${demoUrl}?route=wallet.swap`);
    if (viewport.width <= 768) {
      await page.locator("#mobile-menu-button").click();
      await page.locator("#mobile-popup-menu").evaluate((drawer) => (
        Promise.all(drawer.getAnimations({ subtree: true }).map((animation) => animation.finished))
      ));
    }
    const navigation = viewport.width <= 768
      ? page.locator("#mobile-popup-menu")
      : page.locator("#app-navigation-tree");
    const dapps = navigation.locator('[data-navigation-branch="dapps"]');
    await setBranchExpanded(dapps, true);
    const dappRoutes = navigation.locator('[data-navigation-branch="dapps"] + .navigation-tree-children > [data-navigation-route]');
    await expect(dappRoutes).toHaveText(dappMenuLabels);
    await expect(navigation.locator('[data-navigation-route="wallet.swap"]')).toHaveAttribute("aria-current", "page");
    const wallet = navigation.locator('[data-navigation-branch="wallet"]');
    if (await wallet.getAttribute("aria-expanded") === "true") await wallet.click();
    await dappRoutes.last().scrollIntoViewIfNeeded();
    const navigationScrollRegion = viewport.width <= 768
      ? navigation.locator(".mobile-navigation-scroll-region")
      : navigation;
    await navigationScrollRegion.evaluate((region) => {
      region.scrollTop = Math.max(0, region.scrollTop - 28);
    });
    await page.mouse.move(viewport.width - 1, 1);
    await capture(page, `${viewport.name}-phase-6-dapps-navigation`);
    if (viewport.width <= 768) await page.keyboard.press("Escape");

    await page.goto(`${demoUrl}?route=settings.general`);
    if (viewport.width <= 768) {
      await page.locator("#mobile-menu-button").click();
      await page.locator("#mobile-popup-menu").evaluate((drawer) => (
        Promise.all(drawer.getAnimations({ subtree: true }).map((animation) => animation.finished))
      ));
      const drawerBox = await page.locator("#mobile-popup-menu").boundingBox();
      expect(drawerBox.x).toBeGreaterThanOrEqual(0);
      expect(drawerBox.width).toBeCloseTo(Math.min(18.5 * 16, viewport.width - 14), 0);
    }
    const terminal = viewport.width <= 768
      ? page.locator(".mobile-navigation-terminal")
      : page.locator("#app-navigation-terminal");
    if (viewport.width <= 768) await terminal.scrollIntoViewIfNeeded();
    await expect(terminal.locator('[data-navigation-branch="settings"]')).toBeVisible();
    await expect(terminal.locator('[data-navigation-route^="settings."]')).toHaveCount(4);
    await expect(terminal.locator(":scope > .navigation-tree-terminal")).toHaveText(["Help", "About", "Log out"]);
    await expect(terminal.locator('[data-navigation-route="about"]')).toBeVisible();
    await capture(page, `${viewport.name}-phase-6-terminal-navigation`);
    if (viewport.width <= 768) await page.keyboard.press("Escape");

    await page.goto(`${demoUrl}?route=about`);
    await expect(page.locator(".about-surface a")).toHaveCount(3);
    await expect(page.locator(".about-surface > :last-child")).toHaveAttribute("data-demo-action", "check-for-updates");
    await reviewState(viewport, "about");
  }

  const auditPath = path.join(reviewRoot, "phase-6-responsive-layout-audit.json");
  await writeFile(auditPath, `${JSON.stringify(layoutAudit, null, 2)}\n`);
  const failedRoutes = layoutAudit.filter(({ issues }) => issues.length > 0);
  expect(failedRoutes, `Phase 6 responsive geometry audit failed; inspect ${auditPath}`).toEqual([]);
});

test("capture Phase 7 standalone Help on desktop and mobile", async ({ page }) => {
  await mkdir(reviewRoot, { recursive: true });
  const phaseViewports = [
    { name: "desktop-1280", width: 1280, height: 800 },
    { name: "mobile-320", width: 320, height: 800 },
  ];
  const helpAudit = [];

  const openHelp = async (trigger, viewport) => {
    const popupPromise = page.waitForEvent("popup");
    await trigger.click();
    const helpPage = await popupPromise;
    await helpPage.setViewportSize({ width: viewport.width, height: viewport.height });
    await expect(helpPage.locator("#help-document")).toBeVisible();
    return helpPage;
  };

  const reviewHelp = async (helpPage, viewport, name) => {
    await helpPage.evaluate(() => document.fonts?.ready);
    await helpPage.evaluate(() => Promise.allSettled(
      document.getAnimations()
        .filter((animation) => Number.isFinite(animation.effect?.getTiming().iterations ?? 1))
        .map((animation) => animation.finished)
    ));
    const result = await helpPage.evaluate(({ viewportName, stateName }) => {
      const rootNodes = [...document.querySelectorAll("#help-tree > [data-help-navigation-node]")];
      const terminalNodes = [...document.querySelectorAll("#help-navigation-terminal > [data-help-navigation-node]")];
      const activeLinks = [...document.querySelectorAll('#help-tree [data-help-topic-link][aria-current="page"]')];
      const articleImages = [...document.querySelectorAll("#help-document img")]
        .filter((image) => image.getBoundingClientRect().width > 0);
      const languageControlCount = document.querySelectorAll("[data-help-language-picker]").length;
      const metadataBranchCount = window.Z00ZHelpCatalog.navigations.en.items
        .filter(({ id, type }) => type === "section" && !["settings", "about"].includes(id))
        .length;
      const secondLevelBranchCount = document.querySelectorAll(
        "#help-tree .navigation-tree-children .navigation-tree-children"
      ).length;
      const expectedRootNodeCount = window.Z00ZHelpCatalog.navigations.en.items
        .filter(({ id }) => !["settings", "about"].includes(id)).length;
      const expectedTerminalNodeCount = window.Z00ZHelpCatalog.navigations.en.items
        .filter(({ id }) => ["settings", "about"].includes(id)).length;
      const excludedRootNodeCount = document.querySelectorAll(
        '#help-tree > [data-help-navigation-node="help"], '
        + '#help-tree > [data-help-navigation-node="about"], '
        + '#help-tree > [data-help-navigation-node="logout"], '
        + '#help-navigation-terminal > [data-help-navigation-node="help"], '
        + '#help-navigation-terminal > [data-help-navigation-node="logout"]'
      ).length;
      return {
        viewport: viewportName,
        state: stateName,
        documentWidth: document.documentElement.scrollWidth,
        viewportWidth: window.innerWidth,
        logoVisible: document.querySelector(".help-brand img")?.getBoundingClientRect().width > 0,
        rootNodeCount: rootNodes.length,
        terminalNodeCount: terminalNodes.length,
        nestedBranchCount: document.querySelectorAll("#help-tree .navigation-tree-children").length,
        activeLinkCount: activeLinks.length,
        articleImageCount: articleImages.length,
        languageControlCount,
        issues: [
          ...(document.documentElement.scrollWidth > window.innerWidth + 1 ? ["viewport-overflow"] : []),
          ...(rootNodes.length !== expectedRootNodeCount ? ["root-node-count"] : []),
          ...(terminalNodes.length !== expectedTerminalNodeCount ? ["terminal-node-count"] : []),
          ...(document.querySelectorAll("#help-tree .navigation-tree-children").length !== metadataBranchCount ? ["branch-tree-count"] : []),
          ...(secondLevelBranchCount ? ["second-level-navigation-present"] : []),
          ...(activeLinks.length > 1 ? ["multiple-active-links"] : []),
          ...(articleImages.length !== 1 ? ["app-view-image-count"] : []),
          ...(languageControlCount !== 1 ? ["language-control-missing"] : []),
          ...(excludedRootNodeCount ? ["excluded-navigation-present"] : []),
          ...(!document.querySelector(".help-brand img")?.getBoundingClientRect().width ? ["logo-hidden"] : []),
        ],
      };
    }, { viewportName: viewport.name, stateName: name });
    helpAudit.push(result);
    await helpPage.screenshot({
      path: path.join(reviewRoot, `${viewport.name}-phase-7-${name}.png`),
      fullPage: false,
    });
  };

  for (const viewport of phaseViewports) {
    await page.setViewportSize({ width: viewport.width, height: viewport.height });
    await page.goto(`${demoUrl}?route=dapps.discover`);
    const helpPage = await openHelp(page.locator(".context-help-button"), viewport);
    await expect(helpPage.locator("#help-tree > [data-help-navigation-node]")).toHaveCount(7);
    await expect(helpPage.locator("#help-navigation-terminal > [data-help-navigation-node]")).toHaveCount(2);
    await expect(helpPage.locator("[data-help-language-picker]")).toHaveCount(1);
    await expect(helpPage.locator('#help-tree > [data-help-navigation-node="help"], #help-tree > [data-help-navigation-node="about"], #help-tree > [data-help-navigation-node="logout"]')).toHaveCount(0);
    await expect(helpPage.locator('#help-navigation-terminal > [data-help-navigation-node="about"]')).toHaveCount(1);
    await expect(helpPage.locator('#help-navigation-terminal > [data-help-navigation-node="help"], #help-navigation-terminal > [data-help-navigation-node="logout"]')).toHaveCount(0);
    await reviewHelp(helpPage, viewport, "dapps-local-navigation");

    await helpPage.goto(new URL("help.html?topic=dapps.pay&lang=en", demoUrl).toString());
    await expect(helpPage.locator("#help-title")).toHaveText("dApps: Pay");
    await expect(helpPage.locator("#help-document")).toContainText("dApp does not control the wallet");
    await expect(helpPage.locator('[data-help-topic-link="dapps.pay"]')).toHaveAttribute("aria-current", "page");
    await reviewHelp(helpPage, viewport, "dapps-pay");

    if (viewport.width <= 768) await helpPage.locator("#help-menu-button").click();
    for (const branchId of ["wallet", "telemetry", "dapps"]) {
      const branch = helpPage.locator(`[data-help-navigation-branch="${branchId}"]`);
      if (await branch.getAttribute("aria-expanded") !== "true") await branch.click();
    }
    await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
    await expect(helpPage.locator('[data-help-navigation-branch="telemetry"]')).toHaveAttribute("aria-expanded", "true");
    await expect(helpPage.locator('[data-help-navigation-branch="dapps"]')).toHaveAttribute("aria-expanded", "true");
    await reviewHelp(helpPage, viewport, "tree-multi-open");
    await helpPage.locator('[data-help-navigation-branch="telemetry"]').click();
    await expect(helpPage.locator('[data-help-navigation-branch="telemetry"]')).toHaveAttribute("aria-expanded", "false");
    await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
    await reviewHelp(helpPage, viewport, "tree-one-of-two-closed");
    await helpPage.locator('[data-help-topic-link="wallet.assets"]').click();
    await expect(helpPage.locator("#help-title")).toHaveText("Wallet: Assets");
    await expect(helpPage.locator("#current-view")).toHaveText("App View");
    await expect(helpPage.locator('img[src="help/assets/en/wallet-assets.png"]')).toBeVisible();
    await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
    const contextualMenu = viewport.width <= 768
      ? helpPage.locator("#help-mobile-topbar-context")
      : helpPage.locator(".wallet-assets-layout > .context-rail");
    await expect(contextualMenu.locator('[data-wallet-section="assets"]')).toBeVisible();
    await expect(contextualMenu.locator('[data-wallet-section="vouchers"]')).toBeVisible();
    await expect(contextualMenu.locator('[data-help-context-topic="wallet.send"]')).toHaveCount(0);
    await reviewHelp(helpPage, viewport, "wallet-assets");

    if (viewport.width <= 768) {
      await helpPage.locator("#help-menu-button").click();
      await expect(helpPage.locator("#help-search")).toBeHidden();
      await expect(helpPage.locator(".help-mobile-menu-title")).toHaveText("Help Content");
      await expect(helpPage.locator("#help-sidebar-close, [data-mobile-popup-close]")).toHaveCount(0);
      await expect(helpPage.locator(".mobile-wallet-selector")).toHaveCount(0);
      await reviewHelp(helpPage, viewport, "app-menu-parity");
      await helpPage.keyboard.press("Escape");
      await expect(helpPage.locator("#help-sidebar")).toBeHidden();
    }
    await helpPage.locator("#help-search-trigger").click();
    await expect(helpPage.locator("#help-search-overlay")).toBeVisible();
    await helpPage.locator("#help-search").fill("Safety and limits");
    await expect(helpPage.locator(".help-search-result").first()).toBeVisible();
    await reviewHelp(helpPage, viewport, "search-results");
    await helpPage.locator("#help-search-close").click();
    await expect(helpPage.locator("#help-search-overlay")).toBeHidden();
    await helpPage.close();

    await page.goto(`${demoUrl}?route=telemetry.watchers.alerts`);
    await page.locator("[data-watcher-alert]").first().click();
    const watcherHelp = await openHelp(page.locator(".context-help-button"), viewport);
    await expect(watcherHelp.locator("#help-title")).toHaveText("Telemetry Watchers: Alert Detail");
    await reviewHelp(watcherHelp, viewport, "watcher-alert-detail");
    await watcherHelp.close();

    await page.goto(`${demoUrl}?route=contacts.list`);
    await page.locator('[data-contact="contact_ops"] [data-contact-action="open"]').click();
    await page.locator('[data-contact-action="identity-review"]').click();
    const contactHelp = await openHelp(page.locator(".context-help-button"), viewport);
    await expect(contactHelp.locator("#help-title")).toHaveText("Contacts: Identity Review");
    await reviewHelp(contactHelp, viewport, "contact-identity-review");
    await contactHelp.close();
  }

  const auditPath = path.join(reviewRoot, "phase-7-help-responsive-audit.json");
  await writeFile(auditPath, `${JSON.stringify(helpAudit, null, 2)}\n`);
  expect(helpAudit.filter(({ issues }) => issues.length), `Phase 7 Help audit failed; inspect ${auditPath}`).toEqual([]);
});

test("capture annotated mobile wallet and Help corrections", async ({ page }) => {
  await mkdir(reviewRoot, { recursive: true });
  await page.setViewportSize({ width: 390, height: 844 });
  const audit = [];

  await page.goto(`${demoUrl}?route=wallet.settings.general`);
  const settings = await page.locator(".wallet-settings-view").evaluate((view) => {
    const rows = ["wallet-name", "wallet-id", "wallet-chain"].map((anchor) => {
      const row = view.querySelector(`[data-help-anchor="${anchor}"]`);
      const value = row.querySelector(".compact-value").getBoundingClientRect();
      return { anchor, valueLeft: Math.round(value.left * 10) / 10 };
    });
    const valueLefts = rows.map(({ valueLeft }) => valueLeft);
    return {
      rows,
      issues: Math.max(...valueLefts) - Math.min(...valueLefts) > 1
        ? ["wallet-setting-values-misaligned"]
        : [],
    };
  });
  audit.push({ state: "wallet-settings-general", ...settings });
  await capture(page, "mobile-390-annotated-wallet-settings-general");

  await page.goto(`${demoUrl}?route=telemetry.reticulum.radio`);
  const topbar = await page.evaluate(() => {
    const topbarBackground = getComputedStyle(document.querySelector(".topbar")).backgroundColor;
    const walletBackground = getComputedStyle(document.querySelector("#mobile-active-wallet")).backgroundColor;
    return {
      topbarBackground,
      walletBackground,
      issues: walletBackground === topbarBackground ? [] : ["mobile-wallet-background-mismatch"],
    };
  });
  audit.push({ state: "telemetry-mobile-topbar", ...topbar });
  await capture(page, "mobile-390-annotated-telemetry-topbar");

  const openWalletSheet = async (action, expectedTitle, screenshotName) => {
    await page.goto(`${demoUrl}?route=wallet.assets`);
    await expect(page.locator("#mobile-active-wallet")).toBeVisible();
    await page.locator("#mobile-menu-button").click();
    const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
    await expect(drawer).toBeVisible();
    await drawer.locator("[data-wallet-picker-trigger]").click();
    await page.locator(`#wallet-picker-popup [data-wallet-picker-action="${action}"]`).click();
    const sheet = page.locator("#flow-dialog");
    await expect(sheet).toBeVisible();
    await expect(sheet.locator("#dialog-title")).toHaveText(expectedTitle);
    const result = await sheet.evaluate((element) => {
      const rect = element.getBoundingClientRect();
      const copy = element.textContent.replace(/\s+/g, " ").trim();
      return {
        top: Math.round(rect.top),
        bottom: Math.round(rect.bottom),
        viewportHeight: window.innerHeight,
        mentionsProfiles: /\bprofiles?\b/i.test(copy),
        issues: [
          ...(rect.top > 29 ? ["sheet-too-low"] : []),
          ...(rect.bottom < window.innerHeight - 1 ? ["sheet-not-bottom-aligned"] : []),
        ],
      };
    });
    if (action === "remove-wallet" && result.mentionsProfiles) result.issues.push("remove-profile-copy");
    audit.push({ state: action, ...result });
    await page.screenshot({
      path: path.join(reviewRoot, `${screenshotName}.png`),
      fullPage: false,
    });
    await page.keyboard.press("Escape");
  };

  await openWalletSheet("add-wallet", "Add wallet", "mobile-390-annotated-add-wallet");
  await openWalletSheet("remove-wallet", "Remove Wallet(s)", "mobile-390-annotated-remove-wallets");

  await page.goto(new URL("help.html?topic=wallet.history&lang=en&section=current-view", demoUrl).toString());
  await expect(page.locator("#current-view")).toBeFocused();
  const help = await page.evaluate(() => {
    const backgrounds = [
      document.body,
      document.querySelector(".help-site-header"),
      document.querySelector(".help-header-controls"),
      document.querySelector(".help-header-language"),
    ].map((element) => getComputedStyle(element).backgroundColor);
    const outlineWidth = getComputedStyle(document.querySelector("#current-view")).outlineWidth;
    return {
      backgrounds,
      outlineWidth,
      issues: [
        ...(new Set(backgrounds).size !== 1 ? ["help-background-mismatch"] : []),
        ...(outlineWidth !== "0px" ? ["app-view-frame-present"] : []),
      ],
    };
  });
  audit.push({ state: "help-wallet-history", ...help });
  await page.screenshot({
    path: path.join(reviewRoot, "mobile-390-annotated-help-wallet-history.png"),
    fullPage: false,
  });

  const auditPath = path.join(reviewRoot, "annotated-mobile-corrections-audit.json");
  await writeFile(auditPath, `${JSON.stringify(audit, null, 2)}\n`);
  expect(audit.filter(({ issues }) => issues.length), `Annotated mobile audit failed; inspect ${auditPath}`).toEqual([]);
});
