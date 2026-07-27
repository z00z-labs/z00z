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

async function selectAppLanguage(page, languageId) {
  await page.locator("[data-language-picker-trigger]").click();
  const option = page.locator(`[data-language-picker-option="${languageId}"]`);
  await expect(option).toHaveCount(1);
  await option.click();
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

async function mobileSwipe(page, { from, to, source = "pointer" }) {
  await page.evaluate(({ from, to, source }) => {
    const targetAt = (point) => document.elementFromPoint(point.x, point.y) || document.body;
    const points = [0.25, 0.5, 0.75].map((progress) => ({
      x: from.x + (to.x - from.x) * progress,
      y: from.y + (to.y - from.y) * progress,
    }));
    if (source === "touch") {
      const target = targetAt(from);
      const makeTouch = (point, target) => new Touch({
        identifier: 71,
        target,
        clientX: point.x,
        clientY: point.y,
      });
      const dispatchTouch = (type, point, active) => {
        const changedTouch = makeTouch(point, target);
        const activeTouches = active ? [changedTouch] : [];
        target.dispatchEvent(new TouchEvent(type, {
          bubbles: true,
          cancelable: true,
          composed: true,
          touches: activeTouches,
          targetTouches: activeTouches,
          changedTouches: [changedTouch],
        }));
      };
      dispatchTouch("touchstart", from, true);
      points.forEach((point) => dispatchTouch("touchmove", point, true));
      dispatchTouch("touchend", to, false);
      return;
    }

    const eventInit = (point, buttons) => ({
      bubbles: true,
      cancelable: true,
      composed: true,
      pointerId: 71,
      pointerType: "touch",
      isPrimary: true,
      button: 0,
      buttons,
      clientX: point.x,
      clientY: point.y,
    });
    targetAt(from).dispatchEvent(new PointerEvent("pointerdown", eventInit(from, 1)));
    points.forEach((point) => targetAt(point).dispatchEvent(new PointerEvent("pointermove", eventInit(point, 1))));
    targetAt(to).dispatchEvent(new PointerEvent("pointerup", eventInit(to, 0)));
  }, { from, to, source });
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
  expect(walletPlaceholderGeometry.labelTop - walletPlaceholderGeometry.topbarBottom).toBeCloseTo(0, 0);

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
  await expect(page.locator("#wallet-identity .environment-tag.is-main")).toHaveText("Mainnet");
  await expect(page.locator("#page-title")).toHaveText("Reticulum");
  await expect(page.locator("#page-context")).toBeHidden();
  const desktopTopbarOrder = await page.evaluate(() => {
    const brand = document.querySelector(".desktop-topbar-brand").getBoundingClientRect();
    const logo = document.querySelector(".desktop-topbar-brand .brand-mark").getBoundingClientRect();
    const brandWordmark = document.querySelector(".desktop-topbar-brand > span").getBoundingClientRect();
    const wallet = document.querySelector("#wallet-identity").getBoundingClientRect();
    const heading = document.querySelector(".topbar-address-group").getBoundingClientRect();
    const address = document.querySelector(".wallet-identity-address");
    const copy = document.querySelector(".wallet-identity-copy").getBoundingClientRect();
    const networkBadge = document.querySelector("#wallet-identity .environment-tag").getBoundingClientRect();
    const topbarStyle = getComputedStyle(document.querySelector(".topbar"));
    const addressStyle = getComputedStyle(address);
    return {
      brand: { left: brand.left, right: brand.right, center: brand.left + brand.width / 2 },
      logo: { left: logo.left, width: logo.width, height: logo.height, centerY: logo.top + logo.height / 2 },
      brandGroupCenter: (logo.left + brandWordmark.right) / 2,
      wallet: { left: wallet.left, right: wallet.right },
      heading: { left: heading.left },
      walletIdentityWidth: wallet.width,
      walletAddress: {
        fontFamily: addressStyle.fontFamily,
        fontSize: addressStyle.fontSize,
        fontWeight: addressStyle.fontWeight,
        lineHeight: addressStyle.lineHeight,
      },
      copy: { right: copy.right, width: copy.width, height: copy.height },
      networkBadge: { right: networkBadge.right, width: networkBadge.width, height: networkBadge.height },
      topbar: { centerY: brand.top + brand.height / 2, borderBottomWidth: topbarStyle.borderBottomWidth },
    };
  });
  expect(desktopTopbarOrder.wallet.left).toBeGreaterThanOrEqual(desktopTopbarOrder.brand.right - 1);
  expect(desktopTopbarOrder.heading.left).toBeGreaterThanOrEqual(desktopTopbarOrder.wallet.right - 1);
  expect(desktopTopbarOrder.logo).toMatchObject({ width: 52, height: 52 });
  expect(Math.abs(desktopTopbarOrder.brandGroupCenter - desktopTopbarOrder.brand.center)).toBeLessThanOrEqual(1);
  expect(desktopTopbarOrder.logo.centerY).toBeCloseTo(desktopTopbarOrder.topbar.centerY, 0);
  expect(desktopTopbarOrder.walletIdentityWidth).toBe(262);
  expect(desktopTopbarOrder.walletAddress.fontFamily).toContain("Geist");
  expect(desktopTopbarOrder.walletAddress.fontFamily).not.toContain("Geist Mono");
  expect(desktopTopbarOrder.walletAddress).toMatchObject({ fontSize: "21px", fontWeight: "400", lineHeight: "21.84px" });
  expect(desktopTopbarOrder.copy).toMatchObject({ width: 26, height: 26 });
  expect(desktopTopbarOrder.networkBadge.width).toBeGreaterThanOrEqual(66);
  expect(desktopTopbarOrder.networkBadge.height).toBeGreaterThanOrEqual(22);
  expect(desktopTopbarOrder.networkBadge.right).toBeCloseTo(desktopTopbarOrder.copy.right, 0);
  expect(desktopTopbarOrder.topbar.borderBottomWidth).toBe("0px");

  await page.setViewportSize({ width: 1280, height: 520 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const desktopSidebarScroll = await page.evaluate(() => {
    const sidebar = document.querySelector(".app-body > .sidebar");
    const walletViewport = document.querySelector(".wallet-nav-viewport");
    const walletTrigger = document.querySelector("#wallet-nav [data-wallet-picker-trigger]");
    const scrollRegion = document.querySelector(".sidebar-navigation-scroll-region");
    const terminal = document.querySelector("#app-navigation-terminal");
    const tree = document.querySelector("#app-navigation-tree");
    const walletTopBefore = walletViewport.getBoundingClientRect().top;
    const terminalTopBefore = terminal.getBoundingClientRect().top;
    scrollRegion.scrollTop = scrollRegion.scrollHeight;
    return {
      sidebarOverflow: getComputedStyle(sidebar).overflowY,
      walletOverflow: getComputedStyle(walletViewport).overflowY,
      scrollRegionOverflow: getComputedStyle(scrollRegion).overflowY,
      treeOverflow: getComputedStyle(tree).overflowY,
      walletIsOutsideScrollRegion: walletViewport.parentElement === sidebar,
      triggerIsInsideWalletPlaceholder: walletTrigger.parentElement === walletViewport.firstElementChild,
      treeIsInsideScrollRegion: tree.parentElement === scrollRegion,
      terminalIsInsideScrollRegion: terminal.parentElement === scrollRegion,
      scrollRegionScrollTop: scrollRegion.scrollTop,
      walletTopBefore,
      walletTopAfter: walletViewport.getBoundingClientRect().top,
      terminalTopBefore,
      terminalTopAfter: terminal.getBoundingClientRect().top,
    };
  });
  expect(desktopSidebarScroll.sidebarOverflow).toBe("hidden");
  expect(desktopSidebarScroll.walletOverflow).toBe("visible");
  expect(desktopSidebarScroll.scrollRegionOverflow).toBe("auto");
  expect(desktopSidebarScroll.treeOverflow).toBe("visible");
  expect(desktopSidebarScroll.walletIsOutsideScrollRegion).toBe(true);
  expect(desktopSidebarScroll.triggerIsInsideWalletPlaceholder).toBe(true);
  expect(desktopSidebarScroll.treeIsInsideScrollRegion).toBe(true);
  expect(desktopSidebarScroll.terminalIsInsideScrollRegion).toBe(true);
  expect(desktopSidebarScroll.scrollRegionScrollTop).toBeGreaterThan(0);
  expect(desktopSidebarScroll.walletTopAfter).toBeCloseTo(desktopSidebarScroll.walletTopBefore, 0);
  expect(desktopSidebarScroll.terminalTopAfter).toBeLessThan(desktopSidebarScroll.terminalTopBefore);
});

test("topbar menu search uses localized canonical navigation on desktop and mobile", async ({ page }) => {
  const languageIds = ["en", "ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"];

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=settings.general`);

  const actionOrder = await page.evaluate(() => {
    const search = document.querySelector("#menu-search-trigger").getBoundingClientRect();
    const eye = document.querySelector('[data-demo-action="toggle-balance"]').getBoundingClientRect();
    return {
      searchIsBeforeEye: search.right <= eye.left,
      searchWidth: Math.round(search.width),
      eyeWidth: Math.round(eye.width),
    };
  });
  expect(actionOrder).toEqual({ searchIsBeforeEye: true, searchWidth: 44, eyeWidth: 44 });

  await page.locator("#menu-search-trigger").click();
  const appSearchStyle = await page.evaluate(() => {
    const dialog = document.querySelector("#menu-search-dialog");
    const header = dialog.querySelector(".menu-search-dialog-header");
    const backdrop = document.querySelector("#menu-search-backdrop");
    const dialogStyle = getComputedStyle(dialog);
    return {
      backdropFilter: getComputedStyle(backdrop).backdropFilter,
      background: dialogStyle.backgroundColor,
      borderColor: dialogStyle.borderColor,
      borderRadius: dialogStyle.borderRadius,
      headerPadding: getComputedStyle(header).padding,
      width: Math.round(dialog.getBoundingClientRect().width),
    };
  });
  const helpPage = await page.context().newPage();
  await helpPage.goto(demoUrl.replace("index.html", "help.html?topic=wallet.assets"));
  await helpPage.keyboard.press("Control+K");
  const helpSearchStyle = await helpPage.evaluate(() => {
    const dialog = document.querySelector("#help-search-dialog");
    const header = dialog.querySelector(".help-search-dialog-header");
    const backdrop = document.querySelector("#help-search-backdrop");
    const dialogStyle = getComputedStyle(dialog);
    return {
      backdropFilter: getComputedStyle(backdrop).backdropFilter,
      background: dialogStyle.backgroundColor,
      borderColor: dialogStyle.borderColor,
      borderRadius: dialogStyle.borderRadius,
      headerPadding: getComputedStyle(header).padding,
      width: Math.round(dialog.getBoundingClientRect().width),
    };
  });
  expect(appSearchStyle).toEqual(helpSearchStyle);
  await helpPage.close();
  await page.keyboard.press("Escape");

  for (const languageId of languageIds) {
    await selectAppLanguage(page, languageId);
    const labels = await page.evaluate((id) => ({
      assets: window.Z00ZI18n.translate(id, "navigation.assets"),
      search: window.Z00ZI18n.translate(id, "navigation.search"),
    }), languageId);
    await expect(page.locator("#menu-search-trigger")).toHaveAttribute("aria-label", labels.search);
    await page.locator("#menu-search-trigger").click();
    await expect(page.locator("#menu-search-overlay")).toBeVisible();
    await expect(page.locator("#menu-search-input")).toBeFocused();
    await page.locator("#menu-search-input").fill(labels.assets);
    await expect(page.locator('[data-menu-search-node="wallet.assets-rights"]')).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(page.locator("#menu-search-overlay")).toBeHidden();
    await expect(page.locator("#menu-search-trigger")).toBeFocused();
  }

  await page.locator("#menu-search-trigger").click();
  const permissionsLabel = await page.evaluate(() => window.Z00ZI18n.translate("zh-Hans", "navigation.permissions"));
  await page.locator("#menu-search-input").fill(permissionsLabel);
  await page.locator('[data-menu-search-node="wallet.permissions"]').click();
  await expect(page).toHaveURL(/route=wallet\.permissions/);
  await expect(page.locator("#page-title")).toHaveText(permissionsLabel);
  await expect(page.locator("#menu-search-overlay")).toBeHidden();

  await page.keyboard.press("Control+K");
  await expect(page.locator("#menu-search-input")).toBeFocused();
  await page.keyboard.press("Escape");

  await page.setViewportSize({ width: 390, height: 844 });
  await page.keyboard.press("Control+K");
  await expect(page.locator("#menu-search-overlay")).toBeVisible();
  const mobileGeometry = await page.locator("#menu-search-dialog").evaluate((element) => {
    const rect = element.getBoundingClientRect();
    return {
      left: Math.round(rect.left),
      right: Math.round(rect.right),
      top: Math.round(rect.top),
      bottom: Math.round(rect.bottom),
      pathDisplay: getComputedStyle(document.querySelector(".menu-search-result-path")).display,
    };
  });
  expect(mobileGeometry.left).toBeGreaterThanOrEqual(0);
  expect(mobileGeometry.right).toBeLessThanOrEqual(390);
  expect(mobileGeometry.top).toBeGreaterThanOrEqual(0);
  expect(mobileGeometry.bottom).toBeLessThanOrEqual(844);
  expect(mobileGeometry.pathDisplay).toBe("none");
  await page.keyboard.press("Escape");
});

test("wallet picker is the only anchored selector on desktop and mobile", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 390, height: 844, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    if (viewport.mobile) {
      await expect(page.locator("#mobile-active-wallet [data-wallet-picker-trigger]")).toHaveCount(0);
      await page.locator("#mobile-menu-button").click();
      await expect(page.locator("#mobile-popup-menu .mobile-wallet-selector")).toBeVisible();
      const mobileTrigger = page.locator("#mobile-popup-menu [data-wallet-picker-trigger]");
      await expect(mobileTrigger).toHaveCount(1);
      await mobileTrigger.click();
      await expect(page.locator("#wallet-picker-popup")).toBeVisible();
      await expect(page.locator("#mobile-popup-menu")).toBeVisible();
      await expect(page.locator("#wallet-picker-popup")).toHaveAttribute("role", "menu");
      await page.waitForFunction(() => document.querySelector("#wallet-picker-popup").style.top !== "");
      await expect(page.locator("#wallet-picker-popup .wallet-picker-choice")).toHaveCount(3);
      await expect(page.locator("#wallet-picker-popup .wallet-picker-actions [data-wallet-picker-action]")).toHaveCount(2);
      const mobilePickerGeometry = await page.evaluate(() => {
        const trigger = document.querySelector("#mobile-popup-menu [data-wallet-picker-trigger]").getBoundingClientRect();
        const menu = document.querySelector("#wallet-picker-popup").getBoundingClientRect();
        return { offsetX: Math.round(menu.left - trigger.left), offsetY: Math.round(menu.top - trigger.bottom) };
      });
      expect(mobilePickerGeometry).toEqual({ offsetX: 0, offsetY: 8 });
      await page.locator('#wallet-picker-popup [data-wallet-picker-id="savings"]').click();
      await expect(page.locator("#mobile-popup-menu .mobile-wallet-picker-trigger")).toContainText("Savings");
      await expect(page.locator("#wallet-picker-popup")).toBeHidden();
      await expect(page.locator("#mobile-popup-menu")).toBeVisible();
      await page.locator("#mobile-popup-menu [data-wallet-picker-trigger]").click();
      await page.locator('#wallet-picker-popup [data-wallet-picker-action="remove-wallet"]').click();
      await expect(page.locator("#dialog-title")).toHaveText("Remove Wallet(s)");
      await page.keyboard.press("Escape");
    } else {
      await expect(page.locator(".sidebar-label")).toBeVisible();
      await expect(page.locator(".wallet-nav-viewport")).toBeVisible();
      await expect(page.locator("#wallet-identity [data-wallet-picker-trigger]")).toHaveCount(0);
      await expect(page.locator("#wallet-nav [data-wallet-id]")).toHaveCount(0);
      await expect(page.locator("#wallet-nav [data-wallet-picker-trigger]")).toHaveCount(1);
      await page.locator("#wallet-nav [data-wallet-picker-trigger]").click();
      await expect(page.locator("#wallet-picker-popup")).toBeVisible();
      await expect(page.locator("#wallet-picker-popup")).toHaveAttribute("role", "menu");
      await page.waitForFunction(() => document.querySelector("#wallet-picker-popup").style.top !== "");
      await expect(page.locator("#wallet-picker-popup .wallet-picker-choice")).toHaveCount(3);
      await expect(page.locator("#wallet-picker-popup .wallet-picker-actions [data-wallet-picker-action]")).toHaveCount(2);
      const desktopPickerGeometry = await page.evaluate(() => {
        const trigger = document.querySelector("#wallet-nav [data-wallet-picker-trigger]").getBoundingClientRect();
        const menu = document.querySelector("#wallet-picker-popup").getBoundingClientRect();
        return { offsetX: Math.round(menu.left - trigger.left), offsetY: Math.round(menu.top - trigger.bottom) };
      });
      expect(desktopPickerGeometry).toEqual({ offsetX: 0, offsetY: 8 });
      await page.locator('#wallet-picker-popup [data-wallet-picker-id="savings"]').click();
      await expect(page.locator("#wallet-nav [data-wallet-picker-trigger]")).toContainText("Savings");
      await expect(page.locator("#wallet-picker-popup")).toBeHidden();
      await page.locator("#wallet-nav [data-wallet-picker-trigger]").click();
      await page.locator('#wallet-picker-popup [data-wallet-picker-action="add-wallet"]').click();
      await expect(page.locator("#dialog-title")).toHaveText("Add wallet");
      await page.keyboard.press("Escape");
    }
  }
});

test("removing every wallet leaves only Add wallet on desktop and mobile", async ({ page }) => {
  for (const viewport of [
    { width: 1280, height: 800, mobile: false },
    { width: 390, height: 844, mobile: true },
  ]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?route=wallet.assets`);
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    const selector = viewport.mobile ? "#mobile-popup-menu" : "#wallet-nav";
    await page.locator(`${selector} [data-wallet-picker-trigger]`).click();
    await page.locator('#wallet-picker-popup [data-wallet-picker-action="remove-wallet"]').click();
    const walletCheckboxes = page.locator("[data-remove-wallet-id]");
    for (let index = 0; index < await walletCheckboxes.count(); index += 1) {
      await walletCheckboxes.nth(index).check();
    }
    await page.locator('[data-dialog-action="confirm-remove-wallet"]').click();
    await expect(page.locator("#dialog-title")).toHaveText("Add wallet");
    await page.locator("#flow-dialog [data-dialog-close]").first().click();
    await expect(page.locator("#flow-dialog")).toBeHidden();

    if (viewport.mobile) {
      await page.locator("#mobile-menu-button").click();
      await expect(page.locator('#mobile-popup-menu[data-popup-type="menu"]')).toBeVisible();
      const mobileSelector = page.locator("#mobile-popup-menu .mobile-wallet-selector");
      await expect(mobileSelector.locator('[data-wallet-picker-trigger]')).toHaveCount(0);
      await expect(mobileSelector.locator('[data-wallet-picker-action="remove-wallet"]')).toHaveCount(0);
      await expect(mobileSelector.locator('[data-wallet-picker-action="add-wallet"]')).toHaveCount(1);
      await mobileSelector.locator('[data-wallet-picker-action="add-wallet"]').click();
    } else {
      await expect(page.locator("#wallet-nav [data-wallet-picker-trigger]")).toHaveCount(0);
      await expect(page.locator('#wallet-nav [data-wallet-picker-action="remove-wallet"]')).toHaveCount(0);
      await expect(page.locator('#wallet-nav [data-wallet-picker-action="add-wallet"]')).toHaveCount(1);
      await page.locator('#wallet-nav [data-wallet-picker-action="add-wallet"]').click();
    }
    await expect(page.locator("#dialog-title")).toHaveText("Add wallet");
    await page.keyboard.press("Escape");
  }
});

test("mobile wallet management sheets rise to the safe top and use wallet wording", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });

  const openSheet = async (action) => {
    await page.goto(`${demoUrl}?route=wallet.assets`);
    await page.locator("#mobile-menu-button").click();
    await page.locator("#mobile-popup-menu [data-wallet-picker-trigger]").click();
    await page.locator(`#wallet-picker-popup [data-wallet-picker-action="${action}"]`).click();
    const sheet = page.locator("#flow-dialog");
    await expect(sheet).toBeVisible();
    await expect(sheet).toHaveAttribute("data-flow-type", action);
    const geometry = await sheet.evaluate((element) => {
      const rect = element.getBoundingClientRect();
      return {
        top: Math.round(rect.top),
        bottom: Math.round(rect.bottom),
        viewportHeight: window.innerHeight,
      };
    });
    expect(geometry.top).toBeLessThanOrEqual(29);
    expect(geometry.bottom).toBeGreaterThanOrEqual(geometry.viewportHeight - 1);
    return sheet;
  };

  const addSheet = await openSheet("add-wallet");
  await expect(addSheet.locator("#dialog-title")).toHaveText("Add wallet");
  await expect(addSheet).toContainText("Create, open, or restore a local wallet");
  await page.keyboard.press("Escape");

  const removeSheet = await openSheet("remove-wallet");
  await expect(removeSheet.locator("#dialog-title")).toHaveText("Remove Wallet(s)");
  await expect(removeSheet).toContainText("Remove local wallets from this concept");
  await expect(removeSheet).toContainText("This removes local wallets only");
  await expect(removeSheet.getByRole("button", { name: "Remove Wallet(s)" })).toBeVisible();
  await expect(removeSheet).not.toContainText(/profiles/i);
  await page.keyboard.press("Escape");
});

test("wallet selectors derive their marker colours from the wallet chain", async ({ page }) => {
  const expectedDefaults = [
    { wallet: "everyday", chain: "mainnet", tone: "is-main" },
    { wallet: "savings", chain: "mainnet", tone: "is-main" },
    { wallet: "travel", chain: "mainnet", tone: "is-main" },
  ];
  const readMarkers = (selector) => page.locator(selector).evaluateAll((nodes) => nodes.map((node) => ({
    wallet: node.dataset.walletPickerId,
    chain: node.dataset.walletChain,
    tone: [...node.querySelector(".wallet-nav-state").classList].find((name) => name.startsWith("is-")),
    colour: getComputedStyle(node.querySelector(".wallet-nav-state")).backgroundColor,
  })));
  const createWallet = async (name, chainId) => {
    await page.locator('#wallet-nav [data-wallet-picker-trigger]').click();
    await page.locator('#wallet-picker-popup [data-wallet-picker-action="add-wallet"]').click();
    await page.locator('#flow-dialog [data-demo-action="create-wallet"]').click();
    await page.locator("#create-name").fill(name);
    await page.locator("#create-chain").selectOption(chainId);
    await page.locator("#create-password").fill("demonstration-passphrase");
    await page.locator("#create-confirm").fill("demonstration-passphrase");
    await page.locator('button[form="create-wallet-entry"]').click();
    const seedWords = await page.locator(".seed-grid li strong").allTextContents();
    await page.locator('#flow-dialog [data-dialog-action="create-seed-saved"]').click();
    const verificationIndexes = await page.locator("#create-wallet-verify select[data-seed-index]").evaluateAll((selects) => selects.map((select) => Number(select.dataset.seedIndex)));
    for (const [index, seedIndex] of verificationIndexes.entries()) {
      await page.locator("#create-wallet-verify select[data-seed-index]").nth(index).selectOption(seedWords[seedIndex]);
    }
    await page.locator('button[form="create-wallet-verify"]').click();
    await page.locator('#flow-dialog [data-dialog-action="create-finish"]').click();
  };

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  await page.locator('#wallet-nav [data-wallet-picker-trigger]').click();
  const defaultMarkers = await readMarkers("#wallet-picker-popup .wallet-picker-choice");
  expect(defaultMarkers.map(({ wallet, chain, tone }) => ({ wallet, chain, tone }))).toEqual(expectedDefaults);
  expect(new Set(defaultMarkers.map(({ colour }) => colour)).size).toBe(1);
  await page.keyboard.press("Escape");

  await createWallet("Test network", "testnet-1");
  await createWallet("Development network", "devnet-1");
  await page.locator('#wallet-nav [data-wallet-picker-trigger]').click();
  const desktop = await readMarkers("#wallet-picker-popup .wallet-picker-choice");
  const expected = [
    ...expectedDefaults,
    { wallet: "wallet-4", chain: "testnet-1", tone: "is-test" },
    { wallet: "wallet-5", chain: "devnet-1", tone: "is-dev" },
  ];
  expect(desktop.map(({ wallet, chain, tone }) => ({ wallet, chain, tone }))).toEqual(expected);
  expect(new Set(desktop.map(({ colour }) => colour)).size).toBe(3);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.locator("#mobile-menu-button").click();
  await page.locator('#mobile-popup-menu [data-wallet-picker-trigger]').click();
  const mobile = await readMarkers("#wallet-picker-popup .wallet-picker-choice");
  expect(mobile.map(({ wallet, chain, tone }) => ({ wallet, chain, tone }))).toEqual(expected);
  expect(mobile.map(({ colour }) => colour)).toEqual(desktop.map(({ colour }) => colour));

  const componentsCss = await readFile(path.join(demoDir, "styles/components.css"), "utf8");
  expect(componentsCss).toMatch(/\.wallet-nav-state\.is-main\s*\{[^}]*--network-mainnet/s);
  expect(componentsCss).toMatch(/\.wallet-nav-state\.is-test\s*\{[^}]*--network-testnet/s);
  expect(componentsCss).toMatch(/\.wallet-nav-state\.is-dev\s*\{[^}]*--network-devnet/s);
});

test("desktop Wallets keeps one fixed popup trigger while navigation scrolls", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const viewport = page.locator(".wallet-nav-viewport");
  const scrollRegion = page.locator(".sidebar-navigation-scroll-region");

  const snapshot = () => viewport.evaluate((element) => {
    const trigger = element.querySelector("[data-wallet-picker-trigger]");
    const style = getComputedStyle(element);
    const rect = element.getBoundingClientRect();
    const triggerRect = trigger.getBoundingClientRect();
    return {
      viewportWidth: rect.width,
      clientWidth: element.clientWidth,
      triggerWidth: triggerRect.width,
      triggerRight: triggerRect.right,
      viewportRight: rect.right,
      overflowY: style.overflowY,
      scrollWidth: element.scrollWidth,
    };
  });

  const before = await snapshot();
  await viewport.hover();
  const onHover = await snapshot();
  const scrollContract = await scrollRegion.evaluate((region) => {
    const sidebar = region.closest(".sidebar");
    const firstItem = region.querySelector(".navigation-tree-item");
    const before = region.scrollTop;
    region.scrollTop = region.scrollHeight;
    const sidebarRect = sidebar.getBoundingClientRect();
    const regionRect = region.getBoundingClientRect();
    const firstItemRect = firstItem.getBoundingClientRect();
    return {
      before,
      after: region.scrollTop,
      clientHeight: region.clientHeight,
      scrollHeight: region.scrollHeight,
      sidebarRight: sidebarRect.right,
      regionRight: regionRect.right,
      firstItemRight: firstItemRect.right,
      sidebarPaddingRight: Number.parseFloat(getComputedStyle(sidebar).paddingRight),
    };
  });
  const after = await snapshot();

  expect(before.overflowY).toBe("visible");
  expect(before.scrollWidth).toBeLessThanOrEqual(before.clientWidth);
  expect(onHover).toMatchObject(before);
  expect(onHover.triggerRight).toBeLessThanOrEqual(onHover.viewportRight);
  expect(scrollContract.scrollHeight).toBeGreaterThan(scrollContract.clientHeight);
  expect(scrollContract.after).toBeGreaterThan(scrollContract.before);
  expect(scrollContract.regionRight).toBeCloseTo(scrollContract.sidebarRight - 1, 0);
  expect(scrollContract.firstItemRight).toBeCloseTo(
    scrollContract.regionRight - scrollContract.sidebarPaddingRight,
    0,
  );
  expect(after).toMatchObject(before);
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

  const selectWallet = async (id) => {
    await page.locator('#wallet-nav [data-wallet-picker-trigger]').click();
    await page.locator(`#wallet-picker-popup [data-wallet-picker-id="${id}"]`).click();
  };

  await selectWallet("savings");
  await expect(page.locator("#wallet-identity")).toContainText("Savings");
  await expect(page.locator('#wallet-nav [data-wallet-picker-trigger]')).toContainText("Savings");

  await page.locator(".asset-identity-button").first().click();
  await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  await selectWallet("everyday");
  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  await selectWallet("savings");
  await expect(page.locator(".claim-row")).toHaveCount(0);
  await expect(page.locator(".object-empty-state")).toBeVisible();
  await selectWallet("everyday");
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

test("One Dark YAML highlighting uses the Z00Z dark canvas background", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.settings.advanced`);
  await expect(page.locator("html")).toHaveAttribute("data-code-theme", "atom-one-dark");
  await expect(page.locator(".yaml-editor-shell")).toBeVisible();

  const colors = await page.evaluate(() => {
    const probe = document.createElement("span");
    probe.style.backgroundColor = "var(--lut-z00z-dark-canvas)";
    document.body.append(probe);
    const canvas = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return {
      canvas,
      editor: getComputedStyle(document.querySelector(".yaml-editor-shell")).backgroundColor,
    };
  });

  expect(colors.editor).toBe(colors.canvas);
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
    await expect(navigation.locator('[data-navigation-route="messenger.inbox"] use')).toHaveAttribute("href", "#i-inbox");
    await expect(navigation.locator('[data-navigation-route="messenger.sent"] use')).toHaveAttribute("href", "#i-sent");
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
    const walletTrigger = viewport.mobile
      ? page.locator('#mobile-popup-menu [data-wallet-picker-trigger]')
      : page.locator('#wallet-nav [data-wallet-picker-trigger]');
    await walletTrigger.click();
    expect(await logout.evaluate((element) => getComputedStyle(element).color)).toBe(
      await page.locator('#wallet-picker-popup [data-wallet-picker-action="remove-wallet"]').evaluate((element) => getComputedStyle(element).color),
    );
    await page.keyboard.press("Escape");
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
      "Visit Z00Z GitHub repository",
    ]);
    await expect(about.locator('a[href="https://z00z.io/"]')).toHaveCount(0);
    await expect(about.locator('a[href="https://z00z.io/docs/legal/privacy"]')).toHaveAttribute("target", "_blank");
    await expect(about.locator('a[href="https://z00z.io/docs/legal/terms"]')).toHaveAttribute("rel", "noopener noreferrer");
    await expect(about.locator('a[href="https://github.com/z00z-labs/z00z"]')).toBeVisible();
    await expect(about.locator(":scope > :last-child")).toHaveAttribute("data-demo-action", "check-for-updates");
    await expect(page.locator(".about-card, .about-metadata")).toHaveCount(0);
    await page.locator('[data-demo-action="check-for-updates"]').click();
    await expect(page.locator(".update-check-status")).toContainText("current demo version 0.1.0");

    await page.goto(`${demoUrl}?route=data-storage.disk-usage`);
    await expect(page.locator(".data-storage-view")).toContainText("Disk Usage");
    if (viewport.mobile) await page.locator("#mobile-menu-button").click();
    const navigationScope = viewport.mobile
      ? page.locator('#mobile-popup-menu[data-popup-type="menu"] .mobile-navigation-tree')
      : page.locator("#app-navigation-tree");
    await expect(navigationScope.locator('[data-navigation-route="data-storage.disk-usage"] use')).toHaveAttribute("href", "#i-bar-chart");
    await expect(navigationScope.locator('[data-navigation-route="data-storage.network-usage"] use')).toHaveAttribute("href", "#i-line-chart");
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
    await expect(page.locator("[data-language-picker-trigger]")).toContainText("🇬🇧");
    await selectAppLanguage(page, "ru");
    await expect(page.locator("[data-language-picker-trigger]")).toContainText("Русский");
    await expectNoViewportOverflow(page);
  }
});

test("English Help mirrors the Demo navigation and workspace menu", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.send`);
  const sendTitleStyle = await page.locator("#page-title").evaluate((element) => {
    const styles = getComputedStyle(element);
    return {
      fontFamily: styles.fontFamily,
      fontSize: Number.parseFloat(styles.fontSize),
      fontWeight: styles.fontWeight,
      letterSpacingRatio: Number((Number.parseFloat(styles.letterSpacing) / Number.parseFloat(styles.fontSize)).toFixed(3)),
      lineHeightRatio: Number((Number.parseFloat(styles.lineHeight) / Number.parseFloat(styles.fontSize)).toFixed(2)),
    };
  });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const helpPage = await openStandaloneHelp(page, page.locator("#app-navigation-terminal [data-help-topic]"));

  const appChrome = await page.evaluate(() => ({
    brandWidth: Math.round(document.querySelector(".desktop-topbar-brand").getBoundingClientRect().width),
    headerHeight: Math.round(document.querySelector("#primary-topbar").getBoundingClientRect().height),
  }));
  const helpChrome = await helpPage.evaluate(() => ({
    brandWidth: Math.round(document.querySelector(".help-brand").getBoundingClientRect().width),
    brandRight: Math.round(document.querySelector(".help-brand").getBoundingClientRect().right),
    headerHeight: Math.round(document.querySelector(".help-site-header").getBoundingClientRect().height),
    titleLeft: Math.round(document.querySelector("#help-product-label").getBoundingClientRect().left),
    titleText: document.querySelector("#help-product-label").textContent.trim(),
    titleStyle: (() => {
      const styles = getComputedStyle(document.querySelector("#help-product-label"));
      return {
        fontFamily: styles.fontFamily,
        fontSize: Number.parseFloat(styles.fontSize),
        fontWeight: styles.fontWeight,
        letterSpacingRatio: Number((Number.parseFloat(styles.letterSpacing) / Number.parseFloat(styles.fontSize)).toFixed(3)),
        lineHeightRatio: Number((Number.parseFloat(styles.lineHeight) / Number.parseFloat(styles.fontSize)).toFixed(2)),
      };
    })(),
  }));
  expect(helpChrome.headerHeight).toBe(appChrome.headerHeight);
  expect(helpChrome.brandWidth).toBe(appChrome.brandWidth);
  expect(helpChrome.titleLeft).toBe(helpChrome.brandRight);
  expect(helpChrome.titleText).toBe("Help");
  expect(helpChrome.titleStyle).toEqual({
    ...sendTitleStyle,
    fontSize: helpChrome.titleStyle.fontSize,
  });
  expect(helpChrome.titleStyle.fontSize).toBeGreaterThan(sendTitleStyle.fontSize);
  await expect(helpPage.locator("#help-contents-eyebrow")).toHaveCount(0);
  await expect(helpPage.locator("#help-contents-title")).toHaveCount(0);
  const searchPlacement = await helpPage.evaluate(() => {
    const search = document.querySelector("#help-search-trigger").getBoundingClientRect();
    const language = document.querySelector(".help-header-language").getBoundingClientRect();
    return {
      headerDisplay: getComputedStyle(document.querySelector(".help-sidebar-header")).display,
      sidebarSearchCount: document.querySelectorAll("#help-sidebar #help-search").length,
      searchBeforeLanguage: search.right <= language.left,
      verticalAlignment: Math.round(search.top - language.top),
    };
  });
  expect(searchPlacement).toEqual({
    headerDisplay: "none",
    sidebarSearchCount: 0,
    searchBeforeLanguage: true,
    verticalAlignment: 0,
  });

  await expect(helpPage.locator("#help-tree > [data-help-navigation-node]")).toHaveCount(6);
  await expect(helpPage.locator("#help-navigation-terminal > [data-help-navigation-node]")).toHaveCount(1);
  await expect(helpPage.locator(".help-wallet-link")).toHaveCount(0);
  await expect(helpPage.locator("#help-tree")).not.toContainText(/Help|About|Log out/);
  await expect(helpPage.locator("#help-navigation-terminal")).not.toContainText(/Help|About|Log out/);
  const terminalLayout = await helpPage.evaluate(() => {
    const treeRect = document.querySelector("#help-tree").getBoundingClientRect();
    const terminal = document.querySelector("#help-navigation-terminal");
    return {
      gap: Math.round(terminal.getBoundingClientRect().top - treeRect.bottom),
      topBorderWidth: getComputedStyle(terminal).borderTopWidth,
    };
  });
  expect(terminalLayout.gap).toBeGreaterThanOrEqual(0);
  expect(terminalLayout.gap).toBeLessThanOrEqual(8);
  expect(terminalLayout.topBorderWidth).toBe("0px");
  const navigationParity = await helpPage.evaluate(() => {
    const excluded = new Set(["help", "about", "logout"]);
    const terminal = new Set(["settings", "help", "about", "logout"]);
    const nodes = window.Z00ZDemo.navigationChildren().filter((node) => !excluded.has(node.id));
    const nodeSnapshot = (node) => ({
      id: node.dataset.helpNavigationNode,
      iconId: node.querySelector(":scope > .navigation-tree-item [data-help-navigation-icon], :scope > [data-help-navigation-icon]")
        ?.querySelector("use")?.getAttribute("href")?.replace("#i-", "") || "",
      children: [...node.querySelectorAll(":scope > .navigation-tree-children > [data-help-navigation-node]")]
        .map((child) => child.dataset.helpNavigationNode),
    });
    return {
      actualMain: [...document.querySelectorAll("#help-tree > [data-help-navigation-node]")].map(nodeSnapshot),
      expectedMain: nodes.filter((node) => !terminal.has(node.id)).map((node) => ({
        id: node.id,
        iconId: node.iconId,
        children: node.target.kind === "branch"
          ? window.Z00ZDemo.navigationChildren(node.id).map((child) => child.id)
          : [],
      })),
      actualTerminal: [...document.querySelectorAll("#help-navigation-terminal > [data-help-navigation-node]")].map(nodeSnapshot),
      expectedTerminal: nodes.filter((node) => terminal.has(node.id)).map((node) => ({
        id: node.id,
        iconId: node.iconId,
        children: window.Z00ZDemo.navigationChildren(node.id).map((child) => child.id),
      })),
    };
  });
  expect(navigationParity.actualMain).toEqual(navigationParity.expectedMain);
  expect(navigationParity.actualTerminal).toEqual(navigationParity.expectedTerminal);
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"] use').first()).toHaveAttribute("href", "#i-wallet");
  await expect(helpPage.locator('[data-help-topic-link="wallet.assets"] use')).toHaveAttribute("href", "#i-assets");
  await expect(helpPage.locator('[data-help-topic-link="messenger.inbox"] use')).toHaveAttribute("href", "#i-inbox");
  await expect(helpPage.locator('[data-help-topic-link="messenger.sent"] use')).toHaveAttribute("href", "#i-sent");
  await expect(helpPage.locator('[data-help-topic-link="data-storage.disk-usage"] use')).toHaveAttribute("href", "#i-bar-chart");
  await expect(helpPage.locator('[data-help-topic-link="data-storage.network-usage"] use')).toHaveAttribute("href", "#i-line-chart");
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator('[data-help-navigation-branch="telemetry"]').click();
  await expect(helpPage.locator('[data-help-navigation-branch="telemetry"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator('[data-help-navigation-branch="wallet"]').click();
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "false");
  await expect(helpPage.locator('[data-help-topic-link="wallet.assets"]')).toBeHidden();
  await expect(helpPage.locator('[data-help-navigation-branch="telemetry"]')).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator('[data-help-navigation-branch="wallet"]').click();
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator('[data-help-topic-link="wallet.assets"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Wallet: Assets");
  const navigationRowStyle = async (target) => target.evaluate((element) => {
    const styles = getComputedStyle(element);
    return [
      styles.display,
      styles.minHeight,
      styles.padding,
      styles.borderRadius,
      styles.color,
      styles.backgroundColor,
      styles.fontFamily,
      styles.fontSize,
      styles.fontWeight,
      styles.textDecorationLine,
    ];
  });
  expect(await navigationRowStyle(helpPage.locator('[data-help-topic-link="wallet.assets"]')))
    .toEqual(await navigationRowStyle(page.locator('[data-navigation-workspace="wallet.assets-rights"]')));
  const appWorkspaceMenu = await page.evaluate(() => {
    const rail = document.querySelector(".wallet-assets-layout > .context-rail");
    const nav = rail.querySelector(":scope > .context-nav");
    const styles = getComputedStyle(rail);
    return {
      rail: [styles.position, styles.top, styles.padding, styles.borderRadius, styles.backgroundColor, styles.borderTopColor],
      navClasses: nav.className,
      items: [...nav.querySelectorAll(":scope > .context-nav-item")].map((item) => ({
        className: item.className,
        icon: item.querySelector("use")?.getAttribute("href"),
        label: item.querySelector("span > strong")?.textContent,
        hasAppStructure: Boolean(item.querySelector(":scope > .icon + span > strong")),
      })),
    };
  });
  const helpWorkspaceMenu = await helpPage.evaluate(() => {
    const rail = document.querySelector(".wallet-assets-layout > .context-rail");
    const nav = rail.querySelector(":scope > .context-nav");
    const styles = getComputedStyle(rail);
    return {
      rail: [styles.position, styles.top, styles.padding, styles.borderRadius, styles.backgroundColor, styles.borderTopColor],
      navClasses: nav.className,
      items: [...nav.querySelectorAll(":scope > .context-nav-item")].map((item) => ({
        className: item.className,
        icon: item.querySelector("use")?.getAttribute("href"),
        label: item.querySelector("span > strong")?.textContent,
        hasAppStructure: Boolean(item.querySelector(":scope > .icon + span > strong")),
      })),
    };
  });
  expect(helpWorkspaceMenu).toEqual(appWorkspaceMenu);
  await expect(helpPage.locator(".wallet-assets-layout [data-wallet-section]")).toHaveCount(3);
  await expect(helpPage.locator(".wallet-assets-layout [role=tablist]")).toHaveCount(1);
  await expect(helpPage.locator(".wallet-assets-layout [role=tab][aria-selected=true]")).toHaveCount(1);
  await expect(helpPage.locator('[data-wallet-section="assets"] use')).toHaveAttribute("href", "#i-assets");
  await expect(helpPage.locator('[data-wallet-section="vouchers"] use')).toHaveAttribute("href", "#i-voucher");
  await expect(helpPage.locator('[data-help-context-topic="wallet.send"]')).toHaveCount(0);
  await helpPage.locator('[data-help-context-topic="wallet.vouchers"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Wallet: Vouchers");
  await expect(helpPage.locator(".wallet-assets-layout [role=tab][aria-selected=true]")).toHaveCount(1);
  await expect(helpPage.locator('[data-wallet-section="vouchers"]')).toHaveAttribute("aria-selected", "true");
  await helpPage.locator('[data-help-context-topic="wallet.assets"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Wallet: Assets");
  await expect(helpPage.locator('#help-document #current-view')).toHaveText("App View");
  await expect(helpPage.locator('#help-document img[src="help/assets/en/wallet-assets.png"]')).toBeVisible();
  await expect(helpPage.locator("[data-help-language-picker]")).toHaveCount(1);
  await expect(helpPage.locator("[data-help-language-trigger]")).toContainText("English");
  await helpPage.locator("[data-help-language-trigger]").click();
  await helpPage.locator('[data-help-language-option="ru"]').click();
  await expect(helpPage.locator("html")).toHaveAttribute("lang", "ru");
  await expect(helpPage.locator("[data-help-language-trigger]")).toContainText("Русский");
  await expect(helpPage.locator("#help-title")).toHaveText("Активы");
  await expect(helpPage.locator(".table-of-contents")).toHaveAttribute("aria-label", "Содержание");
  await expect(helpPage.locator(".table-of-contents")).toHaveAttribute("data-title", "Содержание");
  await expect(helpPage.locator("#help-search-trigger")).toContainText("Поиск");
  await helpPage.locator("#help-search-trigger").click();
  await expect(helpPage.locator("#help-search-overlay")).toBeVisible();
  await expect(helpPage.locator("#help-search")).toBeFocused();
  await helpPage.locator("#help-search").fill("Отправка");
  await expect(helpPage.locator('[data-help-search-topic="wallet.send"]')).toBeVisible();
  await expect(helpPage.locator("#help-tree")).toBeVisible();
  await helpPage.locator('[data-help-search-topic="wallet.send"]').click();
  await expect(helpPage.locator("#help-search-overlay")).toBeHidden();
  await expect(helpPage.locator("#help-search")).toHaveValue("");
  await expect(helpPage.locator("#help-title")).toHaveText("Отправка");
  await expect(helpPage.locator('[data-help-topic-link="wallet.send"]')).toHaveAttribute("aria-current", "page");
  await helpPage.keyboard.press("Control+K");
  await expect(helpPage.locator("#help-search-overlay")).toBeVisible();
  await helpPage.keyboard.press("Escape");
  await expect(helpPage.locator("#help-search-overlay")).toBeHidden();
  await expect(helpPage.locator("#help-search-trigger")).toBeFocused();
  await helpPage.locator('[data-help-topic-link="wallet.assets"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Активы");

  await helpPage.setViewportSize({ width: 390, height: 844 });
  await expect(helpPage.locator("#help-mobile-topbar-context")).toBeVisible();
  await expect(helpPage.locator("#help-mobile-topbar-context [data-wallet-section]")).toHaveCount(3);
  await expect(helpPage.locator("#help-mobile-topbar-context [role=tab][aria-selected=true]")).toHaveCount(1);
  await expect(helpPage.locator(".wallet-assets-layout > .context-rail")).toBeHidden();
  await helpPage.locator("#help-menu-button").click();
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator('[data-help-navigation-branch="telemetry"]')).toHaveAttribute("aria-expanded", "true");
  await helpPage.locator('[data-help-navigation-branch="dapps"]').click();
  await expect(helpPage.locator('[data-help-navigation-branch="dapps"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator('[data-help-navigation-branch="wallet"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator("#help-search")).toBeHidden();
  await expect(helpPage.locator(".help-search-results")).toBeHidden();
  await helpPage.locator("#help-sidebar-close").click();
  await expect(helpPage.locator("#help-sidebar")).toBeHidden();
  await helpPage.locator("#help-search-trigger").click();
  await expect(helpPage.locator("#help-search-overlay")).toBeVisible();
  await helpPage.locator("#help-search").fill("Отправка");
  await expect(helpPage.locator('[data-help-search-topic="wallet.send"]')).toBeVisible();
  await helpPage.locator('[data-help-search-topic="wallet.send"]').click();
  await expect(helpPage.locator("#help-search-overlay")).toBeHidden();
  await expect(helpPage.locator("#help-search")).toHaveValue("");
  await expect(helpPage.locator("#help-title")).toHaveText("Отправка");
  await expect(helpPage.locator("#help-sidebar")).toBeHidden();
  await expectNoViewportOverflow(helpPage);
  await helpPage.close();
});

test("English Help uses the Demo Reticulum workspace menu without duplicates", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=telemetry.reticulum.overview`);
  const helpPage = await openStandaloneHelp(page, page.getByRole("button", { name: "Help for this view" }));

  const menuSnapshot = async (target) => target.evaluate(() => {
    const rail = document.querySelector('[data-workspace-id="telemetry.reticulum"] > .context-rail');
    const nav = rail.querySelector(":scope > .context-nav");
    const styles = getComputedStyle(rail);
    return {
      rail: [styles.position, styles.top, styles.padding, styles.borderRadius, styles.backgroundColor, styles.borderTopColor],
      navClasses: nav.className,
      items: [...nav.querySelectorAll(":scope > .context-nav-item")].map((item) => ({
        className: item.className,
        icon: item.querySelector("use")?.getAttribute("href"),
        label: item.querySelector("span > strong")?.textContent,
        hasAppStructure: Boolean(item.querySelector(":scope > .icon + span > strong")),
      })),
    };
  });
  expect(await menuSnapshot(helpPage)).toEqual(await menuSnapshot(page));
  await expect(helpPage.locator('[data-workspace-id="telemetry.reticulum"] .context-nav')).toHaveCount(1);
  await expect(helpPage.locator('[data-workspace-id="telemetry.reticulum"] [data-workspace-route]')).toHaveCount(8);
  await helpPage.locator('[data-help-context-topic="telemetry.reticulum.node"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Telemetry Reticulum: Node");

  await helpPage.setViewportSize({ width: 390, height: 844 });
  await expect(helpPage.locator("#help-mobile-topbar-context")).toBeVisible();
  await expect(helpPage.locator("#help-mobile-topbar-context [data-workspace-route]")).toHaveCount(8);
  await expect(helpPage.locator('[data-workspace-id="telemetry.reticulum"] > .context-rail')).toBeHidden();
  await expectNoViewportOverflow(helpPage);
  await helpPage.close();
});

test("English Help uses the Demo wallet settings menu in its main view", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=wallet.settings.general`);
  const helpPage = await openStandaloneHelp(page, page.getByRole("button", { name: "Help for this view" }));

  const menuSnapshot = async (target) => target.evaluate(() => {
    const rail = document.querySelector(".wallet-settings-view .settings-layout > .context-rail");
    const nav = rail.querySelector(":scope > .context-nav");
    const styles = getComputedStyle(rail);
    return {
      rail: [styles.position, styles.top, styles.padding, styles.borderRadius, styles.backgroundColor, styles.borderTopColor],
      navClasses: nav.className,
      items: [...nav.querySelectorAll(":scope > .context-nav-item")].map((item) => ({
        className: item.className,
        icon: item.querySelector("use")?.getAttribute("href"),
        label: item.querySelector("span > strong")?.textContent,
        hasAppStructure: Boolean(item.querySelector(":scope > .icon + span > strong")),
      })),
    };
  });
  expect(await menuSnapshot(helpPage)).toEqual(await menuSnapshot(page));
  await expect(helpPage.locator(".wallet-settings-view .settings-layout [data-wallet-settings-section]")).toHaveCount(5);
  await helpPage.locator('[data-help-context-topic="wallet.settings.security"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("Wallet Settings: Security");
  await helpPage.close();
});

test("Help follows the App language and resolves the matching localized catalogue", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(new URL("help.html?lang=ru&topic=wallet.send", demoUrl).toString());

  await expect(page.locator("html")).toHaveAttribute("lang", "ru");
  await expect(page.locator("#help-title")).toHaveText("Отправка");
  await expect(page.locator("[data-help-language-trigger]")).toContainText("Русский");
  await expect(page.locator('[data-help-navigation-branch="wallet"]')).toContainText("Кошелёк");
  await expect(page.locator('[data-help-topic-link="wallet.assets"]')).toContainText("Активы");
  await expect(page.locator('#help-document #current-view')).toHaveText("Экран приложения");
  await expect(page.locator('#help-document img[src="help/assets/en/wallet-send.png"]')).toBeVisible();
});

test("Help renders Website Markdown enhancements without a network dependency", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(new URL("help.html", demoUrl).toString());
  await expect(page.locator("#help-title")).toBeVisible();

  await page.evaluate(() => {
    const fixture = document.createElement("article");
    fixture.id = "markdown-enhancer-fixture";
    fixture.className = "help-markdown";
    fixture.innerHTML = `
      <nav class="table-of-contents"><a class="table-of-contents-link" href="#fixture-heading">Fixture heading</a></nav>
      <h2 id="fixture-heading" tabindex="-1">Fixture heading</h2>
      <p><abbr title="HyperText Markup Language">HTML</abbr> <ins>inserted</ins> <mark>marked</mark> <span class="spoiler" tabindex="-1">secret</span> H<sub>2</sub>O x<sup>2</sup> <span class="katex">x²</span> <span class="token-inline-accent">accent</span></p>
      <div class="markdown-alert markdown-alert-note"><p class="markdown-alert-title">Note</p><p>Alert text</p></div>
      <div class="warning"><p>Container warning</p></div>
      <div style="text-align:center">Aligned text</div>
      <dl><dt>Term</dt><dd>Definition</dd></dl>
      <div class="video-embed"><iframe title="Video"></iframe></div>
      <figure><img alt="Sized image"><figcaption>Caption</figcaption></figure>
      <p>Footnote reference<sup class="footnote-ref">[1]</sup></p>
      <div class="tabs-block"><div class="tabs-nav" role="tablist">
        <button type="button" class="tabs-nav-btn tabs-nav-btn-active" data-tab-target="one" aria-selected="true">One</button>
        <button type="button" class="tabs-nav-btn tabs-nav-btn-inactive" data-tab-target="two" aria-selected="false">Two</button>
      </div><div class="tabs-panel tabs-panel-active" id="one" aria-expanded="true">First</div><div class="tabs-panel tabs-panel-hidden" id="two" aria-expanded="false">Second</div></div>
      <ul class="task-list-container"><li class="task-list-item"><input class="task-list-item-checkbox" type="checkbox" checked disabled><label>Checked task</label></li></ul>
      <pre class="code-block"><code class="hljs language-js">const amount = 42;</code></pre>
      <table><thead><tr><th>Control</th><th>Purpose</th></tr></thead><tbody><tr><td>Send</td><td>Opens a draft</td></tr></tbody></table>
      <details><summary>Details</summary><p>Expanded content</p></details>
      <div class="mermaid" data-mermaid-definition="flowchart%20LR%0AA%20--%3E%20B">flowchart LR\nA --&gt; B</div>`;
    document.body.append(fixture);
    window.Z00ZHelpMarkdownEnhancer.enhance(fixture);
  });

  const fixture = page.locator("#markdown-enhancer-fixture");
  for (const selector of [
    "abbr", "ins", "mark", ".spoiler", "sub", "sup", ".katex", ".token-inline-accent",
    ".markdown-alert", ".warning", "dl", ".video-embed", "figure", ".footnote-ref",
    ".task-list-item-checkbox", "pre", "table",
  ]) {
    expect(await fixture.locator(selector).count()).toBeGreaterThan(0);
  }
  expect(await fixture.locator(".markdown-alert").evaluate((element) => getComputedStyle(element).borderLeftWidth)).toBe("4px");
  expect(await fixture.locator(".video-embed iframe").evaluate((element) => getComputedStyle(element).aspectRatio)).toBe("16 / 9");
  expect(await fixture.locator(".spoiler").evaluate((element) => getComputedStyle(element).cursor)).toBe("pointer");
  expect(await fixture.locator(".task-list-item-checkbox").isChecked()).toBe(true);
  await fixture.locator(".tabs-nav-btn", { hasText: "Two" }).click();
  await expect(fixture.locator(".tabs-panel#one")).toHaveClass(/tabs-panel-hidden/);
  await expect(fixture.locator(".tabs-panel#two")).toHaveClass(/tabs-panel-active/);
  await fixture.locator("details summary").click();
  await expect(fixture.locator("details")).toHaveAttribute("open", "");
  await fixture.locator(".table-of-contents-link").click();
  await expect(page).toHaveURL(/#fixture-heading$/);
  await expect(fixture.locator(".mermaid svg")).toHaveCount(1);
  await expect(fixture.locator(".mermaid")).toHaveAttribute("data-mermaid-rendered", "true");
  const frame = fixture.locator(".mermaid-panzoom-frame");
  const diagram = frame.locator("svg");
  await expect(frame).toBeVisible();
  await expect(frame).toHaveAttribute("role", "region");
  await expect(frame).toHaveAttribute("tabindex", "0");
  await expect(frame).toHaveAttribute("aria-keyshortcuts", /ArrowLeft/);
  expect((await frame.boundingBox()).height).toBeLessThan(300);
  await frame.focus();
  const initialTransform = await diagram.evaluate((element) => element.style.transform);
  await page.keyboard.press("Equal");
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).not.toBe(initialTransform);
  const zoomedTransform = await diagram.evaluate((element) => element.style.transform);
  await page.keyboard.press("ArrowRight");
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).not.toBe(zoomedTransform);
  await page.keyboard.press("Minus");
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).not.toBe(zoomedTransform);
  await page.keyboard.press("0");
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).toBe(initialTransform);
  await frame.dispatchEvent("wheel", { deltaY: -120 });
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).not.toBe(initialTransform);
  await frame.dblclick();
  await expect.poll(() => diagram.evaluate((element) => element.style.transform)).toBe(initialTransform);
});

test("all popup menus use the main canvas surface on desktop and mobile", async ({ page }) => {
  const popupColors = async (selector) => page.evaluate((popupSelector) => {
    const popup = document.querySelector(popupSelector);
    return {
      popup: getComputedStyle(popup).backgroundColor,
      canvas: getComputedStyle(document.body).backgroundColor,
    };
  }, selector);

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=settings.notifications`);
  const vibrate = page.locator('[data-config-control="vibrate"]');
  const nativeControls = page.locator("#main-content select");
  expect(await nativeControls.count()).toBeGreaterThan(0);
  expect(await nativeControls.evaluateAll((controls) => controls.every((control) => (
    control.classList.contains("select-picker-native")
    && control.parentElement?.dataset.selectPicker === ""
  )))).toBe(true);
  await vibrate.locator("xpath=..").locator("[data-select-picker-trigger]").click();
  await expect(page.locator(".select-picker-menu:not([hidden])")).toBeVisible();
  expect(await popupColors(".select-picker-menu:not([hidden])")).toEqual({
    popup: "rgb(8, 16, 25)",
    canvas: "rgb(8, 16, 25)",
  });
  await page.locator(".select-picker-menu:not([hidden]) .select-picker-option", { hasText: "Important alerts only" }).click();
  await expect(vibrate).toHaveValue("alerts-only");

  await page.goto(`${demoUrl}?route=settings.appearance`);
  await page.locator('[data-palette="z00z-corporate"]').click();
  await page.goto(`${demoUrl}?route=settings.notifications`);
  await page.locator('[data-config-control="ringtone"]').locator("xpath=..").locator("[data-select-picker-trigger]").click();
  const corporateColors = await popupColors(".select-picker-menu:not([hidden])");
  expect(corporateColors.popup).toBe(corporateColors.canvas);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=settings.notifications`);
  await page.locator('[data-config-control="ringtone"]').locator("xpath=..").locator("[data-select-picker-trigger]").click();
  const mobilePickerColors = await popupColors(".select-picker-menu:not([hidden])");
  expect(mobilePickerColors.popup).toBe(mobilePickerColors.canvas);
  await page.keyboard.press("Escape");
  await expect(page.locator(".select-picker-menu:not([hidden])")).toHaveCount(0);

  await page.locator("#mobile-menu-button").click();
  const mobileDrawerColors = await popupColors("#mobile-popup-menu");
  expect(mobileDrawerColors.popup).toBe(mobileDrawerColors.canvas);
  const mobileWalletSelectorColors = await popupColors("#mobile-popup-menu .mobile-wallet-selector");
  expect(mobileWalletSelectorColors.popup).toBe(mobileWalletSelectorColors.canvas);
  await page.keyboard.press("Escape");

  await page.goto(new URL("help.html", demoUrl).toString());
  await page.locator("#help-menu-button").click();
  const helpDrawerColors = await popupColors("#help-sidebar");
  expect(helpDrawerColors.popup).toBe(helpDrawerColors.canvas);
  await expectNoViewportOverflow(page);
});

test("mobile active wallet keeps Copy above its bound chain and does not open a picker", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  const mobileWallet = page.locator("#mobile-active-wallet");
  await expect(mobileWallet.locator(".mobile-active-wallet-copy")).toBeVisible();
  await expect(mobileWallet.locator(".mobile-active-wallet-actions .environment-tag.is-main")).toHaveText("Mainnet");
  await expect(mobileWallet.locator("[data-mobile-wallet-picker]")).toHaveCount(0);
});

test("Help does not invent a workspace menu for the Demo's DApp view on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=dapps.discover`);
  const helpPage = await openStandaloneHelp(page, page.getByRole("button", { name: "Help for this view" }));
  await helpPage.setViewportSize({ width: 390, height: 844 });

  await expect(helpPage.locator("#help-sidebar")).not.toHaveClass(/is-open/);
  await helpPage.locator("#help-menu-button").click();
  await expect(helpPage.locator('[data-help-navigation-branch="dapps"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator('[data-help-topic-link="dapps.discover"]')).toBeVisible();
  await helpPage.locator('[data-help-topic-link="dapps.installed"]').click();
  await expect(helpPage.locator("#help-title")).toHaveText("dApps: Installed");
  await expect(helpPage.locator("#help-sidebar")).not.toHaveClass(/is-open/);
  await expect(helpPage.locator(".workspace-layout > .context-rail")).toHaveCount(0);
  await expect(helpPage.locator("#help-mobile-topbar-context")).toBeHidden();
  await expectNoViewportOverflow(helpPage);
  await helpPage.close();
});

test("context navigation stays vertical on desktop and uses a second mobile topbar row below Menu, logo, and the active wallet", async ({ page }) => {
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
  await expect(mobileTopbarContext.locator("[role=tablist]")).toHaveCount(1);
  await expect(mobileTopbarContext.locator("[role=tab][aria-selected=true]")).toHaveCount(1);
  await expect(mobileWalletIdentity).toBeVisible();
  await expect(mobileWalletIdentity).toContainText("ZxChpo…2Mj8Pt");
  await expect(mobileWalletIdentity).toContainText("Everyday wallet");
  const mobileAssetLogo = page.locator(".asset-logo").first();
  await expect(mobileAssetLogo).toBeVisible();
  await expect(mobileAssetLogo).toHaveCSS("width", "52px");
  await expect(mobileAssetLogo).toHaveCSS("height", "52px");
  const mobileGeometry = await page.evaluate(() => {
    const topbar = document.querySelector(".topbar").getBoundingClientRect();
    const walletIdentity = document.querySelector("#mobile-active-wallet").getBoundingClientRect();
    const context = document.querySelector("#mobile-topbar-context").getBoundingClientRect();
    const tabs = [...document.querySelectorAll("#mobile-topbar-context [data-wallet-section]")]
      .map((tab) => tab.getBoundingClientRect());
    return {
      topbarTop: topbar.top,
      topbarBottom: topbar.bottom,
      walletIdentityTop: walletIdentity.top,
      walletIdentityHeight: walletIdentity.height,
      topbarBackground: getComputedStyle(document.querySelector(".topbar")).backgroundColor,
      walletIdentityBackground: getComputedStyle(document.querySelector("#mobile-active-wallet")).backgroundColor,
      contextTop: context.top,
      contextBottom: context.bottom,
      tabBounds: tabs.map(({ top, bottom }) => ({ top, bottom })),
    };
  });
  expect(mobileGeometry.walletIdentityTop).toBeCloseTo(mobileGeometry.topbarTop, 0);
  expect(mobileGeometry.walletIdentityHeight).toBe(58);
  expect(mobileGeometry.walletIdentityBackground).toBe(mobileGeometry.topbarBackground);
  expect(mobileGeometry.contextTop).toBeGreaterThanOrEqual(mobileGeometry.walletIdentityTop + mobileGeometry.walletIdentityHeight - 1);
  for (const tab of mobileGeometry.tabBounds) {
    expect(tab.top).toBeGreaterThanOrEqual(mobileGeometry.contextTop);
    expect(tab.bottom).toBeLessThanOrEqual(mobileGeometry.topbarBottom);
  }

  const longPressIsSuppressed = await mobileTabs.filter({ hasText: "Permissions" }).evaluate((tab) => !tab.dispatchEvent(
    new MouseEvent("contextmenu", { bubbles: true, cancelable: true }),
  ));
  expect(longPressIsSuppressed).toBe(true);
  await mobileTabs.filter({ hasText: "Vouchers" }).click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  await expect(mobileTopbarContext.locator('[data-wallet-section="vouchers"]')).toHaveAttribute("aria-current", "page");
  await expect(mobileTopbarContext.locator('[role=tab][aria-selected=true]')).toHaveCount(1);
  await expect(mobileTopbarContext.locator('[data-wallet-section="vouchers"]')).toHaveAttribute("aria-selected", "true");
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
  const compactTopbar = await page.evaluate(() => {
    const topbar = document.querySelector(".topbar").getBoundingClientRect();
    const walletIdentity = document.querySelector("#mobile-active-wallet").getBoundingClientRect();
    return {
      topbarHeight: topbar.height,
      walletTop: walletIdentity.top,
      walletHeight: walletIdentity.height,
    };
  });
  expect(compactTopbar.topbarHeight).toBeGreaterThanOrEqual(58);
  expect(compactTopbar.topbarHeight).toBeLessThanOrEqual(59);
  expect(compactTopbar.walletTop).toBe(0);
  expect(compactTopbar.walletHeight).toBe(58);
  await page.locator("#mobile-menu-button").click();
  await page.locator('#mobile-popup-menu[data-popup-type="menu"] [data-wallet-picker-trigger]').click();
  await page.locator('#wallet-picker-popup [data-wallet-picker-id="savings"]').click();
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

test("wallet settings keep selected wallet values beside their labels", async ({ page }) => {
  for (const width of [1280, 390, 320]) {
    await page.setViewportSize({ width, height: 844 });
    await page.goto(`${demoUrl}?route=wallet.settings.general`);

    const geometry = await page.locator(".wallet-settings-view").evaluate((view) =>
      ["wallet-name", "wallet-id", "wallet-chain"].map((anchor) => {
        const row = view.querySelector(`[data-help-anchor="${anchor}"]`);
        const value = row.querySelector(".compact-value").getBoundingClientRect();
        const action = row.querySelector(".compact-action").getBoundingClientRect();
        const style = getComputedStyle(row.querySelector(".compact-value"));
        return { actionLeft: action.left, valueLeft: value.left, textAlign: style.textAlign };
      }),
    );

    for (const row of geometry) {
      expect(row.valueLeft).toBeLessThan(row.actionLeft);
      expect(row.textAlign).toBe("left");
    }
    const valueLefts = geometry.map(({ valueLeft }) => valueLeft);
    expect(Math.max(...valueLefts) - Math.min(...valueLefts)).toBeLessThanOrEqual(1);
    await expectNoViewportOverflow(page, `wallet settings at ${width}px`);
  }
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

test("every desktop workspace keeps the same menu-to-window gap as Assets", async ({ page }) => {
  const workspaceGeometry = () => page.evaluate(() => {
    const layout = document.querySelector("#main-content .workspace-layout");
    if (!layout) return null;
    const rail = layout.querySelector(":scope > .context-rail");
    const selectors = [
      ".wallet-tool-card",
      ".send-panel",
      ".settings-detail",
      ".telemetry-view",
      ":scope > .workspace-panel",
    ];
    const surface = selectors.map((selector) => layout.querySelector(selector)).find(Boolean);
    const panel = layout.children[1];
    const railRect = rail.getBoundingClientRect();
    return {
      gridGap: panel.getBoundingClientRect().left - railRect.right,
      surfaceGap: surface.getBoundingClientRect().left - railRect.right,
      surfaceClass: surface.className,
    };
  });

  for (const width of [1280, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(`${demoUrl}?route=wallet.assets`);
    const routes = await page.evaluate(() => window.Z00ZDemo.PORT_CONTRACT.routes);
    const assets = await workspaceGeometry();
    expect(assets.gridGap).toBeCloseTo(24, 0);
    expect(assets.surfaceGap).toBeCloseTo(assets.gridGap, 0);

    for (const route of routes) {
      await page.goto(`${demoUrl}?route=${route}`);
      const geometry = await workspaceGeometry();
      if (!geometry) continue;
      expect(
        geometry.gridGap,
        `${route} grid gap at ${width}px`,
      ).toBeCloseTo(assets.gridGap, 0);
      expect(
        geometry.surfaceGap,
        `${route} visible window gap at ${width}px; surface=${geometry.surfaceClass}`,
      ).toBeCloseTo(assets.surfaceGap, 0);
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
  await expect(drawer.locator(".mobile-drawer-header")).toHaveCount(0);
  await expect(drawer.locator(".mobile-navigation-scroll-region")).toBeVisible();
  const walletSelector = drawer.locator(".mobile-wallet-selector");
  await expect(walletSelector).toBeVisible();
  await expect(drawer.locator(":scope > .mobile-wallet-selector")).toHaveCount(1);
  await expect(walletSelector.locator('[data-wallet-picker-trigger]')).toBeVisible();
  await expect(walletSelector.locator(':scope > [data-wallet-picker-trigger]')).toHaveCount(1);
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
  const mobileScrollContract = await drawer.evaluate((drawerElement) => {
    const walletSelector = drawerElement.querySelector(".mobile-wallet-selector");
    const walletTrigger = drawerElement.querySelector(".mobile-wallet-picker-trigger");
    const region = drawerElement.querySelector(".mobile-navigation-scroll-region");
    const tree = drawerElement.querySelector(".mobile-navigation-tree");
    const terminal = region.querySelector(".mobile-navigation-terminal");
    const regionRect = region.getBoundingClientRect();
    const selectorRect = walletSelector?.getBoundingClientRect();
    const triggerRect = walletTrigger?.getBoundingClientRect();
    region.scrollTop = region.scrollHeight;
    const terminalRect = terminal?.getBoundingClientRect();
    const selectorAfterScroll = walletSelector?.getBoundingClientRect();
    return {
      terminalIsInsideScrollRegion: terminal?.parentElement === region,
      treeIsInsideScrollRegion: tree?.parentElement === region,
      walletIsFixedOutsideScrollRegion: walletSelector?.parentElement === drawerElement,
      triggerFullyVisible: Boolean(selectorRect && triggerRect
        && triggerRect.top >= selectorRect.top - 1
        && triggerRect.bottom <= selectorRect.bottom + 1),
      walletStayedFixed: Boolean(selectorRect && selectorAfterScroll
        && Math.abs(selectorRect.top - selectorAfterScroll.top) <= 1),
      clientHeight: region.clientHeight,
      scrollHeight: region.scrollHeight,
      scrollTop: region.scrollTop,
      terminalFullyReachable: Boolean(terminalRect
        && terminalRect.top >= regionRect.top - 1
        && terminalRect.bottom <= regionRect.bottom + 1),
    };
  });
  expect(mobileScrollContract.terminalIsInsideScrollRegion).toBe(true);
  expect(mobileScrollContract.treeIsInsideScrollRegion).toBe(true);
  expect(mobileScrollContract.walletIsFixedOutsideScrollRegion).toBe(true);
  expect(mobileScrollContract.triggerFullyVisible).toBe(true);
  expect(mobileScrollContract.walletStayedFixed).toBe(true);
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
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();

  await page.locator("#mobile-menu-button").click();
  await expect(walletSelector).toBeVisible();
  const walletSelectorContract = await walletSelector.evaluate((selector) => {
    const trigger = selector.querySelector(".mobile-wallet-picker-trigger");
    const drawer = selector.parentElement;
    return {
      isInsideMenu: drawer?.dataset.popupType === "menu",
      triggerIsPresent: Boolean(trigger),
      directChildCount: selector.children.length,
    };
  });
  expect(walletSelectorContract).toEqual({
    isInsideMenu: true,
    triggerIsPresent: true,
    directChildCount: 2,
  });
  await walletSelector.locator('[data-wallet-picker-trigger]').click();
  await expect(page.locator('#wallet-picker-popup [data-wallet-picker-action="add-wallet"]')).toBeVisible();
  await page.locator('#wallet-picker-popup [data-wallet-picker-action="add-wallet"]').click();
  await expect(drawer).toBeHidden();
  await expect(page.locator("#flow-dialog")).toBeVisible();
  await page.locator("#flow-dialog [data-dialog-close]").first().click();
  await expect(page.locator("#flow-dialog")).toBeHidden();

  await page.locator("#mobile-menu-button").click();
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
  await drawer.locator('[data-wallet-picker-trigger]').click();
  await page.locator('#wallet-picker-popup [data-wallet-picker-id="travel"]').click();
  await expect(drawer).toBeVisible();
  await expect(page.locator("#wallet-identity")).toContainText("Travel");

  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await page.locator("#mobile-menu-button").click();
  await expect(drawer.locator("[data-wallet-picker-trigger]")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(drawer).toBeHidden();
  await expect(page.locator("#mobile-menu-button")).toBeFocused();

  await page.locator("#mobile-menu-button").click();
  await expect(page.locator("#app-body")).toHaveJSProperty("inert", true);
  await expect(drawer.locator("[data-wallet-picker-trigger]")).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(drawer.getByRole("button", { name: "Log out", exact: true })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(drawer.locator("[data-wallet-picker-trigger]")).toBeFocused();
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
  await expect(helpPage.locator("#help-tree > [data-help-navigation-node]")).toHaveCount(6);
  await expect(helpPage.locator("#help-navigation-terminal > [data-help-navigation-node]")).toHaveCount(1);
  await expect(helpPage.locator(".help-wallet-link")).toHaveCount(0);
  await helpPage.close();
  await expect(drawer).toBeHidden();
  await expectNoViewportOverflow(page);
});

test("mobile Help reuses the App drawer shell, topbar positions, and interaction contract", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);

  const mobileShellSnapshot = (selectors) => {
    const rectangle = (element) => {
      const rect = element.getBoundingClientRect();
      return {
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      };
    };
    const drawer = document.querySelector(selectors.drawer);
    const drawerHeader = drawer.querySelector(selectors.drawerHeader);
    const navigation = drawer.querySelector(selectors.navigation);
    const terminal = drawer.querySelector(selectors.terminal);
    const context = document.querySelector(selectors.context);
    const itemSnapshot = (item) => ({
      label: item.textContent.trim(),
      rect: rectangle(item),
    });
    return {
      leading: rectangle(document.querySelector(selectors.leading)),
      identity: rectangle(document.querySelector(selectors.identity)),
      context: rectangle(context),
      contextItems: [...context.querySelectorAll(":scope > .context-nav > .context-nav-item")].map(itemSnapshot),
      drawer: rectangle(drawer),
      drawerHeader: drawerHeader ? rectangle(drawerHeader) : null,
      drawerHeaderPadding: drawerHeader ? getComputedStyle(drawerHeader).padding : null,
      drawerBackground: getComputedStyle(drawer).backgroundColor,
      navigationLabels: [...navigation.children].map((node) => node.querySelector(".navigation-tree-label")?.textContent.trim()),
      terminalLabels: [...terminal.querySelectorAll(":scope > .navigation-tree-branch > .navigation-tree-item, :scope > .navigation-tree-item")]
        .map((node) => node.textContent.trim()),
    };
  };

  await page.locator("#mobile-menu-button").click();
  await expect(page.locator("#mobile-popup-menu [data-wallet-picker-trigger]")).toBeFocused();
  await page.waitForTimeout(250);
  const appShell = await page.evaluate(mobileShellSnapshot, {
    leading: ".mobile-navigation-leading",
    identity: "#mobile-active-wallet",
    context: "#mobile-topbar-context",
    drawer: '#mobile-popup-menu[data-popup-type="menu"]',
    drawerHeader: ".mobile-drawer-header",
    navigation: ".mobile-navigation-tree",
    terminal: ".mobile-navigation-terminal",
  });
  expect(appShell.drawerHeader).toBeNull();
  expect(appShell.drawerHeaderPadding).toBeNull();

  await page.goto(new URL("help.html?topic=wallet.assets&lang=en&section=current-view", demoUrl).toString());
  await expect(page.locator("#help-sidebar")).toBeHidden();
  await expect(page.locator("#help-product-label")).toBeHidden();
  await expect(page.locator("#help-mobile-topbar-context")).toBeVisible();
  await expect(page.locator("#help-search")).toBeHidden();
  const mobileBackgrounds = await page.evaluate(() => ({
    body: getComputedStyle(document.body).backgroundColor,
    header: getComputedStyle(document.querySelector(".help-site-header")).backgroundColor,
    controls: getComputedStyle(document.querySelector(".help-header-controls")).backgroundColor,
    language: getComputedStyle(document.querySelector(".help-header-language")).backgroundColor,
  }));
  expect(new Set(Object.values(mobileBackgrounds)).size).toBe(1);
  await expect(page.locator("#current-view")).toBeFocused();
  expect(await page.locator("#current-view").evaluate((element) => getComputedStyle(element).outlineWidth)).toBe("0px");

  const helpTopbar = await page.evaluate((selectors) => {
    const rectangle = (element) => {
      const rect = element.getBoundingClientRect();
      return {
        x: Math.round(rect.x),
        y: Math.round(rect.y),
        width: Math.round(rect.width),
        height: Math.round(rect.height),
      };
    };
    return {
      leading: rectangle(document.querySelector(selectors.leading)),
      identity: rectangle(document.querySelector(selectors.identity)),
      context: rectangle(document.querySelector(selectors.context)),
      contextItems: [...document.querySelectorAll(`${selectors.context} > .context-nav > .context-nav-item`)].map((item) => ({
        label: item.textContent.trim(),
        rect: rectangle(item),
      })),
    };
  }, {
    leading: ".help-navigation-leading",
    identity: ".help-header-controls",
    context: "#help-mobile-topbar-context",
  });
  expect(helpTopbar).toEqual({
    leading: appShell.leading,
    identity: appShell.identity,
    context: appShell.context,
    contextItems: appShell.contextItems,
  });

  await page.locator('[data-help-context-topic="wallet.vouchers"]').click();
  await expect(page.locator("#help-title")).toHaveText("Wallet: Vouchers");
  await expect(page.locator('#help-mobile-topbar-context [data-wallet-section="vouchers"]')).toHaveAttribute("aria-current", "page");

  await page.locator("#help-menu-button").click();
  const helpDrawer = page.locator('#help-sidebar[data-popup-type="menu"]');
  await expect(helpDrawer).toBeVisible();
  await expect(helpDrawer.locator("[data-mobile-popup-close]")).toBeFocused();
  await expect(page.locator("#help-main")).toHaveJSProperty("inert", true);
  await expect(helpDrawer.locator(".mobile-wallet-selector")).toHaveCount(0);
  await expect(helpDrawer.locator("#help-search")).toHaveCount(0);
  await expect(helpDrawer.locator(".mobile-navigation-scroll-region")).toBeVisible();
  await expect(helpDrawer.locator(".mobile-navigation-terminal")).toContainText("Settings");
  await expect(helpDrawer.locator(".mobile-navigation-terminal")).toContainText("Version 0.1.0");
  await expect(helpDrawer.locator(".help-navigation-scroll-region")).not.toContainText(/Help|About|Log out/);
  await page.waitForTimeout(250);

  const helpShell = await page.evaluate(mobileShellSnapshot, {
    leading: ".help-navigation-leading",
    identity: ".help-header-controls",
    context: "#help-mobile-topbar-context",
    drawer: '#help-sidebar[data-popup-type="menu"]',
    drawerHeader: ".mobile-drawer-header",
    navigation: ".mobile-navigation-tree",
    terminal: ".mobile-navigation-terminal",
  });
  expect(helpShell.drawer).toEqual(appShell.drawer);
  expect(helpShell.drawerHeader).not.toBeNull();
  expect(helpShell.drawerHeaderPadding).not.toBeNull();
  expect(helpShell.drawerBackground).toBe(appShell.drawerBackground);
  expect(helpShell.navigationLabels).toEqual(appShell.navigationLabels);
  expect(helpShell.terminalLabels).toEqual(["Settings"]);
  await expect(helpDrawer.locator(".help-mobile-menu-title")).toHaveText("Menu");

  const wallet = helpDrawer.locator('[data-help-navigation-branch="wallet"]');
  const telemetry = helpDrawer.locator('[data-help-navigation-branch="telemetry"]');
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await telemetry.click();
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await expect(wallet).toHaveAttribute("aria-expanded", "true");
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "false");
  await expect(telemetry).toHaveAttribute("aria-expanded", "true");
  await wallet.click();
  await expect(wallet).toHaveAttribute("aria-expanded", "true");

  await page.evaluate(() => new Promise(requestAnimationFrame));
  await helpDrawer.locator("[data-mobile-popup-close]").focus();
  await page.keyboard.press("Shift+Tab");
  await expect(helpDrawer.locator('[data-help-navigation-branch="settings"]')).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(helpDrawer.locator("[data-mobile-popup-close]")).toBeFocused();
  await page.locator("#help-sidebar-backdrop").click({ position: { x: 380, y: 400 } });
  await expect(helpDrawer).toBeHidden();
  await expect(page.locator("#help-main")).toHaveJSProperty("inert", false);
  await expect(page.locator("#help-menu-button")).toBeFocused();

  await mobileSwipe(page, { from: { x: 28, y: 280 }, to: { x: 112, y: 280 }, source: "touch" });
  await expect(helpDrawer).toBeVisible();
  await page.waitForTimeout(250);
  await mobileSwipe(page, { from: { x: 260, y: 340 }, to: { x: 126, y: 340 }, source: "touch" });
  await expect(helpDrawer).toBeHidden();

  await page.locator("#help-menu-button").click();
  await helpDrawer.locator('[data-help-topic-link="wallet.assets"]').click();
  await expect(helpDrawer).toBeHidden();
  await expect(page.locator("#help-title")).toHaveText("Wallet: Assets");
  await expect(page).toHaveURL(/topic=wallet\.assets/);
  await expectNoViewportOverflow(page);
});

test("mobile edge swipe supplements the Menu button without hijacking vertical scroll", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const drawer = page.locator('#mobile-popup-menu[data-popup-type="menu"]');
  const menuButton = page.locator("#mobile-menu-button");

  await mobileSwipe(page, { from: { x: 28, y: 280 }, to: { x: 44, y: 430 }, source: "touch" });
  await expect(drawer).toBeHidden();

  await page.evaluate(() => {
    const eventInit = (x, buttons) => ({
      bubbles: true,
      cancelable: true,
      composed: true,
      pointerId: 73,
      pointerType: "touch",
      isPrimary: true,
      button: 0,
      buttons,
      clientX: x,
      clientY: 280,
    });
    document.elementFromPoint(28, 280).dispatchEvent(new PointerEvent("pointerdown", eventInit(28, 1)));
    document.elementFromPoint(78, 280).dispatchEvent(new PointerEvent("pointermove", eventInit(78, 1)));
  });
  await expect(drawer).toBeVisible();
  const dragProgress = await page.evaluate(() => {
    const drawer = document.querySelector('#mobile-popup-menu[data-popup-type="menu"]');
    const transform = new DOMMatrix(getComputedStyle(drawer).transform);
    return {
      offsetX: Math.round(transform.m41),
      width: Math.round(drawer.getBoundingClientRect().width),
      backdropOpacity: Number(getComputedStyle(document.querySelector("#mobile-menu-backdrop")).opacity),
    };
  });
  expect(dragProgress.offsetX).toBeLessThan(0);
  expect(dragProgress.offsetX).toBeGreaterThan(-dragProgress.width);
  expect(dragProgress.backdropOpacity).toBeGreaterThan(0);
  expect(dragProgress.backdropOpacity).toBeLessThan(1);
  await page.evaluate(() => {
    document.elementFromPoint(112, 280).dispatchEvent(new PointerEvent("pointerup", {
      bubbles: true,
      cancelable: true,
      composed: true,
      pointerId: 73,
      pointerType: "touch",
      isPrimary: true,
      button: 0,
      buttons: 0,
      clientX: 112,
      clientY: 280,
    }));
  });
  await expect(drawer).toBeVisible();
  await expect(menuButton).toHaveAttribute("aria-expanded", "true");
  await page.waitForTimeout(250);
  await expect(drawer.locator(":scope > .mobile-wallet-selector")).toBeVisible();

  await mobileSwipe(page, { from: { x: 260, y: 340 }, to: { x: 126, y: 340 }, source: "touch" });
  await expect(drawer).toBeHidden();
  await expect(menuButton).toHaveAttribute("aria-expanded", "false");

  await menuButton.click();
  await expect(drawer).toBeVisible();
  await page.keyboard.press("Escape");
});

test("mobile contextual Help opens the matching Markdown page without a second Help surface", async ({ page, context }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?route=wallet.assets`);
  const helpPage = await openStandaloneHelp(
    page,
    page.getByRole("button", { name: "Help for this view" }),
  );
  await helpPage.setViewportSize({ width: 390, height: 844 });

  await expect(helpPage).toHaveURL(/topic=wallet\.assets/);
  await expect(helpPage.locator("#help-title")).toHaveText("Wallet: Assets");
  await expect(helpPage.locator('#help-document img[src="help/assets/en/wallet-assets.png"]')).toBeVisible();
  await expect(helpPage.locator("#help-sidebar")).not.toHaveClass(/is-open/);
  await expect(helpPage.locator("#help-menu-button")).toHaveAttribute("aria-expanded", "false");
  expect(context.pages()).toHaveLength(2);
  await helpPage.close();
});

test("standalone Help keeps the Demo branch structure without fabricating workspace navigation", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?route=dapps.discover`);
  const helpPage = await openStandaloneHelp(
    page,
    page.getByRole("button", { name: "Help for this view" }),
  );
  await helpPage.setViewportSize({ width: 1280, height: 800 });

  await expect(helpPage.locator("#help-tree > [data-help-navigation-node]")).toHaveCount(6);
  await expect(helpPage.locator("#help-navigation-terminal > [data-help-navigation-node]")).toHaveCount(1);
  await expect(helpPage.locator('[data-help-navigation-branch="dapps"]')).toHaveAttribute("aria-expanded", "true");
  await expect(helpPage.locator('[data-help-topic-link="dapps.discover"]')).toBeVisible();
  await expect(helpPage.locator(".workspace-layout > .context-rail")).toHaveCount(0);
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

test("768px narrow tablet starts the drawer with Wallets while the tree scrolls", async ({ page }) => {
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
  const walletSelector = drawer.locator(":scope > .mobile-wallet-selector");
  const scrollRegion = drawer.locator(".mobile-navigation-scroll-region");
  await scrollRegion.evaluate((node) => {
    node.scrollTop = node.scrollHeight;
  });
  await expect(drawer.locator(".mobile-drawer-header")).toHaveCount(0);
  await expect(walletSelector).toBeVisible();
  await expect(walletSelector.locator(":scope > p")).toHaveText("Wallets");
  const positions = await Promise.all([
    page.locator(".topbar").boundingBox(),
    walletSelector.boundingBox(),
  ]);
  expect(Math.abs(positions[0].y + positions[0].height - positions[1].y)).toBeLessThanOrEqual(1);
});
