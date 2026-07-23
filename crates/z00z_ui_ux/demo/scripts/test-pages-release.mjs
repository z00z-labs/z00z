import assert from "node:assert/strict";
import {
  injectRefreshScript,
  versionCss,
  versionHtml,
  versionLocalUrl,
  versionManifest,
  versionScriptAssets
} from "./build-pages-release.mjs";

const sha = "a".repeat(40);

assert.equal(versionLocalUrl("app.js", sha), `app.js?v=${sha}`);
assert.equal(versionLocalUrl("manifest.webmanifest?v=2", sha), `manifest.webmanifest?v=${sha}`);
assert.equal(versionLocalUrl("#i-wallet", sha), "#i-wallet");
assert.equal(versionLocalUrl("https://example.com/app.js", sha), "https://example.com/app.js");

const html = versionHtml(
  '<link rel="stylesheet" href="styles.css"><script src="app.js"></script><use href="#i-wallet"></use>',
  sha
);
assert.match(html, new RegExp(`href="styles\\.css\\?v=${sha}"`));
assert.match(html, new RegExp(`src="app\\.js\\?v=${sha}"`));
assert.match(html, /href="#i-wallet"/);

const css = versionCss(
  '@import url("styles/colors.css"); .brand { background: url(assets/logo/brand.svg); }',
  sha
);
assert.match(css, new RegExp(`styles/colors\\.css\\?v=${sha}`));
assert.match(css, new RegExp(`assets/logo/brand\\.svg\\?v=${sha}`));

const script = versionScriptAssets('const icon = "assets/logo/brand.svg";', sha);
assert.equal(script, `const icon = "assets/logo/brand.svg?v=${sha}";`);

const manifest = versionManifest({
  id: "./",
  start_url: "./",
  icons: [{ src: "assets/logo/icon.png", sizes: "192x192" }]
}, sha);
assert.equal(manifest.id, "./");
assert.equal(manifest.start_url, `./?v=${sha}`);
assert.equal(manifest.icons[0].src, `assets/logo/icon.png?v=${sha}`);

const releaseHtml = injectRefreshScript("<html><head></head><body></body></html>", sha);
assert.match(releaseHtml, new RegExp(`data-pages-release="${sha}"`));
assert.match(releaseHtml, /deployment\.json/);
assert.match(releaseHtml, /cache: "no-store"/);
assert.match(releaseHtml, /visibilitychange/);
assert.match(releaseHtml, /no-cache, no-store, must-revalidate/);

console.log("Pages release cache-busting checks passed.");
