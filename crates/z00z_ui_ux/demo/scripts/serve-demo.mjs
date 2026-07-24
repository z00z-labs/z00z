import { createServer } from "node:http";
import { readFile, stat, writeFile } from "node:fs/promises";
import { dirname, extname, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { watch } from "node:fs";
import { compileHelp } from "./compile-help.mjs";
import { synchronizeHelp } from "./sync-help.mjs";

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const demoRoot = resolve(scriptDirectory, "..");
const port = Number(process.argv[2] || process.env.PORT || 4173);
const reloadClients = new Set();
const mimeTypes = Object.freeze({
  ".css": "text/css; charset=utf-8",
  ".html": "text/html; charset=utf-8",
  ".js": "text/javascript; charset=utf-8",
  ".json": "application/json; charset=utf-8",
  ".md": "text/markdown; charset=utf-8",
  ".png": "image/png",
  ".svg": "image/svg+xml",
  ".woff2": "font/woff2"
});
const liveReload = `<script>
  (() => {
    const source = new EventSource("/__z00z-help-reload");
    source.addEventListener("help-updated", () => window.location.reload());
  })();
</script>`;

async function rebuildHelp() {
  const changed = await synchronizeHelp(demoRoot);
  await writeFile(
    resolve(demoRoot, "scripts/generated/help-catalog.js"),
    await compileHelp(demoRoot),
    "utf8"
  );
  reloadClients.forEach((response) => response.write("event: help-updated\ndata: ready\n\n"));
  console.log(changed.length
    ? `Help rebuilt after English source update: ${changed.join(", ")}`
    : "Help catalogue rebuilt.");
}

function safeFilePath(pathname) {
  const relativePath = pathname === "/" ? "index.html" : pathname.replace(/^\/+/, "");
  const filePath = resolve(demoRoot, relativePath);
  if (filePath !== demoRoot && !filePath.startsWith(`${demoRoot}${sep}`)) {
    throw new Error("Path escapes the demo root");
  }
  return filePath;
}

const server = createServer(async (request, response) => {
  const pathname = decodeURIComponent(new URL(request.url, "http://127.0.0.1").pathname);
  if (pathname === "/__z00z-help-reload") {
    response.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive"
    });
    response.write("retry: 1000\n\n");
    reloadClients.add(response);
    request.on("close", () => reloadClients.delete(response));
    return;
  }

  try {
    const filePath = safeFilePath(pathname);
    if (!(await stat(filePath)).isFile()) throw new Error("Not a file");
    let content = await readFile(filePath);
    if (filePath.endsWith("index.html")) {
      content = Buffer.from(content.toString("utf8").replace("</body>", `${liveReload}</body>`));
    }
    response.writeHead(200, {
      "Content-Type": mimeTypes[extname(filePath)] || "application/octet-stream",
      "Cache-Control": "no-store"
    });
    response.end(content);
  } catch {
    response.writeHead(404, { "Content-Type": "text/plain; charset=utf-8" });
    response.end("Not found");
  }
});

await rebuildHelp();
server.listen(port, "127.0.0.1", () => {
  console.log(`Z00Z wallet demo: http://127.0.0.1:${port}`);
  console.log("Watching help/en/*.md; translated Help rebuilds and reloads the page automatically.");
});

let debounce;
const watcher = watch(resolve(demoRoot, "help"), { recursive: true }, (_event, filename = "") => {
  if (!filename.endsWith(".md") || !filename.startsWith(`en${sep}`)) return;
  clearTimeout(debounce);
  debounce = setTimeout(() => {
    rebuildHelp().catch((error) => console.error(`Help rebuild failed: ${error.message}`));
  }, 120);
});

function close() {
  watcher.close();
  reloadClients.forEach((response) => response.end());
  server.close(() => process.exit(0));
}

process.on("SIGINT", close);
process.on("SIGTERM", close);
