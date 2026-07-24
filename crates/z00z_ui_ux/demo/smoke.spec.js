const { expect, test } = require("playwright/test");

const demoUrl = process.env.Z00Z_WALLET_DEMO_URL || "http://127.0.0.1:4173/index.html";
const demoSeedWords = [
  "canvas", "orbit", "maple", "velvet", "harbor", "copper", "quiet", "meadow",
  "lamp", "river", "winter", "piano", "forest", "amber", "window", "salt",
  "comet", "paper", "garden", "silver", "cloud", "stone", "echo", "north"
];

function resourcePath(value) {
  return new URL(value, "https://demo.invalid/").pathname.slice(1);
}

async function resolved_color(page, property) {
  return page.evaluate((token) => {
    const probe = document.createElement("span");
    probe.style.backgroundColor = `var(${token})`;
    document.body.append(probe);
    const color = getComputedStyle(probe).backgroundColor;
    probe.remove();
    return color;
  }, property);
}

async function recoveryChallengeIndexes(page) {
  return page.locator('#create-wallet-verify select[data-seed-index]').evaluateAll((selects) =>
    selects.map((select) => Number(select.dataset.seedIndex))
  );
}

async function completeRecoveryChallenge(page) {
  const selects = page.locator('#create-wallet-verify select[data-seed-index]');
  const indexes = await recoveryChallengeIndexes(page);
  for (let index = 0; index < indexes.length; index += 1) {
    await selects.nth(index).selectOption(demoSeedWords[indexes[index]]);
  }
}

async function expectDialogActionsCentered(page) {
  const footer = page.locator(".dialog-footer");
  await expect(footer).toHaveCount(1);
  await expect(footer).toHaveCSS("justify-content", "center");
  const offset = await footer.evaluate((element) => {
    const buttons = [...element.querySelectorAll("button")];
    const footerBox = element.getBoundingClientRect();
    const first = buttons[0].getBoundingClientRect();
    const last = buttons.at(-1).getBoundingClientRect();
    return Math.abs((first.left + last.right) / 2 - (footerBox.left + footerBox.right) / 2);
  });
  expect(offset).toBeLessThanOrEqual(1);
}

function routeQuery(parameters) {
  return `?${new URLSearchParams(parameters).toString()}`;
}

function allRoutedViews(contract) {
  return contract.views.flatMap((view) => {
    if (view === "wallet") {
      return contract.walletSections.map((walletSection) => ({
        name: `wallet-${walletSection}`,
        query: routeQuery({ view, wallet: walletSection })
      }));
    }
    if (view === "wallet-settings") {
      return contract.walletSettingsSections.map((walletSettingsSection) => ({
        name: `wallet-settings-${walletSettingsSection}`,
        query: routeQuery({ view, walletSettings: walletSettingsSection })
      }));
    }
    if (view === "settings") {
      return contract.settingsSections.map((settingsSection) => ({
        name: `settings-${settingsSection}`,
        query: routeQuery({ view, settings: settingsSection })
      }));
    }
    if (view === "telemetry") {
      return contract.telemetrySources.flatMap((telemetrySource) => (
        contract.telemetryTabs[telemetrySource].map((tab) => ({
          name: `telemetry-${telemetrySource}-${tab}`,
          query: routeQuery({
            view,
            telemetry: telemetrySource,
            [`${telemetrySource === "onionnet" ? "onion" : telemetrySource}Tab`]: tab
          })
        }))
      ));
    }
    return [{ name: view, query: routeQuery({ view }) }];
  });
}

test("object families and claim/voucher/permission flows remain distinct", async ({ page }) => {
  await page.goto(`${demoUrl}?view=home`);

  await expect(page.locator("#i-permission")).toHaveAttribute("viewBox", "0 0 24 24");
  await expect(page.locator("#i-advanced")).toHaveAttribute("viewBox", "0 0 24 24");
  const iconContract = await page.locator(".svg-sprite symbol").evaluateAll((symbols) => ({
    allNormalized: symbols.every((symbol) => symbol.getAttribute("viewBox") === "0 0 24 24"),
    hasSourceFill: symbols.some((symbol) => symbol.querySelector('[fill="currentColor"]'))
  }));
  expect(iconContract).toEqual({ allNormalized: true, hasSourceFill: false });

  const quickPairs = page.locator(".quick-pair");
  const lowerPanels = page.locator(".home-lower > article");
  await expect(quickPairs).toHaveCount(2);
  await expect(lowerPanels).toHaveCount(2);

  const pairBox = await quickPairs.first().boundingBox();
  const panelBox = await lowerPanels.first().boundingBox();
  expect(Math.abs(pairBox.width - panelBox.width)).toBeLessThanOrEqual(1);

  await page.locator('[data-open-flow="asset-claim"]').click();
  await expect(page.getByRole("heading", { name: "Claim asset allocation" })).toBeVisible();
  await expect(page.getByText("The claim package is separate from vouchers.")).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  await page.locator('[data-wallet-id="everyday"]').click();
  await expect(page.locator(".context-rail .context-nav-item > .icon")).toHaveCount(3);
  await expect(page.locator('[data-wallet-section="vouchers"] > svg use')).toHaveAttribute("href", "#i-voucher");
  await expect(page.locator('[data-wallet-section="permissions"] > svg use')).toHaveAttribute("href", "#i-permission");
  await expect(page.locator('.context-rail [data-wallet-section] > .object-family-glyph')).toHaveCount(0);
  await expect(page.locator(".context-rail-label")).toHaveCount(0);
  await expect(page.locator(".context-rail .context-nav-item small")).toHaveCount(0);
  await expect(page.locator(".context-rail .nav-count")).toHaveCount(0);
  const assetContextType = await page.locator(".context-rail .context-nav-item").first().evaluate((item) => {
    const tab = document.querySelector("#wallet-tabs .wallet-tab");
    const icon = item.querySelector(".icon");
    const tabIcon = tab.querySelector(".icon");
    return [getComputedStyle(item).fontSize, getComputedStyle(item).fontWeight, getComputedStyle(icon).width, getComputedStyle(tab).fontSize, getComputedStyle(tab).fontWeight, getComputedStyle(tabIcon).width];
  });
  expect(assetContextType.slice(0, 3)).toEqual(assetContextType.slice(3));
  await page.getByRole("button", { name: /Vouchers/ }).click();
  await expect(page.locator(".claim-row")).toHaveCount(8);
  expect(resourcePath(await page.locator(".claim-row .object-family-glyph").first().getAttribute("src"))).toBe("assets/z00z-friendly/Vauchers/vaucher-orange.svg");
  const voucherIconSources = await page.locator(".claim-row .object-family-glyph").evaluateAll((icons) => icons.map((icon) => icon.getAttribute("src")));
  expect(new Set(voucherIconSources).size).toBe(8);
  await page.getByRole("button", { name: /Travel refund voucher/ }).click();
  await expect(page.getByRole("heading", { name: "Review voucher" })).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  await page.getByRole("button", { name: /Permissions/ }).click();
  await expect(page.locator(".permission-row")).toHaveCount(8);
  expect(resourcePath(await page.locator(".permission-row .object-family-glyph").first().getAttribute("src"))).toBe("assets/z00z-friendly/Permissions/permission-blue.svg");
  const permissionIconSources = await page.locator(".permission-row .object-family-glyph").evaluateAll((icons) => icons.map((icon) => icon.getAttribute("src")));
  expect(new Set(permissionIconSources).size).toBe(8);
  await expect(page.locator(".permission-list")).not.toContainText("Z00Z");
  await expect(page.locator('[aria-label="Permission filters"] button')).toHaveText(["Held", "Delegated", "Used"]);
  await page.getByRole("button", { name: /Delivery receipt access/ }).click();
  await expect(page.getByText("Monetary value")).toBeVisible();
  await expect(page.getByText("None", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Done" }).click();
  await page.getByRole("button", { name: /Deploy to staging/ }).click();
  await expect(page.getByRole("heading", { name: "Deploy to staging" })).toBeVisible();
  await expect(page.getByText("Machine capability", { exact: true })).toBeVisible();
});

test("wallet context rails share a compact width", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });

  const measureRail = () => page.locator(".workspace-layout").evaluate((layout) => {
    const rail = layout.querySelector(".context-rail");
    const style = getComputedStyle(layout);
    return {
      gap: Number.parseFloat(style.columnGap),
      width: Math.round(rail.getBoundingClientRect().width)
    };
  });

  await page.goto(`${demoUrl}?view=wallet`);
  const assetsRail = await measureRail();

  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=general`);
  const settingsRail = await measureRail();

  expect(assetsRail).toEqual({ gap: 24, width: 192 });
  expect(settingsRail).toEqual(assetsRail);
});

test("wallet navigation scopes history and wallet tools to the selected wallet", async ({ page }) => {
  await page.goto(demoUrl);

  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(3);
  const [activeStripeColor, brandColor, stripeTop, stripeBottom, stripeHeight, stripeRadius] = await page.locator("#wallet-nav .wallet-nav-item.is-active").evaluate((node) => {
    const probe = document.createElement("span");
    probe.style.color = "var(--brand)";
    document.body.append(probe);
    const stripe = getComputedStyle(node, "::after");
    const result = [stripe.backgroundColor, getComputedStyle(probe).color, stripe.top, stripe.bottom, stripe.height, stripe.borderRadius];
    probe.remove();
    return result;
  });
  expect(activeStripeColor).toBe(brandColor);
  expect(parseFloat(stripeTop)).toBeGreaterThan(0);
  expect(stripeBottom).toBe("-1px");
  expect(stripeHeight).toBe("3px");
  expect(stripeRadius).toBe("999px");
  await expect(page.locator('.sidebar [data-view="activity"]')).toHaveCount(0);
  await expect(page.locator("#wallet-tabs .wallet-tab")).toHaveCount(9);
  expect(await page.locator("#wallet-tabs .wallet-tab").evaluateAll((tabs) => tabs.map((tab) => tab.dataset.view))).toEqual(["wallet", "wallet-send", "wallet-receive", "swap", "exchange", "staking", "wallet-backup", "activity", "wallet-settings"]);
  await expect(page.locator('#wallet-tabs [data-view="wallet"] > .icon:not(.mobile-tab-disclosure) use')).toHaveAttribute("href", "#i-wallet");
  await expect(page.locator('[data-wallet-section="assets"] use')).toHaveAttribute("href", "#i-assets");
  await expect(page.locator('#wallet-tabs [data-view="home"]')).toHaveCount(0);
  await expect(page.locator('#wallet-tabs [data-view="wallet-send"]')).toHaveText("Send");
  await expect(page.locator('#wallet-tabs [data-view="wallet-receive"]')).toHaveText("Receive");
  await expect(page.locator('#wallet-tabs [data-view="activity"]')).toHaveText("History");
  await expect(page.locator('#wallet-tabs [data-view="swap"]')).toHaveText("Swap");
  await expect(page.locator('#wallet-tabs [data-view="exchange"]')).toContainText("Exchange");
  await expect(page.locator('#wallet-tabs [data-view="staking"]')).toHaveText("Staking");
  await expect(page.locator('#wallet-tabs [data-view="staking"] .icon use')).toHaveAttribute("href", "#i-staking");
  await expect(page.locator('#wallet-tabs [data-view="wallet-backup"]')).toHaveText("Backup");
  await expect(page.locator('#wallet-tabs [data-view="wallet-settings"]')).toHaveText("Settings");
  await expect(page.locator("#network-nav .network-nav-item")).toHaveCount(3);
  await expect(page.locator("#network-nav")).toContainText("OnionNet");
  await expect(page.locator("#network-nav")).toContainText("Reticulum");
  await expect(page.locator("#network-nav")).toContainText("Aggregators");
  await expect(page.locator("#network-nav .network-nav-item strong")).toHaveText(["Reticulum", "OnionNet", "Aggregators"]);
  await expect(page.locator("#network-nav .network-nav-copy small")).toHaveCount(0);
  const sidebarTypography = await page.evaluate(() => {
    const properties = ["fontFamily", "fontSize", "fontWeight", "lineHeight", "letterSpacing"];
    const read = (selector) => {
      const style = getComputedStyle(document.querySelector(selector));
      return Object.fromEntries(properties.map((property) => [property, style[property]]));
    };
    return {
      aggregators: read('#network-nav [data-network-section="aggregators"] .network-nav-copy strong'),
      settings: read('.system-nav [data-view="settings"] > span'),
      logout: read('.system-nav [data-demo-action="logout"] > span')
    };
  });
  expect(sidebarTypography.settings).toEqual(sidebarTypography.aggregators);
  expect(sidebarTypography.logout).toEqual(sidebarTypography.aggregators);
  const [walletNameSize, walletAmountSize, walletTabSize] = await page.locator("#wallet-nav .wallet-nav-item").first().evaluate((walletCard) => {
    const tab = document.querySelector("#wallet-tabs .wallet-tab");
    return [
      parseFloat(getComputedStyle(walletCard.querySelector(".wallet-nav-copy strong")).fontSize),
      parseFloat(getComputedStyle(walletCard.querySelector(".wallet-nav-copy small")).fontSize),
      parseFloat(getComputedStyle(tab).fontSize)
    ];
  });
  expect(walletNameSize).toBeGreaterThanOrEqual(14);
  expect(walletAmountSize).toBeGreaterThanOrEqual(12);
  expect(walletTabSize).toBeGreaterThanOrEqual(14);
  await expect(page.getByRole("button", { name: "Add wallet" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Remove wallet" })).toBeVisible();
  expect(resourcePath(await page.locator(".sidebar .brand-mark").getAttribute("src"))).toBe("assets/logo/z00z-logo-gold-circle.png");
  await expect(page.locator(".sidebar .brand-mark")).toHaveCSS("border-radius", "0px");
  await expect(page.locator(".sidebar .brand-mark")).toHaveCSS("object-fit", "contain");
  expect(await page.locator(".brand-mark").evaluateAll((marks) => marks.every((mark) => new URL(mark.getAttribute("src"), "https://demo.invalid/").pathname === "/assets/logo/z00z-logo-gold-circle.png"))).toBe(true);
  await expect(page.locator(".sidebar .brand-mark")).toHaveJSProperty("complete", true);
  await expect(page.locator(".sidebar .brand-mark")).toHaveCSS("width", "52px");
  const wordmarkSize = await page.locator(".sidebar .brand > span").evaluate((node) => parseFloat(getComputedStyle(node).fontSize));
  expect(wordmarkSize).toBeGreaterThanOrEqual(28);
  await expect(page.locator(".wallet-nav-actions .nav-item")).toHaveCount(2);
  const sidebarButtonOrder = await page.locator(".sidebar button").evaluateAll((nodes) => nodes.map((node) => node.textContent.trim()));
  expect(sidebarButtonOrder.indexOf("Add wallet")).toBeGreaterThan(sidebarButtonOrder.indexOf("Travel"));
  expect(sidebarButtonOrder.indexOf("Add wallet")).toBeLessThan(sidebarButtonOrder.indexOf("Settings"));
  expect(sidebarButtonOrder.indexOf("Remove wallet")).toBeLessThan(sidebarButtonOrder.indexOf("Settings"));
  await expect(page.locator(".wallet-nav-viewport")).toBeVisible();
  await expect(page.locator(".wallet-nav-viewport")).toHaveCSS("overflow-y", "auto");
  const walletViewportScroll = await page.locator(".wallet-nav-viewport").evaluate((viewport) => {
    const nav = viewport.querySelector(".wallet-nav");
    const actions = viewport.querySelector(".wallet-nav-actions");
    const original = nav.innerHTML;
    const initialNavBox = nav.getBoundingClientRect();
    nav.insertAdjacentHTML("afterbegin", Array.from({ length: 12 }, (_, index) => `<button class="wallet-nav-item" type="button"><span class="wallet-avatar">${index + 1}</span><span class="wallet-nav-copy"><strong>Wallet ${index + 1}</strong><small>0.00 Z00Z available</small></span></button>`).join(""));
    const beforeActionsTop = actions.getBoundingClientRect().top;
    viewport.scrollTop = viewport.scrollHeight;
    const afterActionsTop = actions.getBoundingClientRect().top;
    const result = {
      scrollable: viewport.scrollHeight > viewport.clientHeight,
      actionsInsideWalletList: actions.parentElement === nav,
      actionsFollowWallets: afterActionsTop < beforeActionsTop,
      rowsVisible: initialNavBox.height >= 3 * 68,
      visibleRows: Math.floor((viewport.clientHeight - 10) / 68),
      scrollbarGutter: getComputedStyle(viewport).scrollbarGutter
    };
    nav.innerHTML = original;
    return result;
  });
  expect(walletViewportScroll.scrollable).toBe(true);
  expect(walletViewportScroll.actionsInsideWalletList).toBe(true);
  expect(walletViewportScroll.actionsFollowWallets).toBe(true);
  expect(walletViewportScroll.rowsVisible).toBe(true);
  expect(walletViewportScroll.visibleRows).toBe(3);
  expect(walletViewportScroll.scrollbarGutter).toContain("stable");
  await expect(page.locator("html")).toHaveCSS("font-family", /Geist/);
  await expect(page.locator(".sidebar").getByRole("button", { name: "Create wallet" })).toHaveCount(0);
  await expect(page.locator('.system-nav [data-view="settings"]')).toBeVisible();
  await expect(page.getByRole("button", { name: "Log out" })).toBeVisible();
  await expect(page.locator(".connection-card")).toHaveCount(0);
  await expect(page.locator("#page-title")).toHaveText("ZxChpo…2Mj8Pt");
  await expect(page.locator("#copy-wallet-address")).toBeVisible();

  await page.locator('[data-wallet-id="savings"]').click();
  await expect(page.locator("#wallet-identity")).toContainText("Savings");
  await expect(page.locator("#wallet-statusbar")).toContainText("7,215.00 Z00Z");
  await page.locator('[data-view="activity"]:visible').click();
  await expect(page.getByText("Transfer from Everyday")).toBeVisible();
  await expect(page.getByText("Payment to Mira")).toHaveCount(0);

  await page.locator('[data-wallet-id="travel"]').click();
  await page.locator('#wallet-tabs [data-view="activity"]').click();
  await expect(page.getByText("Payment to RailLink")).toBeVisible();
  await expect(page.getByText("Transfer from Everyday")).toHaveCount(0);
  await page.locator('#wallet-tabs [data-view="swap"]').click();
  const swapHeading = page.getByRole("heading", { name: "Build a swap" });
  await expect(swapHeading).toBeVisible();
  await expect(swapHeading).toHaveCSS("font-size", "23.2px");
  await expect(page.locator(".wallet-tool-summary")).toHaveCount(0);
  await expect(page.locator(".wallet-tool-grid-single")).toHaveCSS("grid-template-columns", "640px");
  const swapGeometry = await page.locator(".wallet-tool-grid-single").evaluate((grid) => {
    const gridBox = grid.getBoundingClientRect();
    const card = grid.querySelector(".swap-card");
    const cardBox = card.getBoundingClientRect();
    const heading = card.querySelector(".tool-card-heading");
    const iconBox = heading.querySelector(".list-icon").getBoundingClientRect();
    const titleBox = heading.querySelector("h2").getBoundingClientRect();
    return {
      centerOffset: Math.abs(
        (cardBox.left + cardBox.width / 2) - (gridBox.left + gridBox.width / 2)
      ),
      headingAlignment: getComputedStyle(heading).alignItems,
      titleIconCenterOffset: Math.abs(
        (titleBox.top + titleBox.height / 2) - (iconBox.top + iconBox.height / 2)
      )
    };
  });
  expect(swapGeometry.centerOffset).toBeLessThanOrEqual(1);
  expect(swapGeometry.headingAlignment).toBe("center");
  expect(swapGeometry.titleIconCenterOffset).toBeLessThanOrEqual(1);
  await expect(page.locator('#wallet-tabs [data-view="exchange"]')).toBeDisabled();
  await page.locator('[data-view="staking"]:visible').click();
  const stakingHeading = page.getByRole("heading", { name: "Prepare a stake" });
  await expect(stakingHeading).toBeVisible();
  await expect(stakingHeading).toHaveCSS("font-size", "23.2px");
  await expect(page.locator(".staking-card")).toHaveCount(1);
  await expect(page.locator(".staking-metric")).toHaveCount(3);
  const stakingTypography = await page.locator(".staking-card").evaluate((card) => {
    const heading = card.querySelector(".tool-card-heading h2");
    const headingIcon = card.querySelector(".tool-card-heading .list-icon");
    const metricLabel = card.querySelector(".staking-metric span");
    const metricValue = card.querySelector(".staking-metric strong");
    const availableValue = card.querySelector(".staking-metric strong .sensitive");
    const amountLabel = card.querySelector('label[for="stake-amount"]');
    const style = (element) => {
      const computed = getComputedStyle(element);
      return {
        color: computed.color,
        family: computed.fontFamily,
        size: computed.fontSize,
        weight: computed.fontWeight
      };
    };
    const headingBox = heading.getBoundingClientRect();
    const iconBox = headingIcon.getBoundingClientRect();
    return {
      headingCenterOffset: Math.abs(
        (headingBox.top + headingBox.height / 2) - (iconBox.top + iconBox.height / 2)
      ),
      metricLabel: style(metricLabel),
      metricValue: style(metricValue),
      availableValue: style(availableValue),
      amountLabel: style(amountLabel)
    };
  });
  expect(stakingTypography.headingCenterOffset).toBeLessThanOrEqual(1);
  expect(stakingTypography.metricLabel.family).toContain("Geist");
  expect(stakingTypography.metricLabel.size).toBe("16px");
  expect(stakingTypography.metricLabel.weight).toBe("650");
  expect(stakingTypography.metricValue.family).toContain("Geist");
  expect(stakingTypography.metricValue.family).not.toContain("Mono");
  expect(stakingTypography.metricValue.size).toBe("20px");
  expect(stakingTypography.metricValue.weight).toBe("650");
  expect(stakingTypography.availableValue).toEqual(stakingTypography.metricValue);
  expect(stakingTypography.amountLabel.family).toBe(stakingTypography.metricLabel.family);
  expect(stakingTypography.amountLabel.size).toBe(stakingTypography.metricLabel.size);
  expect(stakingTypography.amountLabel.weight).toBe(stakingTypography.metricLabel.weight);
  await expect(page.locator(".money-summary")).toHaveCount(0);
  await expect(page.locator(".wallet-tool-summary")).toHaveCount(0);
  const stakingAlignment = await page.locator(".wallet-tool-grid-centered").evaluate((grid) => {
    const card = grid.querySelector(".staking-card").getBoundingClientRect();
    const bounds = grid.getBoundingClientRect();
    return Math.abs((card.left + card.right) / 2 - (bounds.left + bounds.right) / 2);
  });
  expect(stakingAlignment).toBeLessThanOrEqual(1);
  await page.locator('#wallet-tabs [data-view="wallet-backup"]').click();
  await expect(page.getByRole("heading", { name: "Backup status" })).toBeVisible();

  const visibleWalletRows = async () => page.locator(".wallet-nav-viewport").evaluate((viewport) => {
    const nav = viewport.querySelector(".wallet-nav");
    const actions = viewport.querySelector(".wallet-nav-actions");
    const extraRows = Array.from({ length: 3 }, (_, index) => `<button class="wallet-nav-item" type="button">Extra wallet ${index + 1}</button>`).join("");
    actions.insertAdjacentHTML("beforebegin", extraRows);
    viewport.scrollTop = 0;
    const box = viewport.getBoundingClientRect();
    const count = [...nav.querySelectorAll(".wallet-nav-item")].filter((row) => {
      const rowBox = row.getBoundingClientRect();
      return rowBox.top >= box.top && rowBox.bottom <= box.bottom;
    }).length;
    nav.querySelectorAll(".wallet-nav-item:nth-last-of-type(-n + 3)").forEach((row) => row.remove());
    return count;
  });

  await page.setViewportSize({ width: 1280, height: 800 });
  expect(await visibleWalletRows()).toBe(3);

  await page.setViewportSize({ width: 1000, height: 800 });
  expect(await visibleWalletRows()).toBe(3);
});

test("every routed view starts on the shared content baseline", async ({ page }) => {
  test.setTimeout(90_000);
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
  const contract = await page.evaluate(() => window.Z00ZDemo.PORT_CONTRACT);
  const routes = allRoutedViews(contract);

  for (const viewport of [
    { name: "desktop", width: 1280, height: 800 },
    { name: "mobile-390", width: 390, height: 844 },
    { name: "mobile-320", width: 320, height: 800 }
  ]) {
    await page.setViewportSize(viewport);
    const measurements = [];

    for (const route of routes) {
      await page.goto(`${demoUrl}${route.query}`);
      await expect(page.locator("#main-content > .view-enter")).toBeVisible();
      const offset = await page.locator("#main-content > .view-enter").evaluate((root) => {
        root.style.animation = "none";
        const firstVisibleChild = [...root.children].find((child) => {
          const bounds = child.getBoundingClientRect();
          const style = getComputedStyle(child);
          return !child.matches(".sr-only")
            && style.display !== "none"
            && style.visibility !== "hidden"
            && bounds.width > 0
            && bounds.height > 0;
        });
        const mainTop = document.querySelector("#main-content").getBoundingClientRect().top;
        return Math.round((firstVisibleChild.getBoundingClientRect().top - mainTop) * 10) / 10;
      });
      measurements.push({ route: route.name, offset });
    }

    const offsets = measurements.map(({ offset }) => offset);
    const spread = Math.max(...offsets) - Math.min(...offsets);
    expect(
      spread,
      `${viewport.name} view-start offsets:\n${JSON.stringify(measurements, null, 2)}`
    ).toBeLessThanOrEqual(1);
  }
});

test("the current workspace tabs live in the single topbar on wide screens", async ({ page }) => {
  await page.setViewportSize({ width: 1920, height: 1080 });
  await page.goto(`${demoUrl}?view=wallet`);

  const navigationContract = await page.evaluate(() => {
    const topbar = document.querySelector("#primary-topbar");
    const addressGroup = document.querySelector(".topbar-address-group");
    const tabsNode = document.querySelector("#wallet-tabs");
    const tabs = document.querySelectorAll("#wallet-tabs .wallet-tab");
    const firstTab = tabs[0];
    const firstTabStyle = getComputedStyle(firstTab);
    const firstTabBox = firstTab.getBoundingClientRect();
    const addressBox = addressGroup.getBoundingClientRect();
    const settings = tabs[tabs.length - 1].getBoundingClientRect();
    return {
      tabsParent: tabsNode.parentElement.id,
      removedSecondRow: document.querySelector(".wallet-navigation-bar") === null,
      settingsInsideTopbar: settings.right <= topbar.getBoundingClientRect().right,
      firstTabAtAddressEdge: Math.abs(firstTabBox.left - addressBox.right) <= 1,
      firstTabPadding: [
        Number.parseFloat(firstTabStyle.paddingLeft),
        Number.parseFloat(firstTabStyle.paddingRight)
      ]
    };
  });

  expect(navigationContract).toEqual({
    tabsParent: "primary-topbar",
    removedSecondRow: true,
    settingsInsideTopbar: true,
    firstTabAtAddressEdge: true,
    firstTabPadding: [15, 15]
  });
});

test("history exposes only object-type filters", async ({ page }) => {
  await page.goto(`${demoUrl}?view=activity`);

  const filters = page.locator("#main-content [data-filter]");
  await expect(filters).toHaveText(["All", "Assets", "Vouchers", "Permissions", "System"]);
  await expect(filters).toHaveCount(5);
  await expect(page.getByRole("button", { name: "Needs attention", exact: true })).toHaveCount(0);
});

test("history rows use a compact desktop measure and remain fluid on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?view=activity`);

  const desktopGeometry = await page.locator(".activity-card-list").evaluate((list) => {
    const listBox = list.getBoundingClientRect();
    const filterBox = document.querySelector(".filter-bar").getBoundingClientRect();
    const searchBox = document.querySelector(".filter-bar .search-wrap").getBoundingClientRect();
    return {
      width: listBox.width,
      filterWidth: filterBox.width,
      leftOffset: Math.abs(listBox.left - filterBox.left),
      searchWidth: searchBox.width,
      searchRightOffset: Math.abs(searchBox.right - listBox.right)
    };
  });
  expect(desktopGeometry.width).toBeLessThanOrEqual(760.5);
  expect(Math.abs(desktopGeometry.filterWidth - desktopGeometry.width)).toBeLessThanOrEqual(1);
  expect(desktopGeometry.leftOffset).toBeLessThanOrEqual(1);
  expect(desktopGeometry.searchWidth).toBeCloseTo(220, 0);
  expect(desktopGeometry.searchRightOffset).toBeLessThanOrEqual(1);

  for (const viewport of [{ width: 390, height: 844 }, { width: 320, height: 800 }]) {
    await page.setViewportSize(viewport);
    await page.goto(`${demoUrl}?view=activity`);
    const mobileGeometry = await page.locator(".activity-card-list").evaluate((list) => {
      const listBox = list.getBoundingClientRect();
      const parentBox = list.parentElement.getBoundingClientRect();
      return {
        width: listBox.width,
        availableWidth: parentBox.width,
        insideViewport: listBox.left >= 0 && listBox.right <= document.documentElement.clientWidth
      };
    });
    expect(Math.abs(mobileGeometry.width - mobileGeometry.availableWidth)).toBeLessThanOrEqual(1);
    expect(mobileGeometry.insideViewport).toBe(true);
  }
});

test("filters and row hovers use the shared neutral interaction state", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet`);

  const selectedBackground = await resolved_color(page, "--interaction-selected-bg");
  const selectedBorder = await resolved_color(page, "--brand");
  const hoverBackground = await resolved_color(page, "--interaction-hover-bg");
  const hoverBorder = await resolved_color(page, "--interaction-hover-border");

  const assetsAll = page.locator('[data-asset-filter="all"]');
  await expect(assetsAll).toHaveCSS("background-color", selectedBackground);
  await expect(assetsAll).toHaveCSS("border-top-color", selectedBorder);
  await page.locator('[data-asset-filter="token"]').click();
  await expect(page.locator('[data-asset-filter="token"]')).toHaveCSS("background-color", selectedBackground);

  const assetRow = page.locator('.asset-row').first();
  await assetRow.hover();
  await expect(assetRow).toHaveCSS("background-color", hoverBackground);
  await expect(assetRow).toHaveCSS("border-top-color", hoverBorder);

  const walletRow = page.locator('#wallet-nav [data-wallet-id="savings"]');
  await walletRow.hover();
  await expect(walletRow).toHaveCSS("background-color", hoverBackground);
  await expect(walletRow).toHaveCSS("border-top-color", hoverBorder);

  const assetsTab = page.locator('[data-wallet-section="assets"]');
  await assetsTab.hover();
  await expect(assetsTab).toHaveCSS("background-color", hoverBackground);
  await expect(assetsTab).toHaveCSS("border-top-color", hoverBorder);

  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.locator('[aria-label="Voucher filters"] .choice-chip.is-active')).toHaveCSS("background-color", selectedBackground);
  await page.locator('[data-wallet-section="permissions"]').click();
  await expect(page.locator('[aria-label="Permission filters"] .choice-chip.is-active')).toHaveCSS("background-color", selectedBackground);

  await page.locator('#wallet-tabs [data-view="activity"]').click();
  await page.locator('[data-filter="voucher"]').click();
  await expect(page.locator('[data-filter="voucher"]')).toHaveCSS("background-color", selectedBackground);
  const activityRow = page.locator('.activity-row').first();
  await activityRow.hover();
  await expect(activityRow).toHaveCSS("background-color", hoverBackground);

  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  await expect(page.locator(".send-panel")).toBeVisible();
  await expect(page.locator(".transfer-asset-row")).toHaveCount(0);

  await page.locator('#wallet-tabs [data-view="wallet-settings"]').click();
  const settingLine = page.locator('.setting-line').first();
  await settingLine.hover();
  await expect(settingLine).toHaveCSS("background-color", hoverBackground);
});

test("backup action is separated from the backup status card", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet-backup`);

  const heading = page.getByRole("heading", { name: "Backup status" });
  await expect(heading).toHaveCSS("font-size", "23.2px");
  await expect(page.locator(".wallet-tool-summary")).toHaveCount(0);
  await expect(page.locator(".wallet-tool-grid-single")).toHaveCSS("grid-template-columns", "640px");
  await expect(page.locator(".wallet-backup-actions")).toHaveCSS("margin-top", "16px");
  const [statusBox, actionsBox, createBox, recoveryBox, iconBox, headingBox, gridBox, cardBox] = await Promise.all([
    page.locator(".wallet-tool-card .review-card").first().boundingBox(),
    page.locator(".wallet-backup-actions").boundingBox(),
    page.locator(".wallet-backup-action").boundingBox(),
    page.locator(".wallet-backup-recovery").boundingBox(),
    page.locator(".backup-card-heading .list-icon").boundingBox(),
    heading.boundingBox(),
    page.locator(".wallet-tool-grid-centered").boundingBox(),
    page.locator(".backup-card").boundingBox()
  ]);
  expect(actionsBox.y - (statusBox.y + statusBox.height)).toBeGreaterThanOrEqual(16);
  expect(recoveryBox.y - (createBox.y + createBox.height)).toBe(10);
  expect(recoveryBox.width).toBe(createBox.width);
  await expect(page.locator(".wallet-backup-recovery .icon")).toBeVisible();
  await expect(page.locator(".wallet-backup-recovery use")).toHaveAttribute("href", "#i-restore");
  expect(Math.abs((iconBox.y + iconBox.height / 2) - (headingBox.y + headingBox.height / 2))).toBeLessThanOrEqual(1);
  expect(Math.abs((gridBox.x + gridBox.width / 2) - (cardBox.x + cardBox.width / 2))).toBeLessThanOrEqual(1);

  const rowHeights = await page.locator(".backup-summary .summary-row").evaluateAll((rows) => (
    rows.map((row) => Math.round(row.getBoundingClientRect().height))
  ));
  expect(new Set(rowHeights).size).toBe(1);

  await page.locator(".wallet-backup-recovery").click();
  await expect(page.locator("#toast-region")).toContainText("Restore validates integrity before any replacement.");
});

test("selected wallet settings are scoped, re-authenticated, and capability-labelled", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=general`);

  await expect(page.locator("#wallet-tabs .wallet-tab.is-active")).toHaveText("Settings");
  await expect(page.locator(".wallet-settings-context .context-nav-item")).toHaveCount(5);
  await expect(page.locator(".wallet-settings-context .context-group-label")).toHaveCount(0);
  await expect(page.locator(".wallet-settings-context .context-nav-item small")).toHaveCount(0);
  await expect(page.locator(".wallet-settings-context .context-nav-item > .icon")).toHaveCount(5);
  const settingsContextType = await page.locator(".wallet-settings-context .context-nav-item").first().evaluate((item) => {
    const tab = document.querySelector("#wallet-tabs .wallet-tab");
    const icon = item.querySelector(".icon");
    const tabIcon = tab.querySelector(".icon");
    return [getComputedStyle(item).fontSize, getComputedStyle(item).fontWeight, getComputedStyle(icon).width, getComputedStyle(tab).fontSize, getComputedStyle(tab).fontWeight, getComputedStyle(tabIcon).width];
  });
  expect(settingsContextType.slice(0, 3)).toEqual(settingsContextType.slice(3));
  await expect(page.getByRole("heading", { name: "Wallet details" })).toHaveCount(0);
  await expect(page.getByText("Selected wallet", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Local profile", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Display currency", { exact: true })).toHaveCount(0);
  await expect(page.getByLabel("Default fee")).toHaveCount(0);
  await expect(page.getByText(/public wallet-settings write route is not registered yet/i)).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Help for this view" })).toBeVisible();

  await page.getByRole("button", { name: "Rename" }).click();
  await expect(page.getByRole("heading", { name: "Rename wallet" })).toBeVisible();
  await expectDialogActionsCentered(page);
  await page.locator("#wallet-rename-name").fill("Daily");
  await expect(page.locator("#wallet-rename-name")).toHaveValue("Daily");
  await page.locator("#wallet-rename-password").fill("concept-pass");
  await expect(page.locator("#wallet-rename-password")).toHaveValue("concept-pass");
  await page.getByRole("button", { name: "Save wallet name" }).click();
  await expect(page.getByText("Wallet name updated")).toBeVisible();
  await expectDialogActionsCentered(page);
  await page.getByRole("button", { name: "Done" }).click();
  await expect(page.locator("#wallet-nav")).toContainText("Daily");

  await page.getByRole("button", { name: /Security/ }).click();
  await page.getByRole("button", { name: "Change password" }).click();
  await expect(page.getByRole("heading", { name: "Change wallet password" })).toBeVisible();
  await expectDialogActionsCentered(page);
  await page.getByLabel("Current password").fill("concept-pass");
  await page.getByLabel("New password", { exact: true }).fill("concept-new");
  await page.getByLabel("Confirm new password").fill("concept-new");
  await page.locator('button[type="submit"][form="wallet-password-change-entry"]').click();
  await expect(page.locator("#dialog-title")).toHaveText("Password updated");
  await expect(page.locator("#wallet-current-password, #wallet-new-password, #wallet-confirm-new-password")).toHaveCount(0);
  await expectDialogActionsCentered(page);
  await page.getByRole("button", { name: "Done" }).click();

  await page.getByRole("button", { name: "View phrase" }).click();
  await page.locator("#wallet-seed-reveal-password").fill("concept-pass");
  await page.locator("#wallet-seed-reveal-confirmation").fill("SHOW SEED");
  await page.getByRole("button", { name: "Reveal demonstration phrase" }).click();
  await expect(page.getByText("DEMONSTRATION WORDS · NOT A REAL WALLET SEED")).toBeVisible();
  await page.getByRole("button", { name: "Done" }).click();

  await page.getByRole("button", { name: /Policies/ }).click();
  await expect(page.getByText("Compliance profile", { exact: true })).toBeVisible();
  await expect(page.getByText("Target", { exact: true }).last()).toBeVisible();
  await page.getByRole("button", { name: "Review" }).click();
  await page.locator("#wallet-policy-apply-password").fill("concept-pass");
  await expect(page.locator("#wallet-policy-apply-password")).toHaveValue("concept-pass");
  await page.locator("#wallet-policy-apply-confirmation").fill("APPLY");
  await expect(page.locator("#wallet-policy-apply-confirmation")).toHaveValue("APPLY");
  await page.getByRole("button", { name: "Apply local rules" }).click();
  await expect(page.getByText("Local spend rules updated")).toBeVisible();
});

test("wallet settings tabs start directly with their controls", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=security`);

  const sections = ["security", "backup", "policies", "advanced"];
  for (const section of sections) {
    await page.locator(`[data-wallet-settings-section="${section}"]`).click();
    await expect(page.locator(".settings-detail > .settings-heading")).toHaveCount(0);
  }

  await expect(page.getByText("Private authority", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Recovery state", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Bounded authority", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Wallet configuration", { exact: true })).toHaveCount(0);
});

test("wallet backup settings keep scheduling controls while backup actions stay in the Backup tab", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=backup`);

  const walletSettings = page.locator(".wallet-settings-view");
  await expect(walletSettings.getByText("Automatic backup", { exact: true })).toBeVisible();
  await expect(walletSettings.getByText("Backup interval", { exact: true })).toBeVisible();
  await expect(walletSettings.locator('[data-demo-action="backup"]')).toHaveCount(0);
  await expect(walletSettings.locator('[data-demo-action="restore"]')).toHaveCount(0);

  await page.goto(`${demoUrl}?view=wallet-backup`);
  await expect(page.locator(".wallet-backup-action")).toBeVisible();
  await expect(page.locator(".wallet-backup-recovery")).toBeVisible();
});

test("every modal flow centers its footer actions on desktop and mobile", async ({ page }) => {
  await page.goto(demoUrl);
  const scenarios = [
    { type: "asset-claim" },
    { type: "create-voucher" },
    { type: "voucher-detail" },
    { type: "voucher-review" },
    { type: "voucher-settled" },
    { type: "create-permission" },
    { type: "permission" },
    { type: "permission-detail", data: { permissionId: "receipt" } },
    { type: "activity", data: { item: { id: "audit", type: "money", direction: "out", title: "Audit transfer", detail: "Modal audit", amount: "1.00 Z00Z", time: "Now", status: "settled" } } },
    { type: "asset-detail", data: { assetKey: "z00z" } },
    { type: "connection" },
    { type: "wallets" },
    { type: "remove-wallet" },
    { type: "add-wallet" },
    { type: "create-wallet" },
    { type: "open-wallet" },
    { type: "recover-wallet" },
    { type: "wallet-rename" },
    { type: "wallet-password-change" },
    { type: "wallet-seed-reveal" },
    { type: "wallet-public-export" },
    { type: "wallet-key-rotation" },
    { type: "wallet-policy-apply" },
    { type: "wallet-policy-profile" },
    { type: "notifications" }
  ];

  for (const viewport of [{ width: 1280, height: 900 }, { width: 390, height: 844 }]) {
    await page.setViewportSize(viewport);
    for (const scenario of scenarios) {
      await test.step(`${viewport.width}px · ${scenario.type}`, async () => {
        await page.evaluate(({ type, data }) => window.openFlow(type, document.body, data || {}), scenario);
        await expect(page.locator("#flow-dialog")).toBeVisible();
        await expectDialogActionsCentered(page);
      });
    }
  }
});

test("secure-entry fields do not activate browser password-manager overlays", async ({ page }) => {
  await page.goto(demoUrl);
  const ignoredByPasswordManagers = async (locator) => locator.evaluateAll((elements) => elements.map((element) => ({
    formType: element.getAttribute("data-form-type"),
    lastPassIgnore: element.getAttribute("data-lpignore"),
    onePasswordIgnore: element.getAttribute("data-1p-ignore"),
    bitwardenIgnore: element.getAttribute("data-bwignore"),
    protonPassIgnore: element.getAttribute("data-protonpass-ignore")
  })));
  const isIgnored = (attributes) => attributes.every((attribute) => attribute.formType === "other"
    && attribute.lastPassIgnore === "true"
    && attribute.onePasswordIgnore === "true"
    && attribute.bitwardenIgnore === "true"
    && attribute.protonPassIgnore === "true");

  for (const type of ["create-wallet", "wallet-rename", "wallet-password-change", "wallet-seed-reveal", "wallet-public-export", "wallet-key-rotation", "wallet-policy-apply"]) {
    await page.evaluate((flowType) => window.openFlow(flowType, document.body), type);
    const passwordForm = page.locator('#flow-dialog form:has(input[data-secure-entry])');
    const passwordInputs = passwordForm.locator('input[data-secure-entry]');
    await expect(passwordForm).toHaveCount(1);
    expect(await passwordInputs.count()).toBeGreaterThan(0);
    expect(isIgnored(await ignoredByPasswordManagers(passwordForm))).toBe(true);
    expect(isIgnored(await ignoredByPasswordManagers(passwordInputs))).toBe(true);
    const autocompleteValues = await passwordInputs.evaluateAll((inputs) => inputs.map((input) => input.autocomplete));
    expect(autocompleteValues.every((value) => /one-time-code$/.test(value))).toBe(true);
    expect(await passwordInputs.evaluateAll((inputs) => inputs.every((input) => input.type === "text"
      && input.dataset.portControl === "secure-entry"
      && getComputedStyle(input).webkitTextSecurity === "disc"))).toBe(true);
    await expect(page.locator('#flow-dialog input[type="password"], #flow-dialog [autocomplete$="new-password"]')).toHaveCount(0);
    for (let index = 0; index < await passwordInputs.count(); index += 1) {
      const input = passwordInputs.nth(index);
      await expect(input).toHaveValue("");
      await input.focus();
      await expect(page.getByText("Manage passwords", { exact: true })).toHaveCount(0);
    }
    const credentialLikeNames = await passwordForm.locator("input").evaluateAll((inputs) => inputs
      .map((input) => input.name)
      .filter((name) => /^(?:name|username|user|password|currentPassword|newPassword)$/i.test(name)));
    expect(credentialLikeNames).toEqual([]);
    const walletLabels = passwordForm.locator('input[name="walletLabel"]');
    if (await walletLabels.count()) {
      expect(isIgnored(await ignoredByPasswordManagers(walletLabels))).toBe(true);
      await expect(walletLabels.first()).toHaveAttribute("autocomplete", /nickname$/);
    }
  }
  const unlockForm = page.locator("#unlock-form");
  const unlockPassword = page.locator("#unlock-password");
  expect(isIgnored(await ignoredByPasswordManagers(unlockForm))).toBe(true);
  expect(isIgnored(await ignoredByPasswordManagers(unlockPassword))).toBe(true);
  await expect(unlockPassword).toHaveAttribute("autocomplete", /one-time-code$/);
  await expect(unlockPassword).toHaveAttribute("type", "text");
  await expect(unlockPassword).toHaveCSS("-webkit-text-security", "disc");
  await expect(page.locator('input[type="password"], [autocomplete$="new-password"]')).toHaveCount(0);
  await expect(page.getByText("Manage passwords", { exact: true })).toHaveCount(0);
});

test("every form field suppresses password-manager overlays", async ({ page }) => {
  const expectSuppressed = async (root) => {
    const forms = root.locator("form");
    const fields = root.locator("input, textarea, select");
    if (await forms.count()) {
      const formContract = await forms.evaluateAll((elements) => elements.map((element) => ({
        autocomplete: element.getAttribute("autocomplete"),
        formType: element.getAttribute("data-form-type"),
        onePasswordIgnore: element.getAttribute("data-1p-ignore"),
        lastPassIgnore: element.getAttribute("data-lpignore"),
        bitwardenIgnore: element.getAttribute("data-bwignore"),
        protonPassIgnore: element.getAttribute("data-protonpass-ignore")
      })));
      expect(formContract.every((entry) => entry.autocomplete === "off"
        && entry.formType === "other"
        && entry.onePasswordIgnore === "true"
        && entry.lastPassIgnore === "true"
        && entry.bitwardenIgnore === "true"
        && entry.protonPassIgnore === "true")).toBe(true);
    }
    if (await fields.count()) {
      const fieldContract = await fields.evaluateAll((elements) => elements.map((element) => ({
        autocomplete: element.getAttribute("autocomplete"),
        formType: element.getAttribute("data-form-type"),
        onePasswordIgnore: element.getAttribute("data-1p-ignore"),
        lastPassIgnore: element.getAttribute("data-lpignore"),
        bitwardenIgnore: element.getAttribute("data-bwignore"),
        protonPassIgnore: element.getAttribute("data-protonpass-ignore")
      })));
      expect(fieldContract.every((entry) => entry.autocomplete
        && entry.formType === "other"
        && entry.onePasswordIgnore === "true"
        && entry.lastPassIgnore === "true"
        && entry.bitwardenIgnore === "true"
        && entry.protonPassIgnore === "true")).toBe(true);
    }
    await expect(page.getByText("Manage passwords", { exact: true })).toHaveCount(0);
  };

  for (const location of [
    demoUrl,
    `${demoUrl}?view=wallet-send`,
    `${demoUrl}?view=activity`,
    `${demoUrl}?view=swap`,
    `${demoUrl}?view=staking`,
    `${demoUrl}?view=settings&settings=general`,
    `${demoUrl}?view=settings&settings=appearance`,
    `${demoUrl}?view=wallet-settings&walletSettings=advanced`
  ]) {
    await page.goto(location);
    await expectSuppressed(page.locator("body"));
  }

  const scenarios = [
    { type: "create-voucher" },
    { type: "create-permission" },
    { type: "permission" },
    { type: "create-wallet" },
    { type: "open-wallet" },
    { type: "recover-wallet" },
    { type: "wallet-rename" },
    { type: "wallet-password-change" },
    { type: "wallet-seed-reveal" },
    { type: "wallet-public-export" },
    { type: "wallet-key-rotation" },
    { type: "wallet-policy-apply" }
  ];
  for (const scenario of scenarios) {
    await page.evaluate(({ type, data }) => window.openFlow(type, document.body, data || {}), scenario);
    const dialogFields = page.locator("#flow-dialog input, #flow-dialog textarea, #flow-dialog select");
    expect(await dialogFields.count()).toBeGreaterThan(0);
    await expectSuppressed(page.locator("#flow-dialog"));
  }
});

test("left navigation has exactly one active destination", async ({ page }) => {
  await page.goto(demoUrl);

  const activeRailItems = page.locator(".sidebar .wallet-nav-item.is-active, .sidebar .network-nav-item.is-active, .sidebar .system-nav .nav-item.is-active");
  const expectOnly = async (selector) => {
    await expect(activeRailItems).toHaveCount(1);
    await expect(page.locator(`.sidebar ${selector}`)).toHaveClass(/is-active/);
    await expect(page.locator(".sidebar .wallet-nav-item[aria-current='page'], .sidebar .network-nav-item[aria-current='page'], .sidebar .system-nav .nav-item[aria-current='page']")).toHaveCount(1);
  };

  await expectOnly('[data-wallet-id="everyday"]');

  await page.locator('[data-network-section="onionnet"]').click();
  await expectOnly('[data-network-section="onionnet"]');
  await expect(page.getByRole("heading", { name: "OnionNet telemetry" })).toBeVisible();
  await expect(page.getByText("Local capability unavailable")).toBeVisible();
  await expect(page.getByRole("button", { name: "Help for this view" })).toBeVisible();
  await expect(page.locator('[data-wallet-id="everyday"]')).not.toHaveClass(/is-active/);
  await expect(page.locator('.system-nav [data-view="settings"]')).not.toHaveClass(/is-active/);

  await page.locator('.system-nav [data-view="settings"]').click();
  await expectOnly('.system-nav [data-view="settings"]');
  await expect(page.locator('[data-network-section="onionnet"]')).not.toHaveClass(/is-active/);

  await page.locator('[data-wallet-id="savings"]').click();
  await expectOnly('[data-wallet-id="savings"]');
  await expect(page.locator('.system-nav [data-view="settings"]')).not.toHaveClass(/is-active/);
});

test("network shortcuts open read-only telemetry rather than setup", async ({ page }) => {
  await page.goto(demoUrl);
  const expectTelemetryTopbar = async (title, context) => {
    await expect(page.locator("#page-title")).toHaveText(title);
    await expect(page.locator("#page-context")).toHaveText(context);
    await expect(page.locator("#page-title")).toHaveClass(/is-telemetry-title/);
    const titleStyle = await page.locator("#page-title").evaluate((node) => ({
      fontSize: getComputedStyle(node).fontSize,
      fontWeight: Number.parseInt(getComputedStyle(node).fontWeight, 10),
      letterSpacing: Number.parseFloat(getComputedStyle(node).letterSpacing)
    }));
    expect(titleStyle.fontSize).toBe("26px");
    expect(titleStyle.fontWeight).toBeGreaterThanOrEqual(700);
    expect(titleStyle.letterSpacing).toBeGreaterThan(0);
  };

  await page.locator('[data-network-section="onionnet"]').click();
  await expect(page.getByRole("heading", { name: "OnionNet telemetry" })).toBeVisible();
  await expectTelemetryTopbar("OnionNet", "Route telemetry");
  const onionnetTabs = [
    ["overview", "Overview"],
    ["epoch", "Epoch"],
    ["privacy", "Privacy"],
    ["transport", "Transport"],
    ["queues", "Queues & Replay"],
    ["probation", "Probation"],
    ["ingress", "Ingress"]
  ];
  await expect(page.locator("#wallet-tabs")).toBeVisible();
  await expect(page.locator("#wallet-tabs [data-onionnet-telemetry-tab]")).toHaveCount(onionnetTabs.length);
  for (const [id, label] of onionnetTabs) {
    await page.locator(`[data-onionnet-telemetry-tab="${id}"]`).click();
    await expect(page.locator(`[data-onionnet-telemetry-tab="${id}"]`)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("[data-onionnet-telemetry-tab][aria-selected='true']")).toHaveCount(1);
    await expect(page.getByRole("tabpanel", { name: label })).toContainText("Unavailable");
  }
  await page.getByRole("button", { name: "Help for this view" }).click();
  await expect(page.locator("#help-title")).toHaveText("OnionNet ingress");
  await page.locator(".help-panel [data-help-close]").click();
  await expect(page.locator("#wallet-tabs .wallet-tab.is-active")).toHaveCount(1);

  await page.locator('[data-network-section="reticulum"]').click();
  await expect(page.getByRole("heading", { name: "Reticulum telemetry" })).toBeVisible();
  await expectTelemetryTopbar("Reticulum", "Carrier telemetry");
  const reticulumTabs = [
    ["overview", "Overview"],
    ["node", "Node"],
    ["interfaces", "Interfaces"],
    ["radio", "Radio"],
    ["entrypoints", "Entry points"],
    ["paths", "Paths"],
    ["probes", "Probes"],
    ["links", "Links"]
  ];
  await expect(page.locator("#wallet-tabs")).toBeVisible();
  await expect(page.locator("#wallet-tabs [data-reticulum-telemetry-tab]")).toHaveCount(reticulumTabs.length);
  for (const [id, label] of reticulumTabs) {
    await page.locator(`[data-reticulum-telemetry-tab="${id}"]`).click();
    await expect(page.locator(`[data-reticulum-telemetry-tab="${id}"]`)).toHaveAttribute("aria-selected", "true");
    await expect(page.locator("[data-reticulum-telemetry-tab][aria-selected='true']")).toHaveCount(1);
    await expect(page.getByRole("tabpanel", { name: label })).toContainText("Unavailable");
  }
  await page.getByRole("button", { name: "Help for this view" }).click();
  await expect(page.locator("#help-title")).toHaveText("Reticulum links");
  await page.locator(".help-panel [data-help-close]").click();
  await expect(page.locator("#copy-wallet-address")).toBeHidden();
  await expect(page.locator("#wallet-tabs .wallet-tab.is-active")).toHaveCount(1);

  await page.locator('[data-network-section="aggregators"]').click();
  await expect(page.getByRole("heading", { name: "Aggregators telemetry" })).toBeVisible();
  await expectTelemetryTopbar("Aggregators", "Publication telemetry");
  await expect(page.locator("#wallet-tabs [data-aggregators-telemetry-tab]")).toHaveCount(1);
  await page.locator('[data-aggregators-telemetry-tab="overview"]').click();
  await expect(page.locator('[data-aggregators-telemetry-tab="overview"]')).toHaveAttribute("aria-selected", "true");
  await expect(page.getByRole("tabpanel", { name: "Overview" })).toContainText("Service bindings");
  await expect(page.getByText("Service bindings", { exact: true }).last()).toBeVisible();
  await expect(page.getByText("Local capability unavailable")).toBeVisible();
  await expect(page.locator('[data-network-section="aggregators"]')).toHaveClass(/is-active/);
  await expect(page.locator(".sidebar .wallet-nav-item.is-active, .sidebar .network-nav-item.is-active, .sidebar .system-nav .nav-item.is-active")).toHaveCount(1);
});

test("wallet address copy stays next to the address and uses the shared native title tooltip", async ({ page }) => {
  await page.goto(demoUrl);

  const copyPositions = [];
  for (const walletId of ["everyday", "savings", "travel"]) {
    await page.locator(`[data-wallet-id="${walletId}"]`).click();
    const geometry = await page.locator(".topbar-address-group").evaluate((group) => {
      const addressBox = group.querySelector("#page-title").getBoundingClientRect();
      const contextBox = group.querySelector("#page-context").getBoundingClientRect();
      const copyBox = group.querySelector("#copy-wallet-address").getBoundingClientRect();
      return {
        copyX: copyBox.x,
        addressRight: addressBox.right,
        copyRight: copyBox.right,
        copyCenterY: copyBox.top + copyBox.height / 2,
        contextCenterY: contextBox.top + contextBox.height / 2,
        copyWidth: copyBox.width,
        copyHeight: copyBox.height
      };
    });
    copyPositions.push(geometry.copyX);
    expect(Math.abs(geometry.addressRight - geometry.copyRight)).toBeLessThanOrEqual(1);
    expect(Math.abs(geometry.copyCenterY - geometry.contextCenterY)).toBeLessThanOrEqual(2);
    expect([geometry.copyWidth, geometry.copyHeight]).toEqual([26, 26]);
  }
  expect(Math.max(...copyPositions) - Math.min(...copyPositions)).toBeLessThanOrEqual(0.5);
  await expect(page.locator(".health-pill")).toHaveCount(0);
  await expect(page.locator("#wallet-statusbar")).toContainText("Route telemetry");
  await expect(page.locator("#wallet-statusbar")).toContainText("Unavailable");

  await page.locator('[data-wallet-id="everyday"]').click();
  const copy = page.locator("#copy-wallet-address");
  await expect(copy).toHaveAttribute("title", "ZxChpoioBEFR1PRJPamJxh5aWdEb94ek8J52PmT8PYAEa8RKVtSs9X3UPgaSaHvMMZKcQoiyVFhEE256vcyGPeFV23d2Mj8Pt");
  const otherPanelButton = page.locator('[data-demo-action="toggle-balance"]');
  await copy.hover();
  await page.waitForTimeout(220);
  const copyHoverStyle = await copy.evaluate((node) => {
    const style = getComputedStyle(node);
    return [style.color, style.borderColor, style.backgroundColor];
  });
  await otherPanelButton.hover();
  const otherHoverStyle = await otherPanelButton.evaluate((node) => {
    const style = getComputedStyle(node);
    return [style.color, style.borderColor, style.backgroundColor];
  });
  expect(copyHoverStyle).toEqual(otherHoverStyle);
  await expect(copy).toHaveAttribute("title", /ZxChpoioBEFR1PRJ/);
});

test("remove wallet selects one or more wallet cards before changing local profiles", async ({ page }) => {
  await page.goto(demoUrl);

  const remove = page.locator(".wallet-nav-actions").getByRole("button", { name: "Remove wallet" });
  await remove.click();
  await expect(page.getByRole("heading", { name: "Remove wallet profiles" })).toBeVisible();
  await expect(page.locator("[data-remove-wallet-id]")).toHaveCount(3);
  await expect(page.getByRole("button", { name: "Remove profiles" })).toBeDisabled();
  const removeFooterCentered = await page.locator(".dialog-footer").evaluate((footer) => getComputedStyle(footer).justifyContent);
  expect(removeFooterCentered).toBe("center");
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(3);

  await remove.click();
  await page.locator('[data-remove-wallet-id="savings"]').check();
  await page.locator('[data-remove-wallet-id="travel"]').check();
  const selectedRemoveCard = page.locator('.wallet-remove-choice:has(input:checked)').first();
  const [selectedCardBackground, selectedCardExpected, checkedBackground, checkedDanger, checkboxWidth, checkboxHeight, checkboxRadius] = await selectedRemoveCard.evaluate((card) => {
    const panelProbe = document.createElement("span");
    panelProbe.style.background = "color-mix(in srgb, var(--danger) 12%, var(--bg-raised))";
    const dangerProbe = document.createElement("span");
    dangerProbe.style.background = "var(--danger)";
    document.body.append(panelProbe, dangerProbe);
    const checkbox = card.querySelector("input");
    const checkboxStyle = getComputedStyle(checkbox);
    const result = [getComputedStyle(card).backgroundColor, getComputedStyle(panelProbe).backgroundColor, checkboxStyle.backgroundColor, getComputedStyle(dangerProbe).backgroundColor, checkboxStyle.width, checkboxStyle.height, checkboxStyle.borderRadius];
    panelProbe.remove();
    dangerProbe.remove();
    return result;
  });
  expect(selectedCardBackground).toBe(selectedCardExpected);
  expect(checkedBackground).toBe(checkedDanger);
  expect(checkboxWidth).toBe("20px");
  expect(checkboxHeight).toBe("20px");
  expect(checkboxRadius).toBe("3px");
  await expect(page.getByText("2 of 3 selected. This removes concept profiles only.")).toBeVisible();
  await page.getByRole("button", { name: "Remove profiles (2)" }).click();
  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(1);
  await expect(page.locator("#wallet-nav")).toContainText("Everyday");
  await expect(page.locator("#wallet-identity")).toContainText("Everyday");

  await page.getByRole("button", { name: "Remove wallet" }).click();
  await page.locator('[data-remove-wallet-id="everyday"]').check();
  await expect(page.getByText("1 of 1 selected. This removes concept profiles only.")).toBeVisible();
  await expect(page.getByText("All concept profiles will be removed.")).toBeVisible();
  await page.getByRole("button", { name: "Remove profiles (1)" }).click();
  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(0);
  await expect(page.locator("#page-title")).not.toHaveText("Add wallet");
  await expect(page.getByRole("heading", { name: "Add wallet" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Remove wallet" })).toBeDisabled();
});

test("send opens one compact inline form and receive shows only the receiver card", async ({ page }) => {
  await page.goto(demoUrl);

  await expect(page.getByText("Pay", { exact: true })).toHaveCount(0);
  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  const sendPanel = page.locator(".send-panel");
  await expect(sendPanel).toBeVisible();
  await expect(page.getByRole("heading", { name: "Send privately" })).toBeVisible();
  await expect(page.locator(".send-view")).toHaveCSS("padding-top", "0px");
  const recipientLabel = page.locator('label[for="send-recipient"]');
  await expect(recipientLabel).toHaveCSS("font-family", /Geist(?! Mono)/);
  await expect(recipientLabel).toHaveCSS("font-size", "16px");
  await expect(page.locator(".transfer-asset-row, .transfer-object-row")).toHaveCount(0);
  await expect(page.locator("#flow-dialog")).not.toHaveAttribute("open", "");
  await expect(page.locator("#send-item option")).toHaveCount(22);
  expect((await sendPanel.boundingBox()).width).toBe(640);
  await page.locator("#send-item").selectOption("z00z");
  await expect(page.locator("#send-amount")).toHaveAttribute("max", "12480.75");
  await page.getByLabel("Recipient or private request").fill("Mira");
  await page.locator("#send-amount").fill("12");
  await page.getByRole("button", { name: /Review send/ }).click();
  await expect(page.locator("#flow-dialog")).not.toHaveAttribute("open", "");
  await expect(page.getByText("Z00Z · Coin")).toBeVisible();
  await page.getByRole("button", { name: "Send asset" }).click();
  await expect(page.getByRole("heading", { name: "Asset sent" })).toBeVisible({ timeout: 2000 });
  await page.getByRole("button", { name: "Done" }).click();

  await page.locator('#wallet-tabs [data-view="wallet-receive"]').click();
  const receiverCard = page.locator(".receiver-card");
  await expect(receiverCard).toBeVisible();
  await expect(receiverCard.locator(".mock-qr span")).toHaveCount(441);
  await expect(receiverCard.locator(".receiver-card-address")).toContainText("ZxChpoioBEFR");
  await expect(receiverCard.locator(".receiver-card-address")).toContainText("23d2Mj8Pt");
  await expect(receiverCard.locator("h1, h2, h3, p, .transfer-asset-row, .choice-strip")).toHaveCount(0);
  await expect(receiverCard.locator("button")).toHaveCount(1);
  await receiverCard.locator(".receiver-card-copy").click();
  await expect(page.getByText("Wallet address copied.")).toBeVisible();

  await page.goto(`${demoUrl}?view=home`);
  await page.locator('.quick-action[data-view="wallet-receive"]').click();
  await expect(page.locator(".receiver-card")).toBeVisible();
  await expect(page.locator("#flow-dialog")).not.toHaveAttribute("open", "");
});

test("send, swap, staking, and backup share one responsive card width", async ({ page }) => {
  const routes = [
    ["wallet-send", ".send-panel"],
    ["swap", ".swap-card"],
    ["staking", ".staking-card"],
    ["wallet-backup", ".backup-card"],
  ];

  await page.setViewportSize({ width: 1280, height: 800 });
  const desktopWidths = [];
  for (const [view, selector] of routes) {
    await page.goto(`${demoUrl}?view=${view}`);
    desktopWidths.push(Math.round((await page.locator(selector).boundingBox()).width));
  }
  expect(desktopWidths).toEqual([640, 640, 640, 640]);

  await page.setViewportSize({ width: 390, height: 844 });
  const mobileWidths = [];
  for (const [view, selector] of routes) {
    await page.goto(`${demoUrl}?view=${view}`);
    const box = await page.locator(selector).boundingBox();
    mobileWidths.push(Math.round(box.width));
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(390);
  }
  expect(new Set(mobileWidths).size).toBe(1);
});

test("swap and staking form labels use the standard readable Geist treatment", async ({ page }) => {
  await page.goto(demoUrl);

  for (const [view, label] of [["swap", "swap-from"], ["staking", "stake-amount"]]) {
    await page.locator(`#wallet-tabs [data-view="${view}"]`).click();
    const fieldLabel = page.locator(`label[for="${label}"]`);
    await expect(fieldLabel).toBeVisible();
    await expect(fieldLabel).toHaveCSS("font-family", /Geist(?! Mono)/);
    await expect(fieldLabel).toHaveCSS("font-size", "16px");
    await expect(fieldLabel).toHaveCSS("font-weight", "650");
  }
});

test("log out clears the application shell before unlock", async ({ page }) => {
  await page.goto(demoUrl);

  await page.getByRole("button", { name: "Log out" }).click();
  await expect(page.locator("#lock-screen")).toBeVisible();
  await expect(page.locator("#app-shell")).toBeHidden();

  const password = page.locator("#unlock-password");
  const visibilityToggle = page.locator("[data-toggle-password]");
  await expect(visibilityToggle).toHaveAttribute("aria-label", "Show password");
  await expect(password).toHaveCSS("-webkit-text-security", "disc");
  await visibilityToggle.click();
  await expect(password).toHaveCSS("-webkit-text-security", "none");
  await expect(visibilityToggle).toHaveAttribute("aria-label", "Hide password");
  await visibilityToggle.click();
  await expect(password).toHaveCSS("-webkit-text-security", "disc");
  await password.fill("concept-lock");
  await page.locator("#unlock-form").press("Enter");
  await expect(page.locator("#lock-screen")).toBeHidden();
  await expect(page.locator("#app-shell")).toBeVisible();
  await expect(password).toHaveValue("");
});

test("add wallet dialog creates and restores wallet cards", async ({ page }) => {
  await page.goto(demoUrl);
  const priorTopbarTitle = await page.locator("#page-title").textContent();

  await page.getByRole("button", { name: "Add wallet" }).click();
  await expect(page.locator("#page-title")).toHaveText(priorTopbarTitle);
  await expect(page.getByRole("heading", { name: "Add wallet" })).toBeVisible();
  await expect(page.locator(".flow-dialog")).toBeVisible();
  await expect(page.getByRole("button", { name: "Create new wallet" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Open existing wallet" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Restore from backup" })).toBeVisible();
  const addCancel = page.getByRole("button", { name: "Cancel" });
  await expect(addCancel).toBeVisible();
  await expect(addCancel).toHaveClass(/button-quiet/);
  await expect(addCancel).toHaveCSS("min-height", "44px");
  await expect(addCancel).not.toHaveCSS("border-top-style", "none");
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await expect(page.getByText(/Back to /)).toHaveCount(0);
  const addChoiceWidths = await page.locator(".add-wallet-choice").evaluateAll((buttons) => buttons.map((button) => button.getBoundingClientRect().width));
  expect(new Set(addChoiceWidths).size).toBe(1);
  expect(addChoiceWidths[0]).toBeLessThanOrEqual(360);
  const [addWalletBackgrounds, addWalletBrandStrong] = await page.locator(".add-wallet-choice.is-primary").evaluateAll((buttons) => {
    const probe = document.createElement("span");
    probe.style.color = "var(--brand-strong)";
    document.body.append(probe);
    const brandStrong = getComputedStyle(probe).color;
    probe.remove();
    return [buttons.map((button) => getComputedStyle(button).backgroundImage), brandStrong];
  });
  expect(addWalletBackgrounds).toHaveLength(2);
  for (const background of addWalletBackgrounds) {
    expect(background).toContain(addWalletBrandStrong);
    expect(background).not.toContain("rgb(50, 169, 232)");
  }

  await page.getByRole("button", { name: "Create new wallet" }).click();
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await expect(page.getByText("Storage", { exact: true })).toHaveCount(0);
  const chainSelect = page.locator("#create-chain");
  await expect(chainSelect.locator("option")).toHaveText(["Mainnet", "Testnet-1", "Testnet-2", "Devnet-1", "Devnet-2"]);
  await expect(chainSelect).toHaveValue("mainnet");
  await chainSelect.selectOption("testnet-2");
  const createName = page.locator("#create-name");
  const createPassword = page.locator("#create-password");
  const createConfirm = page.locator("#create-confirm");
  await createName.fill("Field Fund");
  await expect(createName).toHaveValue("Field Fund");
  await createPassword.fill("concept-password");
  await expect(createPassword).toHaveValue("concept-password");
  await createConfirm.fill("concept-password");
  await expect(createConfirm).toHaveValue("concept-password");
  await page.getByRole("button", { name: "Create securely" }).click();
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await page.getByRole("button", { name: "I've saved these words" }).click();
  await expect(page.getByText("Confirm four random words before continuing")).toBeVisible();
  await expect(page.locator('#create-wallet-verify select[data-seed-index]')).toHaveCount(4);
  const firstChallenge = await recoveryChallengeIndexes(page);
  await page.getByRole("button", { name: "View words again" }).click();
  await page.getByRole("button", { name: "I've saved these words" }).click();
  const secondChallenge = await recoveryChallengeIndexes(page);
  expect(secondChallenge).toHaveLength(4);
  expect(secondChallenge.every((index) => !firstChallenge.includes(index))).toBe(true);
  await completeRecoveryChallenge(page);
  await page.getByRole("button", { name: "Finish setup" }).click();
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await page.getByRole("button", { name: "Open wallet" }).click();

  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(4);
  await expect(page.locator("#wallet-identity")).toContainText("Field Fund");
  await expect(page.locator("#wallet-statusbar")).toContainText("0.00 Z00Z");
  await expect(page.locator(".asset-row")).toHaveCount(16);
  await expect(page.locator(".asset-row").first()).toContainText("Z00Z");
  await expect(page.locator(".asset-row").first()).toContainText("0.00 Z00Z");
  await expect(page.getByText("Acme Credits", { exact: true })).toHaveCount(0);
  const freshWalletAssets = await page.locator(".asset-row").evaluateAll((rows) => rows.map((row) => {
    const values = [...row.querySelectorAll(".asset-number strong")].map((value) => value.textContent.trim());
    const image = row.querySelector(".asset-logo img");
    return { balance: values[0], value: values[1], icon: image?.getAttribute("src"), loaded: Boolean(image?.naturalWidth) };
  }));
  expect(freshWalletAssets.every(({ balance, value, loaded }) => balance.startsWith("0.00 ") && value === "0.00" && loaded)).toBe(true);
  expect(freshWalletAssets.map(({ icon }) => resourcePath(icon))).toEqual([
    "assets/z00z-friendly/Coins/z00z-logo-gold.svg",
    "assets/z00z-friendly/Coins/algorand-algo-logo-z00z.svg",
    "assets/z00z-friendly/Coins/avalanche-avax-logo-z00z.svg",
    "assets/z00z-friendly/Coins/bitcoin-btc-logo-z00z.svg",
    "assets/z00z-friendly/Coins/BOLD_logo-z00z.svg",
    "assets/z00z-friendly/Coins/cardano-ada-logo-z00z.svg",
    "assets/z00z-friendly/Coins/dai-dai-logo-z00z.svg",
    "assets/z00z-friendly/Coins/ethereum-eth-logo-z00z.svg",
    "assets/z00z-friendly/Coins/hyperliquid-hype-logo-z00z.svg",
    "assets/z00z-friendly/Coins/liquity-lqty-logo-z00z.svg",
    "assets/z00z-friendly/Coins/solana-sol-logo-z00z.svg",
    "assets/z00z-friendly/Coins/zcash-zec-logo-z00z.svg",
    "assets/z00z-friendly/Tokens/rain-rain.svg",
    "assets/z00z-friendly/Tokens/sky-sky.svg",
    "assets/z00z-friendly/NFTs/bcap-nft.svg",
    "assets/z00z-friendly/NFTs/stable-nft.svg"
  ]);

  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.getByRole("heading", { name: "No vouchers yet" })).toBeVisible();
  await expect(page.locator(".claim-row")).toHaveCount(0);
  await page.getByRole("button", { name: "Create voucher" }).click();
  await page.locator("#voucher-create-name").fill("Field credit");
  await page.locator("#voucher-create-amount").fill("25");
  await page.locator('button[form="create-voucher-entry"]').click();
  await expect(page.locator(".claim-row")).toHaveCount(1);
  await expect(page.locator(".claim-row")).toContainText("Field credit");

  await page.locator('[data-wallet-section="permissions"]').click();
  await expect(page.getByRole("heading", { name: "No permissions yet" })).toBeVisible();
  await expect(page.locator(".permission-row")).toHaveCount(0);
  await page.getByRole("button", { name: "Create permission" }).click();
  await page.locator("#permission-create-name").fill("Field access");
  await page.locator('button[form="create-permission-entry"]').click();
  await expect(page.locator(".permission-row")).toHaveCount(1);
  await expect(page.locator(".permission-row")).toContainText("Field access");

  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  await expect(page.locator(".transfer-asset-row, .transfer-object-row")).toHaveCount(0);
  await expect(page.locator("#send-item option")).toContainText(["Field credit", "Field access"]);
  await page.locator("#send-item").selectOption({ label: "Field credit · Vouchers" });
  await expect(page.locator("#send-amount")).toHaveCount(0);
  await expect(page.locator(".send-object-value")).toContainText("Field credit");
  await page.locator("#send-recipient").fill("ZxRecipient42");
  await page.getByRole("button", { name: /Review send/ }).click();
  await page.getByRole("button", { name: "Send voucher" }).click();
  await expect(page.getByRole("heading", { name: "Voucher sent" })).toBeVisible({ timeout: 2000 });
  await page.getByRole("button", { name: "Done" }).click();
  await expect(page.locator("#send-item")).toContainText("Field access");
  await expect(page.locator("#send-item")).not.toContainText("Field credit");

  await page.locator('#wallet-tabs [data-view="wallet-settings"]').click();
  const chainRow = page.locator("[data-wallet-chain-readonly]");
  await expect(chainRow).toContainText("Chain");
  await expect(chainRow.locator(".environment-tag")).toHaveText("Testnet-2");
  await expect(chainRow.locator("select, input, button")).toHaveCount(0);
  await page.getByRole("button", { name: "Advanced" }).click();
  const createdWalletYaml = page.locator("#wallet-settings-yaml");
  await expect(createdWalletYaml).toHaveValue(/chain: "testnet-2"/);
  await createdWalletYaml.fill((await createdWalletYaml.inputValue()).replace('chain: "testnet-2"', 'chain: "devnet-1"'));
  await page.getByRole("button", { name: "Apply locally" }).click();
  await expect(page.locator(".config-foot")).toContainText("chain is read-only and must remain testnet-2");
  await page.getByRole("button", { name: "General" }).click();
  await expect(page.locator("[data-wallet-chain-readonly] .environment-tag")).toHaveText("Testnet-2");

  await page.getByRole("button", { name: "Add wallet" }).click();
  await page.getByRole("button", { name: "Restore from backup" }).click();
  await page.locator("#recover-name").fill("Recovered Store");
  await page.getByRole("button", { name: "Fill demonstration words" }).click();
  await page.getByRole("button", { name: "Validate and recover" }).click();
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await page.getByRole("button", { name: "Open wallet" }).click();
  await expect(page.locator("#wallet-nav .wallet-nav-item")).toHaveCount(5);
  await expect(page.locator("#wallet-nav")).toContainText("Recovered Store");

  await page.getByRole("button", { name: "Add wallet" }).click();
  await page.getByRole("button", { name: "Open existing wallet" }).click();
  await expect(page.locator(".dialog-footer")).toHaveCSS("justify-content", "center");
  await page.getByRole("button", { name: "Cancel" }).click();
});

test("assets show table values and expose per-asset details", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet`);

  await expect(page.locator("#i-send")).toHaveAttribute("viewBox", "0 0 24 24");
  await expect(page.locator("#i-send path")).toHaveAttribute("d", "M12 20V5m-6 6 6-6 6 6");
  await expect(page.locator("#i-receive")).toHaveAttribute("viewBox", "0 0 24 24");
  await expect(page.locator("#i-receive path")).toHaveAttribute("d", "M12 4v15m-6-6 6 6 6-6");
  await expect(page.locator("#i-swap path")).toHaveAttribute("d", "M7 7h12m0 0-3-3m3 3-3 3M17 17H5m0 0 3 3m-3-3 3-3");
  await expect(page.locator("#i-exchange g")).toHaveAttribute("stroke-width", "1.5");
  await expect(page.locator("#i-exchange path").first()).toHaveAttribute("d", "M3.53 11.47v2.118a4.235 4.235 0 0 0 4.235 4.236H20.47M3.53 6.176h12.705a4.235 4.235 0 0 1 4.236 4.236v2.117");
  await expect(page.locator("#main-content .page-intro")).toHaveCount(0);
  await expect(page.locator("#main-content .money-summary")).toHaveCount(0);
  for (const walletId of ["everyday", "savings", "travel"]) {
    await page.locator(`[data-wallet-id="${walletId}"]`).click();
    await expect(page.locator("#main-content .page-intro")).toHaveCount(0);
    await expect(page.locator("#main-content .money-summary")).toHaveCount(0);
    await expect(page.locator(".asset-row")).toHaveCount(16);
    await expect(page.locator(".asset-logo img")).toHaveCount(16);
    expect(await page.locator(".asset-logo img").evaluateAll((images) => images.every((image) => image.naturalWidth > 0))).toBe(true);
  }
  await expect(page.locator(".asset-table-head")).toContainText("Name");
  await expect(page.locator(".asset-table-head")).toContainText("Balance");
  await expect(page.locator(".asset-table-head")).toContainText("Value");
  await expect(page.locator(".asset-table-head")).toContainText("Price");
  await expect(page.locator(".asset-actions")).toHaveCount(0);
  await expect(page.locator(".asset-info small")).toHaveCount(0);
  await expect(page.locator(".asset-number-label")).toHaveCount(48);
  await expect(page.locator(".asset-number-label").first()).toBeHidden();
  await expect(page.locator(".asset-logo img")).toHaveCount(16);
  await expect(page.getByText("Acme Credits", { exact: true })).toHaveCount(0);
  await expect(page.getByText("Founders Pass #014", { exact: true })).toHaveCount(0);
  const assetFilters = page.locator("[data-asset-filter]");
  await expect(assetFilters).toHaveCount(4);
  await expect(assetFilters).toHaveText(["All", "Coins", "Tokens", "NFTs"]);
  await page.getByRole("button", { name: "Tokens", exact: true }).click();
  await expect(page.locator(".asset-row")).toHaveCount(5);
  await expect(page.locator(".asset-row")).toContainText(["wBOLD", "wDAI", "wLiquity", "Rain", "Sky"]);
  await page.getByRole("button", { name: "NFTs", exact: true }).click();
  await expect(page.locator(".asset-row")).toHaveCount(2);
  await expect(page.locator(".asset-row")).toContainText(["BCAP", "STABLE"]);
  await page.getByRole("button", { name: "All", exact: true }).click();
  await expect(page.locator(".asset-transfer-links")).toHaveCount(0);
  const walletTabIcons = page.locator('#wallet-tabs [data-view="wallet"], #wallet-tabs [data-view="wallet-send"], #wallet-tabs [data-view="wallet-receive"]');
  await expect(walletTabIcons.locator(":scope > .icon:not(.mobile-tab-disclosure)")).toHaveCount(3);
  await expect(walletTabIcons.nth(0).locator(":scope > .icon:not(.mobile-tab-disclosure) use")).toHaveAttribute("href", "#i-wallet");
  await expect(walletTabIcons.nth(1).locator(":scope > .icon:not(.mobile-tab-disclosure) use")).toHaveAttribute("href", "#i-send");
  await expect(walletTabIcons.nth(2).locator(":scope > .icon:not(.mobile-tab-disclosure) use")).toHaveAttribute("href", "#i-receive");
  const tabIconBoxes = await walletTabIcons.locator(":scope > .icon:not(.mobile-tab-disclosure)").evaluateAll((icons) => icons.map((item) => {
    const box = item.getBoundingClientRect();
    return [box.width, box.height, getComputedStyle(item).transform];
  }));
  expect(tabIconBoxes).toEqual([[19, 19, "none"], [19, 19, "none"], [19, 19, "none"]]);
  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  await expect(page.locator(".send-panel")).toBeVisible();
  await expect(page.locator(".transfer-asset-list")).toHaveCount(0);
  await page.locator('#wallet-tabs [data-view="wallet"]').click();
  await page.locator('#wallet-tabs [data-view="wallet-receive"]').click();
  await expect(page.locator(".receiver-card")).toBeVisible();
  await expect(page.locator(".transfer-asset-list")).toHaveCount(0);
  await page.locator('#wallet-tabs [data-view="wallet"]').click();
  await expect(page.locator(".asset-table-head")).toBeVisible();
  const columnPositions = await page.locator(".asset-table-head").evaluate((head) => {
    const headers = [...head.querySelectorAll("span")];
    const values = [
      document.querySelector(".asset-row .asset-info strong"),
      ...document.querySelectorAll(".asset-row .asset-number strong")
    ];
    return headers.slice(0, 4).map((header, index) => ({
      label: header.textContent.trim(),
      headerX: header.getBoundingClientRect().x + parseFloat(getComputedStyle(header).paddingLeft),
      valueX: values[index].getBoundingClientRect().x
    }));
  });
  for (const column of columnPositions) {
    expect(Math.abs(column.headerX - column.valueX), `${column.label} must align with its values`).toBeLessThanOrEqual(1);
  }
  await page.getByRole("button", { name: "View details for wZcash" }).click();
  const assetDetailLogo = page.locator(".dialog-header .asset-detail-logo");
  expect(resourcePath(await assetDetailLogo.locator("img").getAttribute("src"))).toBe("assets/z00z-friendly/Coins/zcash-zec-logo-z00z.svg");
  const detailLogoBox = await assetDetailLogo.boundingBox();
  const detailLogoImageBox = await assetDetailLogo.locator("img").boundingBox();
  expect([detailLogoBox.width, detailLogoBox.height]).toEqual([64, 64]);
  expect(Math.abs(detailLogoImageBox.x - detailLogoBox.x - 6)).toBeLessThanOrEqual(1);
  expect(Math.abs(detailLogoImageBox.y - detailLogoBox.y - 6)).toBeLessThanOrEqual(1);
  for (const field of ["Asset name", "Ticker", "Owner", "Asset ID", "Current supply", "Max supply"]) {
    await expect(page.getByText(field, { exact: true })).toBeVisible();
  }
  await page.getByRole("button", { name: "OK", exact: true }).click();
});

test("wallet object rows stay consistent while receive uses a single responsive receiver card", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet`);

  const readIconBoxes = (selector) => page.locator(selector).evaluateAll((icons) => icons.map((icon) => {
    const style = getComputedStyle(icon);
    return [style.width, style.height, style.borderRadius];
  }));
  const expectCardRows = async (rows, minHeight, renderedHeight = 64) => {
    const styles = await rows.evaluateAll((items) => items.map((item) => {
      const style = getComputedStyle(item);
      return [style.minHeight, style.borderTopWidth, style.borderRadius, style.backgroundColor, Math.round(item.getBoundingClientRect().height)];
    }));
    expect(styles.length).toBeGreaterThan(1);
    styles.forEach((style) => {
      expect(style[0]).toBe(minHeight);
      expect(style[1]).toBe("1px");
      expect(style[2]).toBe("14px");
      expect(style[4]).toBeGreaterThanOrEqual(renderedHeight);
      expect(style[4]).toBeLessThanOrEqual(renderedHeight + 1);
    });
    return styles[0].slice(0, 4);
  };
  const expectCardRowGaps = async (rows) => {
    const gaps = await rows.evaluateAll((items) => items.slice(1).map((item, index) => (
      Math.round(item.getBoundingClientRect().top - items[index].getBoundingClientRect().bottom)
    )));
    expect(gaps).toEqual(Array(Math.max(0, gaps.length)).fill(8));
  };

  const assetCardStyle = await expectCardRows(page.locator(".asset-row"), "64px");
  await expectCardRowGaps(page.locator(".asset-row"));
  expect(await readIconBoxes(".asset-logo")).toEqual(Array(16).fill(["40px", "40px", "11px"]));

  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.locator(".action-panel")).toHaveCount(0);
  expect(await expectCardRows(page.locator(".claim-row"), "64px")).toEqual(assetCardStyle);
  expect(await readIconBoxes(".claim-row .list-icon")).toEqual(Array(8).fill(["40px", "40px", "11px"]));

  await page.locator('[data-wallet-section="permissions"]').click();
  await expect(page.locator(".action-panel")).toHaveCount(0);
  expect(await expectCardRows(page.locator(".permission-row"), "64px")).toEqual(assetCardStyle);
  expect(await readIconBoxes(".permission-row .list-icon")).toEqual(Array(8).fill(["40px", "40px", "11px"]));

  await page.locator('[data-wallet-section="assets"]').click();

  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  const desktopSendBox = await page.locator(".send-panel").boundingBox();
  expect(desktopSendBox.width).toBe(640);
  await expect(page.locator(".transfer-asset-row")).toHaveCount(0);

  await page.locator('#wallet-tabs [data-view="wallet-receive"]').click();
  await expect(page.locator(".receiver-card")).toBeVisible();
  await expect(page.locator(".transfer-asset-row")).toHaveCount(0);
  const [desktopMainBox, desktopReceiverBox, desktopQrBox] = await Promise.all([
    page.locator("#main-content").boundingBox(),
    page.locator(".receiver-card").boundingBox(),
    page.locator(".receiver-card-qr").boundingBox()
  ]);
  expect(Math.abs((desktopReceiverBox.x + desktopReceiverBox.width / 2) - (desktopMainBox.x + desktopMainBox.width / 2))).toBeLessThanOrEqual(1);
  expect(desktopReceiverBox.width).toBeLessThanOrEqual(300);
  expect(Math.abs(desktopQrBox.width - desktopQrBox.height)).toBeLessThanOrEqual(1);

  await page.locator('#wallet-tabs [data-view="activity"]').click();
  expect(await expectCardRows(page.locator(".activity-card-list .activity-row"), "64px")).toEqual(assetCardStyle);
  await expectCardRowGaps(page.locator(".activity-card-list .activity-row"));
  expect(await readIconBoxes(".activity-card-list .activity-icon")).toEqual(Array(7).fill(["40px", "40px", "11px"]));

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?view=wallet`);
  await expectCardRows(page.locator(".asset-row"), "88px", 88);
  await expectCardRowGaps(page.locator(".asset-row"));
  await page.locator('#wallet-tabs [data-view="wallet-send"]').click();
  const mobileSendBox = await page.locator(".send-panel").boundingBox();
  expect(mobileSendBox.x).toBeGreaterThanOrEqual(0);
  expect(mobileSendBox.x + mobileSendBox.width).toBeLessThanOrEqual(390);
  expect(mobileSendBox.width).toBeLessThan(390);
  await page.locator(".send-panel-footer").scrollIntoViewIfNeeded();
  await expect(page.locator(".send-panel-footer")).toBeVisible();
  await page.locator('#wallet-tabs [data-view="wallet-receive"]').click();
  const mobileReceiverBox = await page.locator(".receiver-card").boundingBox();
  await expect(page.locator(".receiver-card")).toBeVisible();
  await expect(page.locator(".transfer-asset-row")).toHaveCount(0);
  expect(mobileReceiverBox.x).toBeGreaterThanOrEqual(0);
  expect(mobileReceiverBox.x + mobileReceiverBox.width).toBeLessThanOrEqual(390);
  await page.locator('#wallet-tabs [data-view="activity"]').click();
  await expectCardRows(page.locator(".activity-card-list .activity-row"), "88px", 88);
  await expectCardRowGaps(page.locator(".activity-card-list .activity-row"));
});

test("typography LUT assigns Geist and Geist Mono to their semantic roles", async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(`${demoUrl}?view=home`);

  const [wordmark, topbarAddress, topbarContext, topbarPairHeight, copyHeight, balance, quickTitle, quickMeta, rowTitle, rowValue, navigation] = await page.evaluate(() => {
    const read = (selector) => {
      const style = getComputedStyle(document.querySelector(selector));
      return { family: style.fontFamily, size: parseFloat(style.fontSize), weight: style.fontWeight };
    };
    const brandBox = document.querySelector(".sidebar > .brand").getBoundingClientRect();
    const topbarBox = document.querySelector(".topbar").getBoundingClientRect();
    return [
      read(".sidebar .brand > span"),
      read(".page-heading h1.is-wallet-address"),
      read("#page-context"),
      document.querySelector(".page-heading").getBoundingClientRect().height,
      document.querySelector("#copy-wallet-address").getBoundingClientRect().height,
      read(".balance-amount"),
      read(".quick-action strong"),
      read(".quick-action small"),
      read(".attention-item .list-copy strong"),
      read(".attention-item .list-meta strong"),
      {
        sidebarLabel: read(".sidebar-label"),
        walletTab: read(".wallet-tab"),
        centerDelta: Math.abs((brandBox.top + brandBox.height / 2) - (topbarBox.top + topbarBox.height / 2))
      }
    ];
  });
  expect(wordmark.family).toContain("Geist");
  expect(wordmark.family).not.toContain("Rajdhani");
  expect(wordmark.weight).toBe("780");
  expect(topbarAddress.family).toBe(wordmark.family);
  expect(topbarAddress.weight).toBe("400");
  expect(topbarAddress.size).toBe(21);
  expect(topbarContext.size).toBe(13);
  expect(topbarPairHeight).toBeGreaterThan(copyHeight);
  expect(copyHeight).toBe(26);
  expect(balance.family).toContain("Geist Mono");
  expect(balance.size).toBeGreaterThanOrEqual(35);
  expect(balance.weight).toBe("700");
  expect(quickTitle.family).toContain("Geist");
  expect(quickTitle.size).toBe(16);
  expect(quickTitle.weight).toBe("700");
  expect(quickMeta.family).toContain("Geist");
  expect(quickMeta.family).not.toContain("Geist Mono");
  expect(quickMeta.size).toBeGreaterThanOrEqual(14);
  expect(rowTitle.family).toContain("Geist");
  expect(rowTitle.size).toBeGreaterThanOrEqual(15);
  expect(rowValue.family).toContain("Geist Mono");
  expect(rowValue.size).toBeGreaterThanOrEqual(14);
  expect(navigation.sidebarLabel.size).toBe(16);
  expect(navigation.walletTab.size).toBe(16);
  expect(navigation.centerDelta).toBeLessThanOrEqual(1);

  await page.goto(`${demoUrl}?view=wallet`);
  const [headerSize, assetName, assetNumber, assetKind] = await page.evaluate(() => {
    const read = (selector) => {
      const style = getComputedStyle(document.querySelector(selector));
      return { family: style.fontFamily, size: parseFloat(style.fontSize) };
    };
    return [
      parseFloat(getComputedStyle(document.querySelector(".asset-table-head")).fontSize),
      read(".asset-info strong"),
      read(".asset-number strong"),
      read(".object-kind")
    ];
  });
  expect(headerSize).toBeGreaterThanOrEqual(12);
  expect(assetName.family).toContain("Geist");
  expect(assetName.size).toBeGreaterThanOrEqual(15);
  expect(assetNumber.family).toContain("Geist Mono");
  expect(assetNumber.size).toBeGreaterThanOrEqual(15);
  expect(assetKind.family).toContain("Geist");
  expect(assetKind.family).not.toContain("Geist Mono");
  expect(assetKind.size).toBeGreaterThanOrEqual(12);
});

test("common settings and selected-wallet settings keep their scopes separate", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=general`);

  await expect(page.locator("#page-title")).toHaveClass(/is-settings-title/);
  await expect(page.locator("#page-title")).toHaveText("Settings");
  await expect(page.locator("#page-context")).toHaveText("Application preferences");
  await expect(page.getByLabel("Language")).toBeVisible();
  await expect(page.getByLabel("Notifications on")).toBeVisible();
  await expect(page.getByLabel("Wallet name")).toHaveCount(0);
  await expect(page.getByLabel("Default fee")).toHaveCount(0);
  await expect(page.getByText("Configuration file", { exact: true })).toHaveCount(0);
  await expect(page.locator("#wallet-tabs [data-settings-section]")).toHaveCount(4);
  await expect(page.locator("#wallet-tabs [data-settings-section=general]")).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(".settings-layout .context-rail")).toHaveCount(0);
  await expect(page.locator('[data-settings-section="security"], [data-settings-section="backup"], [data-settings-section="policies"], [data-settings-section="advanced"]')).toHaveCount(0);
  await expect(page.locator(".settings-detail > h2")).toHaveCount(0);

  await page.locator('#wallet-tabs [data-settings-section="appearance"]').click();
  await expect(page.locator('#wallet-tabs [data-settings-section="appearance"]')).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(".settings-detail > .settings-heading")).toHaveCount(0);
  await expect(page.locator(".palette-grid")).toHaveCount(1);
  await page.locator('#wallet-tabs [data-settings-section="reticulum"]').click();
  await expect(page.locator('#wallet-tabs [data-settings-section="reticulum"]')).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(".settings-network-tabs")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Reticulum" })).toBeVisible();

  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=security`);
  await expect(page.getByLabel("Lock app after")).toBeVisible();
  await expect(page.getByRole("button", { name: "Lock now" })).toBeVisible();
  await expect(page.locator(".wallet-settings-context .context-nav-item")).toHaveCount(5);
});

test("global and contextual Help are local, multilingual, focus-safe, and state-aware", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet&wallet=assets`);

  const globalHelp = page.locator(".help-button");
  await globalHelp.focus();
  await globalHelp.click();
  await expect(page.locator("#help-title")).toHaveText("Application help");
  await expect(page.locator(".help-panel")).toBeVisible();
  await expect(page.locator(".help-panel")).toContainText("Test Text");
  const globalClose = page.locator(".help-panel [data-help-close]");
  const globalSections = page.locator(".help-panel [data-help-section]");
  await expect(globalClose).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(globalSections.last()).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(globalClose).toBeFocused();
  await expect(page.locator(".is-help-highlighted")).toHaveCount(0);
  await page.keyboard.press("Escape");
  await expect(page.locator("#help-host")).toBeHidden();
  await expect(globalHelp).toBeFocused();

  const contextualHelp = page.getByRole("button", { name: "Help for this view" });
  const [globalHelpBox, contextualHelpBox, statusbarBox] = await Promise.all([
    globalHelp.boundingBox(),
    contextualHelp.boundingBox(),
    page.locator("#wallet-statusbar").boundingBox()
  ]);
  expect(Math.abs(globalHelpBox.x + globalHelpBox.width - contextualHelpBox.x - contextualHelpBox.width)).toBeLessThanOrEqual(1);
  expect(contextualHelpBox.y + contextualHelpBox.height).toBeLessThan(statusbarBox.y);
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  const contextualHelpAfterScroll = await contextualHelp.boundingBox();
  expect(Math.abs(contextualHelpAfterScroll.y - contextualHelpBox.y)).toBeLessThanOrEqual(1);
  await contextualHelp.click();
  await expect(page.locator("#help-title")).toHaveText("Assets");
  await expect(page.locator('[data-help-anchor="current-view"]')).toHaveClass(/is-help-highlighted/);
  await page.locator('[data-help-section="1"]').click();
  await expect(page.locator(".is-help-highlighted")).toHaveCount(0);
  await page.locator('[data-help-section="0"]').click();
  await expect(page.locator('[data-help-anchor="current-view"]')).toHaveClass(/is-help-highlighted/);
  await page.locator(".help-panel [data-help-close]").click();
  await expect(contextualHelp).toBeFocused();

  await page.goto(`${demoUrl}?view=home`);
  await page.getByRole("button", { name: "Help for this view" }).click();
  await expect(page.locator("#help-title")).toHaveText("Home");
  await page.keyboard.press("Escape");

  await page.context().setOffline(true);
  await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
  await globalHelp.click();
  await expect(page.locator("#help-title")).toHaveText("Application help");
  await page.keyboard.press("Escape");
  await page.context().setOffline(false);

  await page.goto(`${demoUrl}?view=settings&settings=general`);
  await page.selectOption('[data-config-control="language"]', "ru");
  await page.locator(".help-button").click();
  await expect(page.locator("#help-title")).toHaveText("Справка приложения");
  await expect(page.locator(".help-panel")).toContainText("работает без интернета");
  await expect(page.locator("#toast-region")).toHaveCSS("visibility", "hidden");
});

test("mobile Help is a bottom sheet and compact rows keep logical fields aligned", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=general`);

  const rowGeometry = await page.locator(".compact-row").first().evaluate((row) => {
    const parts = [...row.children].map((element) => element.getBoundingClientRect());
    return {
      row: row.getBoundingClientRect(),
      centers: parts.filter((box) => box.width > 0).map((box) => Math.round(box.top + box.height / 2)),
      documentWidth: document.documentElement.scrollWidth,
      viewportWidth: window.innerWidth
    };
  });
  expect(Math.max(...rowGeometry.centers) - Math.min(...rowGeometry.centers)).toBeLessThanOrEqual(4);
  expect(rowGeometry.documentWidth).toBeLessThanOrEqual(rowGeometry.viewportWidth);
  const contextualHelp = page.getByRole("button", { name: "Help for this view" });
  const contextHelpBox = await contextualHelp.boundingBox();
  expect(await contextualHelp.evaluate((button) => getComputedStyle(button.parentElement).position)).toBe("fixed");
  expect(contextHelpBox.x + contextHelpBox.width).toBeLessThanOrEqual(390 - 16 + 1);
  expect(contextHelpBox.y + contextHelpBox.height).toBeLessThanOrEqual(844 - 16 + 1);
  await page.evaluate(() => window.scrollTo(0, document.body.scrollHeight));
  const contextHelpAfterScroll = await contextualHelp.boundingBox();
  expect(Math.abs(contextHelpAfterScroll.x - contextHelpBox.x)).toBeLessThanOrEqual(1);
  expect(Math.abs(contextHelpAfterScroll.y - contextHelpBox.y)).toBeLessThanOrEqual(1);

  await page.locator("#mobile-menu-button").click();
  await page.getByRole("button", { name: "Help", exact: true }).click();
  const sheet = await page.locator(".help-panel").boundingBox();
  expect(Math.round(sheet.x)).toBe(0);
  expect(Math.round(sheet.width)).toBe(390);
  expect(Math.round(sheet.y + sheet.height)).toBe(844);
  const closeSize = await page.locator(".help-panel [data-help-close]").boundingBox();
  expect(closeSize.width).toBeGreaterThanOrEqual(44);
  expect(closeSize.height).toBeGreaterThanOrEqual(44);
});

test("asset detail key and value remain one mobile row with full values accessible", async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 800 });
  await page.goto(`${demoUrl}?view=wallet&wallet=assets`);
  await page.locator('[data-open-flow="asset-detail"]').first().click();

  const rows = page.locator(".asset-detail-row");
  await expect(rows).toHaveCount(6);
  const geometry = await rows.evaluateAll((items) => items.map((row) => {
    const label = row.children[0].getBoundingClientRect();
    const value = row.children[1].getBoundingClientRect();
    return {
      labelCenter: Math.round(label.top + label.height / 2),
      valueCenter: Math.round(value.top + value.height / 2)
    };
  }));
  geometry.forEach(({ labelCenter, valueCenter }) => expect(Math.abs(labelCenter - valueCenter)).toBeLessThanOrEqual(2));
  await expect(rows.nth(3).locator("strong")).toHaveAttribute("title", /.+/);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);

  const dialogHelp = page.locator(".dialog-help-button");
  await dialogHelp.click();
  await expect(page.locator("#help-title")).toHaveText("Asset details");
  await expect(page.locator(".help-panel")).toBeVisible();
  await expect(page.locator("#dialog-content")).toHaveJSProperty("inert", true);
  await expect(page.locator("#flow-dialog .dialog-shell")).toHaveClass(/is-help-highlighted/);
  await page.keyboard.press("Escape");
  await expect(page.locator("#help-host")).toBeHidden();
  await expect(page.getByRole("heading", { name: "Asset details" })).toBeVisible();
  await expect(dialogHelp).toBeFocused();
});

test("common Settings uses the same topbar typography as OnionNet", async ({ page }) => {
  const readTitleType = (selector) => page.locator(selector).evaluate((node) => {
    const style = getComputedStyle(node);
    return [style.fontFamily, style.fontSize, style.fontWeight, style.lineHeight, style.letterSpacing];
  });

  await page.goto(`${demoUrl}?view=settings&settings=general`);
  await expect(page.locator("#page-context")).toHaveText("Application preferences");
  const settingsType = await readTitleType("#page-title");

  await page.goto(`${demoUrl}?view=telemetry&telemetry=onionnet`);
  const onionnetType = await readTitleType("#page-title");

  expect(settingsType).toEqual(onionnetType);
});

test("target-only capabilities are disclosed", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=network&network=onionnet`);
  await expect(page.getByText("Target Phase 080 simulation")).toBeVisible();
  await expect(page.getByText(/current live network RPC is stubbed/i)).toBeVisible();

  const onionnet = page.locator('#wallet-tabs [data-settings-section="onionnet"]');
  await expect(onionnet).toHaveAttribute("aria-selected", "true");
  await expect(page.locator(".settings-network-tabs")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Carriers" })).toHaveCount(0);
  await expect(page.locator(".settings-layout .context-rail")).toHaveCount(0);

  await onionnet.click();
  await expect(onionnet).toHaveAttribute("aria-selected", "true");
  await expect(onionnet).toHaveAttribute("aria-current", "page");
  await expect(page.getByRole("heading", { name: "OnionNet" })).toBeVisible();

  await expect(page.locator('[data-settings-section="security"], [data-settings-section="backup"], [data-settings-section="policies"], [data-settings-section="advanced"]')).toHaveCount(0);
});

test("telemetry tabs use the assigned icon LUT", async ({ page }) => {
  await page.goto(`${demoUrl}?view=telemetry&telemetry=reticulum`);

  const reticulumIcons = {
    overview: "#i-overview",
    node: "#i-reticulum-node",
    interfaces: "#i-reticulum-interface",
    entrypoints: "#i-entry",
    paths: "#i-reticulum-paths",
    probes: "#i-probe",
    links: "#i-reticulum-link"
  };

  for (const [tabId, iconId] of Object.entries(reticulumIcons)) {
    await expect(page.locator(`#reticulum-tab-${tabId} use`)).toHaveAttribute("href", iconId);
  }
  await expect(page.locator("#i-reticulum-paths")).toHaveAttribute("viewBox", "0 0 24 24");
  await expect(page.locator("#i-reticulum-paths path")).toHaveAttribute("d", "M7 5h3a4 4 0 0 1 4 4v3a4 4 0 0 0 4 4h1M7 19h2a5 5 0 0 0 5-5V9a2 2 0 0 1 2-2h1");

  await page.goto(`${demoUrl}?view=telemetry&telemetry=onionnet`);
  const onionnetIcons = {
    overview: "#i-overview",
    queues: "#i-queue",
    probation: "#i-probe",
    ingress: "#i-entry"
  };
  for (const [tabId, iconId] of Object.entries(onionnetIcons)) {
    await expect(page.locator(`#onionnet-tab-${tabId} use`)).toHaveAttribute("href", iconId);
  }

  await page.goto(`${demoUrl}?view=telemetry&telemetry=aggregators`);
  await expect(page.locator("#aggregators-tab-overview use")).toHaveAttribute("href", "#i-overview");

});

test("appearance palettes and YAML highlighting stay application-wide", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=appearance`);

  await expect(page.locator(".palette-grid [data-palette]")).toHaveCount(4);
  const paletteIds = await page.locator(".palette-grid [data-palette]").evaluateAll((cards) => cards.map((card) => card.dataset.palette));
  expect(paletteIds).toEqual([
    "z00z-default",
    "black-gold-elegance",
    "moonlit-stroll",
    "walking-at-night"
  ]);
  await expect(page.locator(".code-theme-card")).toHaveCount(4);
  await page.locator('[data-code-theme="night-owl"]').click();
  await expect(page.locator("html")).toHaveAttribute("data-code-theme", "night-owl");
  await page.locator('[data-palette="moonlit-stroll"]').click();
  await expect(page.locator("html")).toHaveAttribute("data-palette", "moonlit-stroll");
  await page.getByLabel("Custom brand color").evaluate((input) => {
    input.value = "#f4c95d";
    input.dispatchEvent(new Event("change", { bubbles: true }));
  });
  await expect(page.locator("html")).toHaveCSS("--brand", "#f4c95d");
  await page.getByLabel("Text scale").selectOption("110");
  await expect(page.locator("html")).toHaveAttribute("data-text-scale", "110");

  await page.locator('[data-wallet-id="savings"]').click();
  await page.locator('#wallet-tabs [data-view="wallet-settings"]').click();
  await expect(page.locator(".wallet-settings-context .context-nav-item")).toHaveCount(5);
  await expect(page.locator('[data-wallet-settings-section="advanced"] use')).toHaveAttribute("href", "#i-advanced");
  await expect(page.getByRole("button", { name: "Appearance", exact: true })).toHaveCount(0);
  await expect(page.locator("html")).toHaveAttribute("data-code-theme", "night-owl");
  await page.getByRole("button", { name: /Advanced/ }).click();
  const walletYaml = page.locator("#wallet-settings-yaml");
  await expect(walletYaml).not.toHaveValue(/code_theme:/);
  await walletYaml.fill("schema_version: 1\nwallet:\n  id: savings\n  chain: \"mainnet\"\n  display:\n    currency: Z00Z\n  transactions:\n    default_fee: \"0.010\"");
  await page.getByRole("button", { name: "Apply locally" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-code-theme", "night-owl");
  await expect(walletYaml).not.toHaveValue(/code_theme:/);
});

test("colors.css is the single source for palette, semantic, and YAML preview colours", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=appearance`);

  const [styleEntry, componentStyles, lutSource] = await page.evaluate(async () => {
    const [entry, foundation, components, lut] = await Promise.all([
      fetch("styles.css").then((response) => response.text()),
      fetch("styles/foundation.css").then((response) => response.text()),
      fetch("styles/components.css").then((response) => response.text()),
      fetch("styles/colors.css").then((response) => response.text())
    ]);
    return [entry, [foundation, components].join("\n"), lut];
  });
  expect(styleEntry).toMatch(/@import url\("styles\/colors\.css(?:\?v=[a-f0-9]{40})?"\)/i);
  expect(componentStyles).not.toMatch(/#[0-9a-f]{3,8}\b|rgba?\(/i);
  expect(lutSource).toContain("--lut-z00z-dark-brand");
  expect(lutSource).toContain("--lut-code-night-owl-keyword");

  const swatchColours = await page.locator('.palette-card[data-palette="z00z-default"] .palette-swatches i').evaluateAll((swatches) => swatches.map((swatch) => getComputedStyle(swatch).backgroundColor));
  expect(swatchColours).toHaveLength(5);
  expect(new Set(swatchColours).size).toBe(5);
  expect(swatchColours).not.toContain("rgba(0, 0, 0, 0)");

  const themeToggle = page.locator("[data-theme-toggle]");
  await expect(themeToggle).toHaveCount(1);
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(themeToggle).toHaveText("Dark");
  await expect(themeToggle).toHaveAttribute("aria-label", "Switch to light mode");
  await themeToggle.click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(themeToggle).toHaveText("Light");
  await expect(themeToggle).toHaveAttribute("aria-label", "Switch to dark mode");
  await themeToggle.click();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await themeToggle.click();
  await page.locator('[data-palette="moonlit-stroll"]').click();
  const semanticColours = await page.locator("html").evaluate((root) => {
    const style = getComputedStyle(root);
    return [style.getPropertyValue("--brand").trim(), style.getPropertyValue("--rail").trim(), style.getPropertyValue("--success").trim()];
  });
  expect(semanticColours).toEqual(["#9c6500", "#006f94", "#087a52"]);

  await page.locator('.code-theme-card[data-code-theme="night-owl"]').click();
  await expect(page.locator('.code-theme-card[data-code-theme="night-owl"] .code-theme-preview')).toHaveCSS("background-color", "rgb(1, 22, 39)");
});

test("mobile wallet drawer keeps actions fixed and scrolls only an overflowing wallet list", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(demoUrl);
  await page.locator("#mobile-menu-button").click();
  await page.locator('[data-mobile-popup-open="wallets"]').click();

  const drawer = page.locator('#mobile-popup-menu[data-popup-type="wallets"]');
  const walletList = drawer.locator(".mobile-popup-list");
  const actions = drawer.locator(".mobile-wallet-actions");
  const addWallet = actions.locator('[data-mobile-popup-action="add-wallet"]');
  const removeWallet = actions.locator('[data-mobile-popup-action="remove-wallet"]');

  await expect(addWallet).toHaveText("Add wallet");
  await expect(removeWallet).toHaveText("Remove wallet");
  await expect(removeWallet).toBeEnabled();
  await expect(walletList).toHaveCSS("overflow-y", "auto");

  const initialOverflow = await walletList.evaluate((list) => ({
    clientHeight: list.clientHeight,
    scrollHeight: list.scrollHeight,
  }));
  expect(initialOverflow.scrollHeight).toBeLessThanOrEqual(initialOverflow.clientHeight);

  const fixedActionTop = (await actions.boundingBox()).y;
  await walletList.evaluate((list) => {
    const source = list.querySelector("[data-mobile-select-wallet]");
    for (let index = 0; index < 12; index += 1) {
      const clone = source.cloneNode(true);
      clone.removeAttribute("aria-current");
      clone.classList.remove("is-active");
      clone.dataset.mobileSelectWallet = `overflow-${index}`;
      clone.querySelector("span:nth-child(2)").textContent = `Overflow wallet ${index + 1}`;
      clone.querySelector(".icon")?.remove();
      list.append(clone);
    }
    list.scrollTop = list.scrollHeight;
  });

  const overflowGeometry = await walletList.evaluate((list) => ({
    clientHeight: list.clientHeight,
    scrollHeight: list.scrollHeight,
    scrollTop: list.scrollTop,
  }));
  expect(overflowGeometry.scrollHeight).toBeGreaterThan(overflowGeometry.clientHeight);
  expect(overflowGeometry.scrollTop).toBeGreaterThan(0);
  expect(Math.abs((await actions.boundingBox()).y - fixedActionTop)).toBeLessThanOrEqual(1);

  await addWallet.click();
  await expect(page.getByRole("heading", { name: "Add wallet" })).toBeVisible();
  await page.keyboard.press("Escape");

  await page.locator("#mobile-menu-button").click();
  await page.locator('[data-mobile-popup-open="wallets"]').click();
  await page.locator('[data-mobile-popup-action="remove-wallet"]').click();
  await expect(page.getByRole("heading", { name: "Remove wallet profiles" })).toBeVisible();
});

test("responsive navigation, hover, focus, and overflow contract", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto(`${demoUrl}?view=settings&settings=network&network=onionnet`);

  const activeContext = page.locator('#wallet-tabs [data-settings-section="onionnet"].is-active');
  await expect(activeContext).toHaveText("OnionNet");
  const activeBox = await activeContext.boundingBox();
  expect(activeBox.x).toBeGreaterThanOrEqual(0);
  expect(activeBox.x + activeBox.width).toBeLessThanOrEqual(390);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(390);

  await page.goto(demoUrl);
  await expect(page.locator(".bottom-nav")).toHaveCount(0);
  const mobileNavigationLayout = await page.evaluate(() => {
    const box = (selector) => {
      const bounds = document.querySelector(selector)?.getBoundingClientRect();
      return bounds ? { left: bounds.left, right: bounds.right, top: bounds.top, bottom: bounds.bottom } : null;
    };
    return {
      bar: box("#primary-topbar"),
      menu: box("#mobile-menu-button"),
      logo: box(".mobile-nav-brand"),
      tabs: box("#wallet-tabs"),
      topbarDisplay: getComputedStyle(document.querySelector(".topbar")).display
    };
  });
  expect(mobileNavigationLayout.topbarDisplay).toBe("flex");
  [mobileNavigationLayout.bar, mobileNavigationLayout.menu, mobileNavigationLayout.logo, mobileNavigationLayout.tabs].forEach((box) => {
    expect(box.left).toBeGreaterThanOrEqual(0);
    expect(box.right).toBeLessThanOrEqual(390);
  });
  expect(mobileNavigationLayout.menu.right).toBeLessThanOrEqual(mobileNavigationLayout.logo.left);
  expect(mobileNavigationLayout.logo.right).toBeLessThanOrEqual(mobileNavigationLayout.tabs.left);

  await page.locator("#mobile-menu-button").click();
  await expect(page.locator("#mobile-popup-menu")).toBeVisible();
  await expect(page.locator("#mobile-popup-menu .mobile-popup-item")).toHaveText(["Wallets", "Network", "Settings", "Help", "Log out"]);
  await page.locator('[data-mobile-popup-open="wallets"]').click();
  await expect(page.locator("[data-mobile-select-wallet] > span:nth-child(2)")).toHaveText(["Everyday", "Savings", "Travel"]);
  await expect(page.locator(".mobile-wallet-actions .mobile-popup-item")).toHaveText(["Add wallet", "Remove wallet"]);
  await page.locator('[data-mobile-select-wallet="savings"]').click();
  await expect(page.locator("#page-context")).toHaveText("Savings wallet");

  const assetTab = page.locator('#wallet-tabs [data-mobile-popup="assets"]');
  await assetTab.click();
  await expect(page.locator("[data-mobile-wallet-section]")).toHaveText(["Assets", "Vouchers", "Permissions"]);
  await page.locator('[data-mobile-wallet-section="vouchers"]').click();
  await assetTab.click();
  await expect(page.locator('[data-mobile-wallet-section="vouchers"]')).toHaveAttribute("aria-current", "page");
  await page.locator('[data-mobile-wallet-section="assets"]').click();

  const mobileAssetGeometry = await page.locator(".asset-row").evaluateAll((rows) => rows.slice(0, 6).map((row) => {
    const rowBox = row.getBoundingClientRect();
    const identityBox = row.querySelector(".asset-identity-button").getBoundingClientRect();
    const numberBoxes = [...row.querySelectorAll(".asset-number")].map((number) => number.getBoundingClientRect());
    return {
      row: { left: rowBox.left, right: rowBox.right, height: rowBox.height },
      identityBottom: identityBox.bottom,
      numberTop: Math.min(...numberBoxes.map((box) => box.top)),
      numbersInside: numberBoxes.every((box) => box.left >= rowBox.left && box.right <= rowBox.right),
      numbersSeparated: numberBoxes.every((box, index) => index === 0 || numberBoxes[index - 1].right <= box.left)
    };
  }));
  mobileAssetGeometry.forEach((geometry) => {
    expect(Math.round(geometry.row.height)).toBeGreaterThanOrEqual(88);
    expect(Math.round(geometry.row.height)).toBeLessThanOrEqual(89);
    expect(geometry.identityBottom).toBeLessThanOrEqual(geometry.numberTop);
    expect(geometry.numbersInside).toBe(true);
    expect(geometry.numbersSeparated).toBe(true);
  });
  await expect(page.locator(".asset-number-label").first()).toBeVisible();

  await page.locator("#mobile-menu-button").click();
  await page.locator('[data-mobile-popup-open="network"]').click();
  await page.locator('[data-mobile-select-network="reticulum"]').click();
  await expect(page.locator("#page-title")).toHaveText("Reticulum");
  await page.locator("#mobile-menu-button").click();
  await page.locator('[data-mobile-popup-open="wallets"]').click();
  await page.locator('[data-mobile-select-wallet="everyday"]').click();

  await expect(page.locator("#wallet-statusbar")).toHaveCSS("position", "static");

  await page.setViewportSize({ width: 320, height: 700 });
  await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=advanced`);
  const compactActiveContext = page.locator(".wallet-settings-context .context-nav-item.is-active");
  await expect(compactActiveContext.locator("strong")).toHaveText("Advanced");
  await expect(page.locator(".wallet-settings-view .context-rail")).toBeHidden();
  const walletSettingsTab = page.locator('#wallet-tabs [data-mobile-popup="wallet-settings"]');
  await walletSettingsTab.scrollIntoViewIfNeeded();
  await walletSettingsTab.click();
  await expect(page.locator('[data-mobile-wallet-settings-section="advanced"]')).toHaveAttribute("aria-current", "page");
  const compactPopupBox = await page.locator("#mobile-popup-menu").boundingBox();
  expect(compactPopupBox.x).toBeGreaterThanOrEqual(0);
  expect(compactPopupBox.x + compactPopupBox.width).toBeLessThanOrEqual(320);
  await page.keyboard.press("Escape");
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);

  await page.goto(`${demoUrl}?view=wallet`);
  const compactFilterHeight = await page.locator(".choice-chip").first().evaluate((node) => node.getBoundingClientRect().height);
  expect(Math.round(compactFilterHeight)).toBeGreaterThanOrEqual(44);
  const compactNavigationBoxes = await page.locator("#primary-topbar").evaluate((navigation) => [
    navigation.querySelector("#mobile-menu-button"),
    navigation.querySelector(".mobile-nav-brand"),
    navigation.querySelector("#wallet-tabs")
  ].map((element) => {
    const box = element.getBoundingClientRect();
    return { left: box.left, right: box.right };
  }));
  compactNavigationBoxes.forEach((box, index) => {
    expect(box.left).toBeGreaterThanOrEqual(0);
    expect(box.right).toBeLessThanOrEqual(320);
    if (index > 0) expect(compactNavigationBoxes[index - 1].right).toBeLessThanOrEqual(box.left);
  });
  const compactAssetOverlap = await page.locator(".asset-row").evaluateAll((rows) => rows.slice(0, 6).some((row) => {
    const identity = row.querySelector(".asset-identity-button").getBoundingClientRect();
    const numbers = [...row.querySelectorAll(".asset-number")].map((number) => number.getBoundingClientRect());
    return numbers.some((number, index) => identity.bottom > number.top
      || number.right > row.getBoundingClientRect().right
      || (index > 0 && numbers[index - 1].right > number.left));
  }));
  expect(compactAssetOverlap).toBe(false);
  expect(await page.evaluate(() => document.documentElement.scrollWidth)).toBeLessThanOrEqual(320);

  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto(`${demoUrl}?view=settings&settings=general`);
  const [settingsTabBox, detailBox] = await Promise.all([
    page.locator('#wallet-tabs [data-settings-section="general"]').boundingBox(),
    page.locator(".settings-detail").boundingBox()
  ]);
  expect(settingsTabBox.x).toBeGreaterThanOrEqual(detailBox.x);
  await expect(page.locator(".settings-layout .context-rail")).toHaveCount(0);
  await expect(page.locator('#wallet-tabs [data-settings-section="general"]')).toHaveAttribute("aria-selected", "true");

  const [languageBox, notificationBox] = await Promise.all([
    page.getByLabel("Language").boundingBox(),
    page.getByLabel("Notifications on").boundingBox()
  ]);
  expect(Math.abs((languageBox.x + languageBox.width) - (notificationBox.x + notificationBox.width))).toBeLessThanOrEqual(1);

  const activeGlobal = page.locator('.nav-item[data-view="settings"]');
  const [activeGlobalColor, brandStrongColor] = await activeGlobal.evaluate((node) => {
    const probe = document.createElement("span");
    probe.style.color = "var(--brand-strong)";
    document.body.append(probe);
    const result = [getComputedStyle(node).color, getComputedStyle(probe).color];
    probe.remove();
    return result;
  });
  expect(activeGlobalColor).not.toBe(brandStrongColor);

  await page.setViewportSize({ width: 1920, height: 700 });
  await page.goto(demoUrl);
  const [tabBox, topbarBox] = await Promise.all([
    page.locator("#wallet-tabs .wallet-tab").first().boundingBox(),
    page.locator("#primary-topbar").boundingBox()
  ]);
  expect(tabBox.y).toBe(topbarBox.y);
  expect(Math.abs(tabBox.height - topbarBox.height)).toBeLessThanOrEqual(1);
  const [tabStart, addressEdge] = await Promise.all([
    page.locator("#wallet-tabs .wallet-tab").first().evaluate((node) => node.getBoundingClientRect().left),
    page.locator(".topbar-address-group").evaluate((node) => node.getBoundingClientRect().right)
  ]);
  expect(Math.abs(tabStart - addressEdge)).toBeLessThanOrEqual(1);
  const workspaceTabStarts = [tabStart];
  for (const route of [
    `${demoUrl}?view=telemetry&telemetry=reticulum`,
    `${demoUrl}?view=settings&settings=general`
  ]) {
    await page.goto(route);
    await expect(page.locator("#wallet-tabs .wallet-tab").first()).toBeVisible();
    workspaceTabStarts.push(await page.locator("#wallet-tabs .wallet-tab").first().evaluate((node) => node.getBoundingClientRect().left));
  }
  expect(Math.max(...workspaceTabStarts) - Math.min(...workspaceTabStarts)).toBeLessThanOrEqual(1);
  await page.goto(demoUrl);
  for (const width of [1280, 1024]) {
    await page.setViewportSize({ width, height: 700 });
    const segments = await page.evaluate(() => {
      const box = (selector) => document.querySelector(selector).getBoundingClientRect();
      return {
        topbar: box("#primary-topbar"),
        address: box(".topbar-address-group"),
        tabs: box("#wallet-tabs"),
        actions: box(".topbar-actions")
      };
    });
    expect(segments.address.right).toBeLessThanOrEqual(segments.tabs.left);
    expect(segments.tabs.right).toBeLessThanOrEqual(segments.actions.left);
    expect(segments.actions.right).toBeLessThanOrEqual(segments.topbar.right);
  }
  await page.setViewportSize({ width: 1920, height: 700 });
  await page.evaluate(() => window.scrollTo(0, 500));
  await page.waitForTimeout(100);
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);
  const [stickyTabBox, stickyTopbarBox] = await Promise.all([
    page.locator("#wallet-tabs").boundingBox(),
    page.locator("#primary-topbar").boundingBox()
  ]);
  expect(Math.abs(stickyTabBox.y - stickyTopbarBox.y)).toBeLessThanOrEqual(1);
  const [topbarBackground, canvasBackground] = await page.locator("#primary-topbar").evaluate((topbar) => {
    const probe = document.createElement("span");
    probe.style.background = "var(--bg-canvas)";
    document.body.append(probe);
    const result = [getComputedStyle(topbar).backgroundColor, getComputedStyle(probe).backgroundColor];
    probe.remove();
    return result;
  });
  expect(topbarBackground).toBe(canvasBackground);

  await page.goto(`${demoUrl}?view=home`);
  const quickAction = page.locator('.quick-action[data-view="wallet-send"]');
  const before = await quickAction.evaluate((node) => getComputedStyle(node).backgroundColor);
  await quickAction.hover();
  await page.waitForTimeout(220);
  const after = await quickAction.evaluate((node) => getComputedStyle(node).backgroundColor);
  expect(after).not.toBe(before);

  await page.keyboard.press("Tab");
  const focusOutline = await page.evaluate(() => getComputedStyle(document.activeElement).outlineStyle);
  expect(focusOutline).not.toBe("none");
});

test("language catalogues stay complete and relocalize the shell without changing wallet data", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=general`);

  const reports = await page.evaluate(() => window.Z00ZI18n.auditCatalogues());
  expect(reports).toHaveLength(10);
  expect(reports.every((report) => report.ready)).toBe(true);
  expect(reports.map((report) => report.language)).toEqual(["en", "ru", "fr", "de", "es", "pt", "ko", "tr", "ja", "zh-Hans"]);

  const languageControl = page.locator('[data-config-control="language"]');
  await expect(languageControl.locator("option")).toHaveText(["English", "Русский", "Français", "Deutsch", "Español", "Português", "한국어", "Türkçe", "日本語", "简体中文"]);
  for (const [language, title] of [["pt", "Definições"], ["ko", "설정"], ["tr", "Ayarlar"]]) {
    await languageControl.selectOption(language);
    await expect(page.locator("html")).toHaveAttribute("lang", language);
    await expect(page.locator("#page-title")).toHaveText(title);
  }

  await languageControl.selectOption("ru");
  await expect(page.locator("html")).toHaveAttribute("lang", "ru");
  await expect(page.getByLabel("Язык")).toBeVisible();
  await expect(page.locator("#page-title")).toHaveText("Настройки");
  await expect(page.locator("#page-context")).toHaveText("Настройки приложения");
  await expect(page.locator(".sidebar-label").first()).toHaveText("Кошельки");
  await expect(page.locator('[data-wallet-id="everyday"] strong')).toHaveText("Everyday");
  await expect(page.locator('[data-wallet-id="savings"] strong')).toHaveText("Savings");
  await expect(page.locator('[data-wallet-id="travel"] strong')).toHaveText("Travel");
  await expect(page.locator('[data-wallet-id="everyday"] small')).toContainText("доступно");

  await languageControl.selectOption("ja");
  await expect(page.locator("html")).toHaveAttribute("lang", "ja");
  await expect(page.getByLabel("言語")).toBeVisible();
  await expect(page.locator("#page-title")).toHaveText("設定");
  await expect(page.locator('[data-wallet-id="everyday"] strong')).toHaveText("Everyday");
  await expect(page.locator('[data-wallet-id="savings"] strong')).toHaveText("Savings");
  await expect(page.locator('[data-wallet-id="travel"] strong')).toHaveText("Travel");
  await expect(page.locator('[data-wallet-id="everyday"] small')).toContainText("利用可能");
  await expect(page.locator("#network-nav")).toContainText("OnionNet");
  await expect(page.locator("#network-nav")).toContainText("Reticulum");

  await languageControl.selectOption("zh-Hans");
  await expect(page.locator("html")).toHaveAttribute("lang", "zh-Hans");
  await expect(page.getByLabel("语言")).toBeVisible();
  await page.locator('[data-wallet-id="everyday"]').click();
  await page.locator('#wallet-tabs [data-view="activity"]').click();
  await expect(page.locator("#main-content")).toContainText("向 Mira 付款");
  await expect(page.locator("#main-content")).toContainText("7月21日");
  await expect(page.locator("#main-content")).not.toContainText("Everything that changed");
  await expect(page.locator("#main-content")).not.toContainText("21 Jul");
  await page.locator('[data-open-activity="tx-7f31"]').click();
  await expect(page.getByText("历史详情")).toBeVisible();
  await expect(page.getByRole("button", { name: "复制收据" })).toBeVisible();
  await expect(page.getByRole("button", { name: "关闭" })).toBeVisible();
  await page.getByRole("button", { name: "关闭" }).click();
  await page.locator('#wallet-tabs [data-view="staking"]').click();
  await expect(page.locator("#main-content")).toContainText("准备质押");

  await page.locator('[data-wallet-id="everyday"]').click();
  await expect(page.locator(".context-rail-label")).toHaveCount(0);
  await expect(page.locator("#main-content .page-intro")).toHaveCount(0);
  await expect(page.locator("#main-content .money-summary")).toHaveCount(0);
  await expect(page.locator("#main-content")).toContainText("全部");
  await expect(page.locator("#main-content")).toContainText("名称余额价值价格");
  await expect(page.locator("#main-content")).not.toContainText(/\[assets\./);
  await expect(page.locator("#main-content")).toContainText("Z00Z");
  await expect(page.locator("#main-content")).not.toContainText("Acme Credits");

  const localized = await page.evaluate(() => ({
    russianNumber: window.Z00ZI18n.formatNumber(12480.75, "ru", "ru-RU", { minimumFractionDigits: 2 }),
    japaneseRate: window.Z00ZI18n.formatBitrate(12500, "ja", "ja-JP")
  }));
  expect(localized.russianNumber).toContain(",75");
  expect(localized.japaneseRate).toContain("kbit/s");
});

test("settings tabs share the standard single-topbar tab treatment", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet`);
  const assetTabStyle = await page.locator('#wallet-tabs [data-view="wallet"]').evaluate((element) => {
    const style = getComputedStyle(element);
    return [style.fontFamily, style.fontSize, style.minHeight];
  });

  await page.goto(`${demoUrl}?view=settings&settings=general`);
  const settingsTab = page.locator('#settings-tab-general');
  const [settingsTabStyle, tabBarJustification, tabsParent] = await Promise.all([
    settingsTab.evaluate((element) => {
      const style = getComputedStyle(element);
      return [style.fontFamily, style.fontSize, style.minHeight];
    }),
    page.locator("#wallet-tabs").evaluate((element) => getComputedStyle(element).justifyContent),
    page.locator("#wallet-tabs").evaluate((element) => element.parentElement.id)
  ]);

  expect(settingsTabStyle).toEqual(assetTabStyle);
  expect(tabBarJustification).toBe("flex-start");
  expect(tabsParent).toBe("primary-topbar");
  expect(
    await page.locator("#wallet-tabs [data-settings-section]").evaluateAll((tabs) =>
      tabs.map((tab) => tab.dataset.settingsSection)
    )
  ).toEqual(["general", "reticulum", "onionnet", "appearance"]);
});

test("all app settings sections share one centered compact card", async ({ page }) => {
  const sections = ["general", "reticulum", "onionnet", "appearance"];
  const desktopWidths = [];

  await page.setViewportSize({ width: 1280, height: 800 });
  for (const section of sections) {
    await page.goto(`${demoUrl}?view=settings&settings=${section}`);
    const geometry = await page.locator(".settings-layout--full").evaluate((card) => {
      const cardBox = card.getBoundingClientRect();
      const mainBox = document.querySelector("#main-content").getBoundingClientRect();
      return {
        width: cardBox.width,
        centerOffset: Math.abs(
          (cardBox.left + cardBox.width / 2) - (mainBox.left + mainBox.width / 2)
        )
      };
    });
    desktopWidths.push(Math.round(geometry.width));
    expect(geometry.width).toBeLessThanOrEqual(640.5);
    expect(geometry.centerOffset).toBeLessThanOrEqual(1);
  }
  expect(new Set(desktopWidths).size).toBe(1);

  await page.setViewportSize({ width: 390, height: 844 });
  for (const section of sections) {
    await page.goto(`${demoUrl}?view=settings&settings=${section}`);
    const geometry = await page.locator(".settings-layout--full").evaluate((card) => {
      const cardBox = card.getBoundingClientRect();
      const mainBox = document.querySelector("#main-content").getBoundingClientRect();
      return {
        width: cardBox.width,
        mainWidth: mainBox.width,
        centerOffset: Math.abs(
          (cardBox.left + cardBox.width / 2) - (mainBox.left + mainBox.width / 2)
        )
      };
    });
    expect(geometry.width).toBeLessThanOrEqual(geometry.mainWidth);
    expect(geometry.centerOffset).toBeLessThanOrEqual(1);
  }
});

test("all wallet settings sections share the Assets rail gap and one compact card width", async ({ page }) => {
  const sections = ["general", "security", "backup", "policies", "advanced"];
  const desktopWidths = [];

  await page.setViewportSize({ width: 1280, height: 800 });
  for (const section of sections) {
    await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=${section}`);
    const geometry = await page.locator(".wallet-settings-view .settings-layout").evaluate((layout) => {
      const layoutBox = layout.getBoundingClientRect();
      const railBox = layout.querySelector(".context-rail").getBoundingClientRect();
      const cardBox = layout.querySelector(".settings-detail").getBoundingClientRect();
      const gap = Number.parseFloat(getComputedStyle(layout).columnGap);
      return {
        width: cardBox.width,
        expectedGap: gap,
        actualGap: cardBox.left - railBox.right,
        contained: cardBox.right <= layoutBox.right
      };
    });
    desktopWidths.push(Math.round(geometry.width));
    expect(geometry.width).toBeLessThanOrEqual(520.5);
    expect(Math.abs(geometry.actualGap - geometry.expectedGap)).toBeLessThanOrEqual(1);
    expect(geometry.contained).toBe(true);
    const labelsFit = await page.locator(".wallet-settings-view .compact-row-label").evaluateAll(
      (labels) => labels.every((label) => label.scrollWidth <= label.clientWidth + 1)
    );
    expect(labelsFit).toBe(true);
  }
  expect(new Set(desktopWidths).size).toBe(1);

  for (const viewport of [{ width: 390, height: 844 }, { width: 320, height: 800 }]) {
    await page.setViewportSize(viewport);
    for (const section of sections) {
      await page.goto(`${demoUrl}?view=wallet-settings&walletSettings=${section}`);
      const geometry = await page.locator(".wallet-settings-view .settings-layout").evaluate((layout) => {
        const layoutBox = layout.getBoundingClientRect();
        const cardBox = layout.querySelector(".settings-detail").getBoundingClientRect();
        return {
          width: cardBox.width,
          layoutWidth: layoutBox.width,
          centerOffset: Math.abs(
            (cardBox.left + cardBox.width / 2) - (layoutBox.left + layoutBox.width / 2)
          )
        };
      });
      expect(geometry.width).toBeLessThanOrEqual(geometry.layoutWidth);
      expect(geometry.centerOffset).toBeLessThanOrEqual(1);
      const truncatedLabels = await page.locator(".wallet-settings-view .compact-row-label").evaluateAll(
        (labels) => labels
          .filter((label) => label.scrollWidth > label.clientWidth + 1)
          .map((label) => ({
            text: label.textContent.trim(),
            width: label.clientWidth,
            requiredWidth: label.scrollWidth
          }))
      );
      expect(truncatedLabels, `${viewport.width}px ${section} labels must not truncate`).toEqual([]);
    }
  }
});

test("appearance starts without a redundant horizontal divider", async ({ page }) => {
  await page.goto(`${demoUrl}?view=settings&settings=appearance`);
  const firstGroupStyle = await page.locator(".settings-detail > .setting-group").evaluate((group) => {
    const style = getComputedStyle(group);
    return {
      borderTopWidth: style.borderTopWidth,
      marginTop: style.marginTop,
      paddingTop: style.paddingTop
    };
  });
  expect(firstGroupStyle).toEqual({
    borderTopWidth: "0px",
    marginTop: "0px",
    paddingTop: "0px"
  });
});

test("wallet sections omit redundant introductory headers", async ({ page }) => {
  await page.goto(`${demoUrl}?view=wallet`);

  await page.locator('[data-wallet-section="vouchers"]').click();
  await expect(page.locator("#main-content .page-intro")).toHaveCount(0);
  await page.locator('[data-wallet-section="permissions"]').click();
  await expect(page.locator("#main-content .page-intro")).toHaveCount(0);

  for (const view of ["activity", "swap", "staking", "wallet-backup"]) {
    await page.locator(`#wallet-tabs [data-view="${view}"]`).click();
    await expect(page.locator("#main-content .page-intro")).toHaveCount(0);
  }
});
