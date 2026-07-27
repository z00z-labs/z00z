#!/usr/bin/env python3
"""Capture English Demo views and create non-destructive Help review drafts."""

from __future__ import annotations

import argparse
import base64
from functools import partial
import hashlib
import json
import re
import socket
import struct
import subprocess
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from datetime import date
from html.parser import HTMLParser
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from typing import Any
from urllib.parse import quote, urlparse
from urllib.request import urlopen


DEMO_ROOT = Path(__file__).resolve().parents[2]
HELP_ROOT = DEMO_ROOT / "help"
ASSET_ROOT = HELP_ROOT / "assets" / "en"
STATE_ROOT = HELP_ROOT / "en" / "_generated"
CHROMIUM_PATH = "/usr/bin/chromium"
CAPTURE_WIDTH = 1440
CAPTURE_HEIGHT = 960
DIALOG_ROUTES = {
    "asset.details": "wallet.assets",
    "dapps.detail": "dapps.discover",
    "dapps.permission-review": "dapps.connections",
    "messenger.detail": "messenger.inbox",
    "messenger.request-review": "messenger.inbox",
    "contacts.detail": "contacts.list",
    "contacts.identity-review": "contacts.list",
    "telemetry.watchers.alert-detail": "telemetry.watchers.alerts",
    "telemetry.explorer.detail": "telemetry.explorer.checkpoints",
}
DIALOG_STEPS = {
    "asset.details": ["[data-open-flow='asset-detail']"],
    "dapps.detail": ["[data-dapp-card] [data-dapp-action='open']"],
    "dapps.permission-review": ["[data-dapp-connection] [data-dapp-action='review']"],
    "messenger.detail": ["[data-messenger-message] [data-messenger-action='open']"],
    "messenger.request-review": [
        "[data-messenger-message='message_payment_001'] [data-messenger-action='open']",
        "[data-messenger-action='review']",
    ],
    "contacts.detail": ["[data-contact] [data-contact-action='open']"],
    "contacts.identity-review": [
        "[data-contact='contact_ops'] [data-contact-action='open']",
        "[data-contact-action='identity-review']",
    ],
    "telemetry.watchers.alert-detail": ["[data-watcher-alert]"],
    "telemetry.explorer.detail": ["[data-explorer-record]"],
}
DIALOG_ROOT_IDS = {"asset.details": "flow-dialog"}
TEXT_SPACE = re.compile(r"\s+")
IGNORED_TEXT_TAGS = frozenset({"script", "style", "svg"})
VOID_TAGS = frozenset({"area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"})


class QuietHandler(SimpleHTTPRequestHandler):
    def log_message(self, _format: str, *_arguments: object) -> None:
        return


class ViewExtractor(HTMLParser):
    def __init__(self, root_id: str = "main-content") -> None:
        super().__init__(convert_charrefs=True)
        self.components: set[str] = set()
        self.headings: set[str] = set()
        self.heading_depth = 0
        self.main_depth = 0
        self.suppressed_depth = 0
        self.tag_stack: list[str] = []
        self.terms: set[str] = set()
        self.root_id = root_id

    def handle_starttag(self, tag: str, attributes: list[tuple[str, str | None]]) -> None:
        attrs = {name: value or "" for name, value in attributes}
        if attrs.get("id") == self.root_id:
            self.main_depth = 1
            self.tag_stack = [tag]
            return
        if not self.main_depth:
            return
        is_void = tag in VOID_TAGS
        if not is_void:
            self.main_depth += 1
            self.tag_stack.append(tag)
        if self.suppressed_depth:
            if not is_void:
                self.suppressed_depth += 1
            return
        if "hidden" in attrs or attrs.get("aria-hidden") == "true" or "display: none" in attrs.get("style", "").lower():
            self.suppressed_depth = 1
            return
        if tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self.heading_depth += 1
        if tag in {"button", "input", "select", "textarea", "a"}:
            signature = [tag]
            for name in (
                "type", "role", "name", "aria-label", "placeholder", "data-demo-action", "data-workspace-route",
                "data-help-topic", "data-dialog-close", "data-dapp-action", "data-messenger-action", "data-contact-action",
                "data-watcher-action", "data-explorer-action",
            ):
                if attrs.get(name):
                    signature.append(f"{name}={attrs[name]}")
            self.components.add("|".join(signature))
        for name in ("aria-label", "placeholder", "title", "value"):
            if attrs.get(name):
                self.terms.add(self.clean_text(attrs[name]))

    def handle_endtag(self, tag: str) -> None:
        if not self.main_depth:
            return
        if self.suppressed_depth:
            self.suppressed_depth -= 1
        elif tag in {"h1", "h2", "h3", "h4", "h5", "h6"}:
            self.heading_depth -= 1
        if self.tag_stack:
            self.tag_stack.pop()
        self.main_depth -= 1
        if not self.main_depth:
            self.heading_depth = 0
            self.tag_stack = []

    def handle_data(self, data: str) -> None:
        if self.main_depth and not self.suppressed_depth and not (set(self.tag_stack) & IGNORED_TEXT_TAGS):
            value = self.clean_text(data)
            if value:
                self.terms.add(value)
                if self.heading_depth:
                    self.headings.add(value)

    @staticmethod
    def clean_text(value: str) -> str:
        return TEXT_SPACE.sub(" ", value).strip()

    def snapshot(self) -> dict[str, list[str]]:
        terms = sorted(term for term in self.terms if len(term) > 1 and len(term) < 160)
        return {
            "components": sorted(self.components),
            "sections": sorted(term for term in self.headings if len(term) > 1 and len(term) < 160),
            "terms": terms,
        }


def run_command(arguments: list[str]) -> str:
    result = subprocess.run(
        arguments,
        check=True,
        cwd=DEMO_ROOT,
        capture_output=True,
        text=True,
        timeout=45,
    )
    return result.stdout


def load_contract() -> list[dict[str, Any]]:
    output = run_command(["node", "scripts/help/export-view-contract.mjs"])
    return json.loads(output)["views"]


def asset_name(view: dict[str, Any]) -> str:
    return view["id"].replace(".", "-")


def capture_route(view: dict[str, Any]) -> str:
    if view["routeId"]:
        return view["routeId"]
    if view["scope"] == "dialog":
        return DIALOG_ROUTES[view["id"]]
    return "wallet.assets"


def available_port() -> int:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as listener:
        listener.bind(("127.0.0.1", 0))
        return int(listener.getsockname()[1])


class DevToolsSocket:
    """Tiny dependency-free Chrome DevTools client for deterministic static captures."""

    def __init__(self, web_socket_url: str) -> None:
        parsed = urlparse(web_socket_url)
        self.socket = socket.create_connection((parsed.hostname, parsed.port), timeout=15)
        key = base64.b64encode(hashlib.sha1(str(time.monotonic()).encode()).digest()).decode()
        request = (
            f"GET {parsed.path} HTTP/1.1\r\nHost: {parsed.hostname}:{parsed.port}\r\n"
            f"Upgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\nOrigin: http://localhost\r\n\r\n"
        )
        self.socket.sendall(request.encode("ascii"))
        if b" 101 " not in self._read_http_headers():
            raise RuntimeError("Chrome DevTools WebSocket handshake failed")
        self.next_id = 1

    def _read_http_headers(self) -> bytes:
        response = bytearray()
        while b"\r\n\r\n" not in response:
            fragment = self.socket.recv(4096)
            if not fragment:
                raise RuntimeError("Chrome DevTools closed during handshake")
            response.extend(fragment)
        return bytes(response)

    def send(self, payload: dict[str, Any]) -> None:
        encoded = json.dumps(payload, separators=(",", ":")).encode("utf-8")
        mask = hashlib.sha256(encoded).digest()[:4]
        size = len(encoded)
        if size < 126:
            header = bytes([0x81, 0x80 | size])
        elif size < 65536:
            header = bytes([0x81, 0x80 | 126]) + struct.pack("!H", size)
        else:
            header = bytes([0x81, 0x80 | 127]) + struct.pack("!Q", size)
        masked = bytes(value ^ mask[index % 4] for index, value in enumerate(encoded))
        self.socket.sendall(header + mask + masked)

    def receive(self) -> dict[str, Any]:
        header = self._receive_exact(2)
        size = header[1] & 0x7F
        if size == 126:
            size = struct.unpack("!H", self._receive_exact(2))[0]
        elif size == 127:
            size = struct.unpack("!Q", self._receive_exact(8))[0]
        masked = bool(header[1] & 0x80)
        mask = self._receive_exact(4) if masked else b""
        payload = self._receive_exact(size)
        if masked:
            payload = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
        if header[0] & 0x0F == 0x8:
            raise RuntimeError("Chrome DevTools closed the connection")
        return json.loads(payload.decode("utf-8"))

    def _receive_exact(self, size: int) -> bytes:
        response = bytearray()
        while len(response) < size:
            fragment = self.socket.recv(size - len(response))
            if not fragment:
                raise RuntimeError("Chrome DevTools connection closed")
            response.extend(fragment)
        return bytes(response)

    def command(self, method: str, parameters: dict[str, Any] | None = None) -> dict[str, Any]:
        request_id = self.next_id
        self.next_id += 1
        self.send({"id": request_id, "method": method, "params": parameters or {}})
        while True:
            response = self.receive()
            if response.get("id") != request_id:
                continue
            if "error" in response:
                raise RuntimeError(f"Chrome DevTools {method}: {response['error']['message']}")
            if "exceptionDetails" in response:
                raise RuntimeError(f"Chrome DevTools {method}: browser evaluation raised an exception")
            return response.get("result", {})

    def close(self) -> None:
        self.socket.close()


class CaptureBrowser:
    def __init__(self) -> None:
        self.port = available_port()
        self.process = subprocess.Popen(
            [
                CHROMIUM_PATH,
                "--headless",
                "--no-sandbox",
                "--disable-gpu",
                "--hide-scrollbars",
                "--remote-allow-origins=*",
                f"--remote-debugging-port={self.port}",
                f"--window-size={CAPTURE_WIDTH},{CAPTURE_HEIGHT}",
                "about:blank",
            ],
            cwd=DEMO_ROOT,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        self.browser = DevToolsSocket(self._web_socket_url())
        target = self.browser.command("Target.createTarget", {"url": "about:blank"})["targetId"]
        page = None
        for _ in range(100):
            listing = self._json("/json/list")
            page = next((item for item in listing if item.get("id") == target), None)
            if page:
                break
            time.sleep(0.05)
        if not page:
            raise RuntimeError("Chrome DevTools did not expose the capture page")
        self.page = DevToolsSocket(page["webSocketDebuggerUrl"])
        self.page.command("Page.enable")
        self.page.command("Runtime.enable")

    def _json(self, path: str) -> Any:
        with urlopen(f"http://127.0.0.1:{self.port}{path}", timeout=10) as response:
            return json.loads(response.read().decode("utf-8"))

    def _web_socket_url(self) -> str:
        for _ in range(100):
            try:
                return str(self._json("/json/version")["webSocketDebuggerUrl"])
            except (OSError, KeyError, json.JSONDecodeError):
                time.sleep(0.05)
        raise RuntimeError("Chrome DevTools did not become ready")

    def evaluate(self, expression: str) -> Any:
        result = self.page.command("Runtime.evaluate", {
            "expression": expression,
            "awaitPromise": True,
            "returnByValue": True,
        })["result"]
        if result.get("subtype") == "error" or "exceptionDetails" in result:
            raise RuntimeError(f"Browser evaluation failed: {expression}")
        return result.get("value")

    def navigate(self, url: str) -> None:
        self.page.command("Page.navigate", {"url": url})
        for _ in range(100):
            if self.evaluate("document.readyState") == "complete":
                self.settle()
                return
            time.sleep(0.05)
        raise RuntimeError(f"Capture page did not finish loading: {url}")

    def click(self, selector: str) -> None:
        expression = (
            "(() => { const node = document.querySelector(" + json.dumps(selector) + "); "
            "if (!node) throw new Error('capture selector not found: ' + " + json.dumps(selector) + "); "
            "node.click(); return true; })()"
        )
        self.evaluate(expression)
        self.settle()

    def settle(self) -> None:
        self.evaluate("""(async () => {
          let style = document.querySelector('#z00z-help-capture-style');
          if (!style) {
            style = document.createElement('style');
            style.id = 'z00z-help-capture-style';
            style.textContent = '*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important}';
            document.head.append(style);
          }
          await document.fonts?.ready;
          await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
          return true;
        })()""")

    def capture(self, root_id: str) -> tuple[str, bytes, str]:
        image = self.page.command("Page.captureScreenshot", {"format": "png", "fromSurface": True})["data"]
        presentation = self.evaluate("""(() => {
          const root = document.getElementById(""" + json.dumps(root_id) + """);
          if (!root) throw new Error('capture root is missing');
          return JSON.stringify([root, ...root.querySelectorAll('*')]
            .filter((node) => {
              const style = getComputedStyle(node);
              return style.display !== 'none' && style.visibility !== 'hidden' && style.opacity !== '0';
            })
            .map((node) => {
              const rect = node.getBoundingClientRect();
              const style = getComputedStyle(node);
              return [
                node.tagName,
                node.getAttribute('class') || '',
                node.getAttribute('role') || '',
                node.getAttribute('data-demo-action') || node.getAttribute('data-workspace-route') || '',
                Math.round(rect.x), Math.round(rect.y), Math.round(rect.width), Math.round(rect.height),
                style.display, style.position, style.color, style.backgroundColor,
                style.borderColor, style.fontFamily, style.fontSize, style.fontWeight,
                style.gridTemplateColumns, style.flexDirection,
              ];
            }));
        })()""")
        return str(self.evaluate("document.documentElement.outerHTML")), base64.b64decode(image), str(presentation)

    def close(self) -> None:
        self.page.close()
        self.browser.close()
        self.process.terminate()
        try:
            self.process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait(timeout=10)


def capture_view(view: dict[str, Any], server_url: str, browser: CaptureBrowser) -> tuple[dict[str, Any], dict[str, Any], bytes]:
    route = capture_route(view)
    browser.navigate(f"{server_url}/index.html?route={quote(route)}")
    for selector in DIALOG_STEPS.get(view["id"], []):
        browser.click(selector)
    root_id = DIALOG_ROOT_IDS.get(view["id"], "main-content")
    dom, image, presentation = browser.capture(root_id)
    extractor = ViewExtractor(DIALOG_ROOT_IDS.get(view["id"], "main-content"))
    extractor.feed(dom)
    view_data = extractor.snapshot()
    snapshot = {
        "capture_route": route,
        "components": view_data["components"],
        "scope": view["scope"],
        "presentation_sha256": hashlib.sha256(presentation.encode("utf-8")).hexdigest(),
        "screenshot_sha256": hashlib.sha256(image).hexdigest(),
        "sections": view_data["sections"],
        "screenshot": view["screenshot"],
        "terms": view_data["terms"],
        "topic_id": view["id"],
        "version": 3,
    }
    return view, snapshot, image


def state_path(view: dict[str, Any]) -> Path:
    return STATE_ROOT / f"{asset_name(view)}.json"


def page_path(view: dict[str, Any]) -> Path:
    return HELP_ROOT / "en" / view["pagePath"]


def load_state(path: Path) -> dict[str, Any] | None:
    return json.loads(path.read_text(encoding="utf-8")) if path.is_file() else None


def changes(previous: dict[str, Any] | None, current: dict[str, Any]) -> dict[str, list[str]]:
    if previous is None:
        return {
            "components": current["components"],
            "sections": current["sections"],
            "screenshot": ["Initial App View capture"],
            "terms": current["terms"],
        }
    result: dict[str, list[str]] = {}
    for key in ("components", "sections", "terms"):
        result[key] = sorted(set(current[key]) - set(previous.get(key, [])))
        result[f"{key}_removed"] = sorted(set(previous.get(key, [])) - set(current[key]))
    result["presentation"] = (
        ["App View layout or presentation changed"]
        if (
            previous.get("version") == current.get("version")
            and previous.get("presentation_sha256")
            and previous["presentation_sha256"] != current["presentation_sha256"]
        )
        else []
    )
    return result


def draft_path(page: Path) -> Path:
    base = page.with_name(f"{page.stem}-draft-{date.today():%Y%m%d}.md")
    if not base.exists():
        return base
    sequence = 2
    while True:
        candidate = page.with_name(f"{page.stem}-draft-{date.today():%Y%m%d}-{sequence}.md")
        if not candidate.exists():
            return candidate
        sequence += 1


def draft_source(view: dict[str, Any], snapshot: dict[str, Any], change_set: dict[str, list[str]]) -> str:
    items = []
    for section, values in change_set.items():
        if not values:
            continue
        items.extend(f"- **{section.replace('_', ' ').capitalize()}**: `{value}`" for value in values)
    term_rows = snapshot["terms"] or ["Review all captured terms"]
    rows = "\n".join(
        "| `{}` | Add an accurate user-facing explanation. |".format(term.replace("|", "\\|"))
        for term in term_rows
    )
    return f"""---
id: {view['id']}-draft-{date.today():%Y%m%d}
title: {view['id']} Help update draft
route: {view['routeId'] or 'none'}
scope: draft
---

# {view['id']} Help update draft

## App View {{#current-view}}

![Current application view]({view['screenshot']})

## Required updates

{chr(10).join(items) if items else '- Review the updated interface capture.'}

## Terms and controls

| Term or control | Required explanation |
| --- | --- |
{rows}

## Review note

Merge only reviewed explanations into `{view['pagePath']}`. This draft never replaces the canonical Help page.
"""


def write_result(view: dict[str, Any], snapshot: dict[str, Any], image: bytes, check_only: bool) -> tuple[int, int]:
    page = page_path(view)
    state = state_path(view)
    asset = DEMO_ROOT / view["screenshot"]
    if check_only:
        missing = [str(path) for path in (page, state, asset) if not path.is_file()]
        if missing:
            raise FileNotFoundError(", ".join(missing))
        recorded = load_state(state)
        if recorded is None or recorded.get("topic_id") != view["id"] or recorded.get("screenshot") != view["screenshot"]:
            raise ValueError(f"Invalid capture state for {view['id']}")
        if hashlib.sha256(asset.read_bytes()).hexdigest() != recorded.get("screenshot_sha256"):
            raise ValueError(f"Captured App View asset does not match state for {view['id']}")
        change_set = changes(recorded, snapshot)
        if any(change_set.values()):
            raise ValueError(f"Stale Help capture state for {view['id']}; run sync_views.py to create a review draft")
        return 0, 0
    previous = load_state(state)
    change_set = changes(previous, snapshot)
    is_schema_migration = (
        previous is not None
        and snapshot.get("version") is not None
        and previous.get("version") != snapshot["version"]
    )
    if not any(change_set.values()) and not is_schema_migration:
        return 0, 0
    state.parent.mkdir(parents=True, exist_ok=True)
    asset.parent.mkdir(parents=True, exist_ok=True)
    asset.write_bytes(image)
    state.write_text(f"{json.dumps(snapshot, indent=2, sort_keys=True)}\n", encoding="utf-8")
    if not any(change_set.values()):
        return 0, 0
    draft = draft_path(page)
    draft.write_text(draft_source(view, snapshot, change_set), encoding="utf-8")
    return 1, 0


def check_baseline(view: dict[str, Any]) -> None:
    page = page_path(view)
    state = state_path(view)
    asset = DEMO_ROOT / view["screenshot"]
    missing = [str(path) for path in (page, state, asset) if not path.is_file()]
    if missing:
        raise FileNotFoundError(", ".join(missing))
    recorded = load_state(state)
    if recorded is None or recorded.get("topic_id") != view["id"] or recorded.get("screenshot") != view["screenshot"]:
        raise ValueError(f"Invalid capture state for {view['id']}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--verify-current", action="store_true", help="capture views and fail on un-synchronized UI drift")
    parser.add_argument("--topic", help="capture or verify one Help topic ID")
    parser.add_argument("--workers", type=int, default=4)
    arguments = parser.parse_args()
    views = load_contract()
    if arguments.topic:
        views = [view for view in views if view["id"] == arguments.topic]
        if not views:
            raise SystemExit(f"Unknown Help topic: {arguments.topic}")
    if arguments.check and not arguments.verify_current:
        for view in views:
            check_baseline(view)
        print(f"English Help capture baseline ready: {len(views)} views")
        return
    if not Path(CHROMIUM_PATH).is_file():
        raise SystemExit(f"Chromium is required at {CHROMIUM_PATH}")
    server = ThreadingHTTPServer(("127.0.0.1", 0), partial(QuietHandler, directory=str(DEMO_ROOT)))
    server_thread = threading.Thread(target=server.serve_forever, daemon=True)
    server_thread.start()
    server_url = f"http://127.0.0.1:{server.server_port}"
    try:
        worker_count = min(max(1, arguments.workers), len(views))
        batches = [views[index::worker_count] for index in range(worker_count)]

        def capture_batch(batch: list[dict[str, Any]]) -> list[tuple[dict[str, Any], dict[str, Any], bytes]]:
            browser = CaptureBrowser()
            try:
                return [capture_view(view, server_url, browser) for view in batch]
            finally:
                browser.close()

        with ThreadPoolExecutor(max_workers=worker_count) as executor:
            captures = [capture for batch in executor.map(capture_batch, batches) for capture in batch]
        drafts = 0
        preserved = 0
        for view, snapshot, image in captures:
            created, retained = write_result(view, snapshot, image, arguments.verify_current)
            drafts += created
            preserved += retained
        if arguments.verify_current:
            print(f"English Help capture state current: {len(views)} views")
        else:
            print(f"English Help synchronized: views={len(views)}, drafts={drafts}, existing_drafts={preserved}")
    finally:
        server.shutdown()
        server.server_close()


if __name__ == "__main__":
    main()
