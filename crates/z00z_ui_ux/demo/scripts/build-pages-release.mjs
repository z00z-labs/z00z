import { readFile, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { compileHelp } from "./compile-help.mjs";
import { synchronizeHelp } from "./sync-help.mjs";

const SHA_PATTERN = /^[0-9a-f]{40}$/i;
const CSS_FILES = Object.freeze([
  "styles.css",
  "styles/colors.css",
  "styles/foundation.css",
  "styles/components.css"
]);

function splitUrl(value) {
  const hashIndex = value.indexOf("#");
  const hash = hashIndex >= 0 ? value.slice(hashIndex) : "";
  const withoutHash = hashIndex >= 0 ? value.slice(0, hashIndex) : value;
  const queryIndex = withoutHash.indexOf("?");
  return {
    path: queryIndex >= 0 ? withoutHash.slice(0, queryIndex) : withoutHash,
    query: queryIndex >= 0 ? withoutHash.slice(queryIndex + 1) : "",
    hash
  };
}

export function versionLocalUrl(value, sha) {
  if (
    !value ||
    value.startsWith("#") ||
    value.startsWith("data:") ||
    value.startsWith("blob:") ||
    /^(?:https?:)?\/\//i.test(value)
  ) {
    return value;
  }

  const parts = splitUrl(value);
  const params = new URLSearchParams(parts.query);
  params.set("v", sha);
  return `${parts.path}?${params.toString()}${parts.hash}`;
}

export function versionHtml(source, sha) {
  return source.replace(
    /(<(?:script|link|img|source|video|audio)\b[^>]*?\b(?:src|href)=)(["'])([^"']+)\2/gi,
    (_match, prefix, quote, value) => `${prefix}${quote}${versionLocalUrl(value, sha)}${quote}`
  );
}

export function versionCss(source, sha) {
  return source.replace(
    /url\((["']?)([^"'()]+)\1\)/gi,
    (_match, quote, value) => `url(${quote}${versionLocalUrl(value.trim(), sha)}${quote})`
  );
}

export function versionScriptAssets(source, sha) {
  return source.replace(
    /(["'])(assets\/[^"'`\r\n]+)\1/g,
    (_match, quote, value) => `${quote}${versionLocalUrl(value, sha)}${quote}`
  );
}

export function versionManifest(manifest, sha) {
  return {
    ...manifest,
    start_url: versionLocalUrl("./", sha),
    icons: manifest.icons.map((icon) => ({
      ...icon,
      src: versionLocalUrl(icon.src, sha)
    }))
  };
}

export function injectRefreshScript(source, sha) {
  const refreshScript = `
    <script data-pages-release="${sha}">
      (() => {
        const buildSha = ${JSON.stringify(sha)};
        let isChecking = false;
        const checkRelease = async () => {
          if (isChecking || !navigator.onLine) return;
          isChecking = true;
          try {
            const endpoint = new URL("deployment.json", window.location.href);
            endpoint.searchParams.set("check", Date.now().toString());
            const response = await fetch(endpoint, { cache: "no-store", credentials: "same-origin" });
            if (!response.ok) return;
            const deployment = await response.json();
            if (!deployment.sha || deployment.sha === buildSha) return;
            const nextUrl = new URL(window.location.href);
            nextUrl.searchParams.set("v", deployment.sha);
            nextUrl.searchParams.set("refresh", Date.now().toString());
            window.location.replace(nextUrl);
          } catch {
            // The Pages preview remains usable when the device is offline.
          } finally {
            isChecking = false;
          }
        };
        window.addEventListener("focus", checkRelease);
        window.addEventListener("online", checkRelease);
        document.addEventListener("visibilitychange", () => {
          if (!document.hidden) checkRelease();
        });
        window.setInterval(checkRelease, 60_000);
        checkRelease();
      })();
    </script>`;
  const cacheMeta = `
    <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate">
    <meta http-equiv="Pragma" content="no-cache">
    <meta http-equiv="Expires" content="0">`;
  return source
    .replace("<head>", `<head>${cacheMeta}`)
    .replace("</head>", `${refreshScript}\n  </head>`);
}

function scriptPaths(source) {
  return [...source.matchAll(/<script\s+src="([^"]+)"/g)]
    .map((match) => splitUrl(match[1]).path);
}

export async function buildPagesRelease(root, sha, ref = "main") {
  if (!SHA_PATTERN.test(sha)) {
    throw new Error("Pages release SHA must be a 40-character hexadecimal commit id.");
  }

  const read = (path) => readFile(resolve(root, path), "utf8");
  const write = (path, value) => writeFile(resolve(root, path), value, "utf8");
  await synchronizeHelp(root);
  await write("scripts/generated/help-catalog.js", await compileHelp(root));
  const rawIndex = await read("index.html");

  for (const path of scriptPaths(rawIndex)) {
    await write(path, versionScriptAssets(await read(path), sha));
  }

  for (const path of CSS_FILES) {
    await write(path, versionCss(await read(path), sha));
  }

  const manifest = versionManifest(JSON.parse(await read("manifest.webmanifest")), sha);
  await write("manifest.webmanifest", `${JSON.stringify(manifest, null, 2)}\n`);

  const index = injectRefreshScript(versionHtml(rawIndex, sha), sha);
  await write("index.html", index);
  await write("deployment.json", `${JSON.stringify({
    sha,
    ref,
    built_at: new Date().toISOString()
  })}\n`);
}

async function main() {
  const [root, sha, ref = "main"] = process.argv.slice(2);
  if (!root || !sha) {
    throw new Error("Usage: node build-pages-release.mjs <site-root> <commit-sha> [ref]");
  }
  await buildPagesRelease(resolve(root), sha, ref);
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  main().catch((error) => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
