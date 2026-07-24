const { mkdir, readFile, writeFile } = require("node:fs/promises");
const path = require("node:path");
const vm = require("node:vm");
const { test, expect } = require("@playwright/test");

const demoUrl = process.env.Z00Z_WALLET_DEMO_URL;
const reviewRoot = path.resolve(
  __dirname,
  "../../z00z_storage/outputs/checkpoint/phase-110/ui-help-review",
);

const viewports = [
  { name: "desktop", width: 1280, height: 800 },
  { name: "mobile-390", width: 390, height: 844 },
  { name: "mobile-320", width: 320, height: 800 },
];

const desktopReviewRoutes = [
  { name: "wallet-general", query: "?view=wallet-settings&walletSettings=general" },
  { name: "wallet-security", query: "?view=wallet-settings&walletSettings=security" },
  { name: "wallet-backup", query: "?view=wallet-settings&walletSettings=backup" },
  { name: "wallet-policies", query: "?view=wallet-settings&walletSettings=policies" },
  { name: "wallet-advanced", query: "?view=wallet-settings&walletSettings=advanced" },
  { name: "app-general", query: "?view=settings&settings=general" },
  { name: "app-reticulum", query: "?view=settings&settings=reticulum" },
  { name: "app-onionnet", query: "?view=settings&settings=onionnet" },
  { name: "app-appearance", query: "?view=settings&settings=appearance" },
  { name: "assets", query: "?view=wallet&wallet=assets" },
  { name: "send", query: "?view=wallet-send" },
  { name: "swap", query: "?view=swap" },
  { name: "staking", query: "?view=staking" },
  { name: "backup", query: "?view=wallet-backup" },
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

function allMobileRoutes(contract) {
  return contract.views.flatMap((view) => {
    if (view === "wallet") {
      return contract.walletSections.map((walletSection) => ({
        name: `wallet-${walletSection}`,
        query: routeQuery({ view, wallet: walletSection }),
      }));
    }
    if (view === "wallet-settings") {
      return contract.walletSettingsSections.map((walletSettingsSection) => ({
        name: `wallet-settings-${walletSettingsSection}`,
        query: routeQuery({ view, walletSettings: walletSettingsSection }),
      }));
    }
    if (view === "settings") {
      return contract.settingsSections.map((settingsSection) => ({
        name: `settings-${settingsSection}`,
        query: routeQuery({ view, settings: settingsSection }),
      }));
    }
    if (view === "telemetry") {
      return contract.telemetrySources.flatMap((telemetrySource) => (
        contract.telemetryTabs[telemetrySource].map((tab) => ({
          name: `telemetry-${telemetrySource}-${tab}`,
          query: routeQuery({
            view,
            telemetry: telemetrySource,
            [`${telemetrySource === "onionnet" ? "onion" : telemetrySource}Tab`]: tab,
          }),
        }))
      ));
    }
    return [{ name: view, query: routeQuery({ view }) }];
  });
}

async function capture(page, name, { fullPage = false } = {}) {
  await expect(page.locator("#main-content")).toBeVisible();
  await page.screenshot({
    path: path.join(reviewRoot, `${name}.png`),
    fullPage,
  });
}

async function auditMobileGeometry(page, viewport, route) {
  await page.evaluate(() => document.fonts?.ready);
  await page.waitForTimeout(40);

  return page.evaluate(({ viewportName, routeName }) => {
    const tolerance = 1;
    const ignoredOverflowHost = ".wallet-tabs, .choice-strip, .filter-bar, .context-rail, .yaml-editor, .help-contents";
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

    const activeTab = document.querySelector("#wallet-tabs .wallet-tab.is-active");
    const tabStrip = document.querySelector("#wallet-tabs");
    if (activeTab && tabStrip) {
      const activeRect = activeTab.getBoundingClientRect();
      const stripRect = tabStrip.getBoundingClientRect();
      if (activeRect.left < stripRect.left - tolerance || activeRect.right > stripRect.right + tolerance) {
        issues.push({
          type: "active-tab-clipped",
          element: elementName(activeTab),
          activeRect: roundedRect(activeRect),
          stripRect: roundedRect(stripRect),
        });
      }
    }

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

test("capture multilingual Help and compact-layout review matrix", async ({ page }) => {
  await mkdir(reviewRoot, { recursive: true });
  const mobileRoutes = allMobileRoutes(await loadPortContract());
  const mobileAudit = [];

  for (const viewport of viewports) {
    await page.setViewportSize(viewport);
    const routes = viewport.width > 390 ? desktopReviewRoutes : mobileRoutes;
    for (const route of routes) {
      await page.goto(`${demoUrl}${route.query}`);
      if (viewport.width <= 390) {
        mobileAudit.push(await auditMobileGeometry(page, viewport, route));
      }
      await capture(page, `${viewport.name}-${route.name}`, { fullPage: viewport.width <= 390 });
    }

    if (viewport.width > 390) {
      await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
      for (const walletId of ["everyday", "savings", "travel"]) {
        await page.locator(`[data-wallet-id="${walletId}"]`).click();
        await capture(page, `${viewport.name}-wallet-header-${walletId}`);
      }
    }

    await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
    await page.locator(".asset-identity-button").first().click();
    await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
    await capture(page, `${viewport.name}-asset-details`);
    await page.locator(".dialog-help-button").click();
    await expect(page.locator("#help-title")).toHaveText("Asset details");
    await capture(page, `${viewport.name}-asset-details-help`);
    await page.keyboard.press("Escape");
    await page.keyboard.press("Escape");

    await page.getByRole("button", { name: "Help for this view" }).click();
    await expect(page.locator(".help-panel")).toBeVisible();
    await capture(page, `${viewport.name}-context-help`);
    await page.keyboard.press("Escape");

    if (viewport.width <= 390) {
      await page.locator("#mobile-menu-button").click();
      await page.getByRole("button", { name: "Help", exact: true }).click();
    } else {
      await page.locator(".help-button").click();
    }
    await expect(page.locator(".help-panel")).toBeVisible();
    await expect(page.locator("#help-title")).toHaveText("Application help");
    await capture(page, `${viewport.name}-global-help`);
    await page.keyboard.press("Escape");

    if (viewport.width <= 390) {
      await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
      await page.locator("#mobile-menu-button").click();
      await page.locator('[data-mobile-popup-open="wallets"]').click();
      await expect(page.locator(".mobile-wallet-actions")).toBeVisible();
      await capture(page, `${viewport.name}-wallets-menu`);
      await page.keyboard.press("Escape");
    }

    if (viewport.width === 320) {
      for (const locale of ["ru", "de", "ja", "ko", "zh-Hans"]) {
        await page.goto(`${demoUrl}?view=settings&settings=general`);
        await page.locator('[data-config-control="language"]').selectOption(locale);
        await capture(page, `${viewport.name}-locale-${locale}`);
        await page.locator(".context-help-button").click();
        await expect(page.locator(".help-panel")).toBeVisible();
        await capture(page, `${viewport.name}-locale-${locale}-context-help`);
        await page.keyboard.press("Escape");
      }
    }
  }

  const auditPath = path.join(reviewRoot, "mobile-layout-audit.json");
  await writeFile(auditPath, `${JSON.stringify(mobileAudit, null, 2)}\n`);
  const failedRoutes = mobileAudit.filter(({ issues }) => issues.length > 0);
  expect(failedRoutes, `Mobile geometry audit failed; inspect ${auditPath}`).toEqual([]);
});
