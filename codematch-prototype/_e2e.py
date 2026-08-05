#!/usr/bin/env python3
"""End-to-end browser walkthrough of the W2 prototype.

Drives the actual page in a real browser, screenshotting at every
state transition (matching → lobby → room → chat echo). Uses the
test-only sweep endpoint so transitions are deterministic instead
of relying on the 2s background tick.

Run after the server is up:
    PROTOTYPE_DIR=.../codematch-prototype DEV_MODE=1 HOST=127.0.0.1 PORT=18081 \
    .../target/debug/codematch-server
"""
import asyncio, json, urllib.request, sys

from playwright.async_api import async_playwright

PROTOTYPE = "http://127.0.0.1:18081/app/index.html"
API = "http://127.0.0.1:18081"


def queue_user(handle: str) -> str:
    """Dev-login and return the session token."""
    req = urllib.request.Request(f"{API}/auth/dev-login?as={handle}")
    with urllib.request.urlopen(req) as r:
        cookie = r.headers.get("set-cookie", "")
        return cookie.split("cm_session=")[1].split(";")[0]


def api_call(token: str, method: str, path: str, body=None) -> dict:
    data = json.dumps(body).encode() if body is not None else None
    req = urllib.request.Request(
        f"{API}{path}",
        data=data,
        method=method,
        headers={
            "Cookie": f"cm_session={token}",
            "Content-Type": "application/json",
        },
    )
    with urllib.request.urlopen(req) as r:
        return json.loads(r.read() or b"{}")


async def wait_for_screen(page, target: str, timeout_ms: int = 8000):
    """Block until window.__app.state.screen === target, or timeout."""
    deadline = timeout_ms
    while deadline > 0:
        cur = await page.evaluate("window.__app?.state?.screen")
        if cur == target:
            return True
        await page.wait_for_timeout(150)
        deadline -= 150
    return False


async def wait_for_lobby_id(page, timeout_ms: int = 6000):
    """Block until the matching poll surfaces a pending_lobby_id."""
    deadline = timeout_ms
    while deadline > 0:
        lid = await page.evaluate(
            "window.__app?.state?.matchStatus?.pending_lobby_id"
        )
        if lid:
            return lid
        await page.wait_for_timeout(150)
        deadline -= 150
    return None


async def main():
    async with async_playwright() as p:
        browser = await p.chromium.launch()
        ctx = await browser.new_context(viewport={"width": 480, "height": 920})
        page = await ctx.new_page()
        page.on("console", lambda m: print(f"   [c.{m.type}] {m.text[:160]}"))
        page.on("requestfailed", lambda r: print(f"   [reqfail] {r.url[:80]}"))

        # Block Google Fonts so the page doesn't hang in this sandbox.
        await ctx.route("**/fonts.googleapis.com/**", lambda r: r.abort())
        await ctx.route("**/fonts.gstatic.com/**", lambda r: r.abort())

        print("→ boot prototype + dev-login as @you")
        await page.goto(PROTOTYPE, wait_until="domcontentloaded", timeout=15000)
        await page.wait_for_timeout(1200)
        await page.get_by_role("button", name="Sign in (dev) as @you").click()
        # dev-login → me → enqueue → matching screen
        ok = await wait_for_screen(page, "matching", timeout_ms=8000)
        if not ok:
            body = await page.locator("body").inner_text()
            print(f"   stuck on {await page.evaluate('window.__app.state.screen')!r}: {body[:200]!r}")
            await page.screenshot(path="/tmp/e2e-fail-boot.png", full_page=True)
            await browser.close()
            sys.exit(1)

        # --- Screenshot 01: matching screen with "you" alone in queue ---
        # Wait a tick so the first matchStatus poll lands and the
        # queue size renders as a real number.
        await page.wait_for_timeout(2200)
        await page.screenshot(path="/tmp/e2e-01-matching.png", full_page=True)
        print("   e2e-01-matching.png — matching, queue=1")

        print("→ push maya/raj/lin/sam into the queue")
        for h in ["maya", "raj", "lin", "sam"]:
            t = queue_user(h)
            api_call(t, "POST", "/api/match/queue", {"languages": ["rust"]})
        # give the page's poll one tick to redraw the queue size
        await page.wait_for_timeout(2200)
        qs = await page.evaluate("window.__app.state.matchStatus.queue_size")
        print(f"   queue size after push: {qs}")

        print("→ drive matching sweep → lobby forms")
        # Test-only endpoint: run the matching engine synchronously
        # instead of waiting for the 2s background tick.
        api_call(queue_user("you"), "POST", "/api/_test/sweep", {})

        ok = await wait_for_screen(page, "lobby", timeout_ms=8000)
        if not ok:
            cur = await page.evaluate("window.__app.state.screen")
            lid = await page.evaluate("window.__app.state.matchStatus?.pending_lobby_id")
            print(f"   did not reach lobby (screen={cur!r} lid={lid!r})")
            await page.screenshot(path="/tmp/e2e-fail-lobby.png", full_page=True)
            await browser.close()
            sys.exit(1)

        # Give the lobby's first poll one tick to load the seat list.
        await page.wait_for_timeout(1800)
        await page.screenshot(path="/tmp/e2e-02-lobby.png", full_page=True)
        print("   e2e-02-lobby.png — lobby with 4 seats")

        print("→ all 4 vote accept")
        # Read the actual seats from the page so we only vote for
        # whoever is in the lobby (the engine may pick a different
        # 4 from whoever was pushed).
        seats = await page.evaluate(
            "window.__app.state.currentLobby.seats.map(s => s.username)"
        )
        print(f"   lobby seats: {seats}")
        lid = await page.evaluate("window.__app.state.currentLobby.id")
        for h in seats:
            t = queue_user(h)
            api_call(t, "POST", f"/api/lobbies/{lid}/vote", {"vote": "accept"})

        ok = await wait_for_screen(page, "room", timeout_ms=8000)
        if not ok:
            cur = await page.evaluate("window.__app.state.screen")
            lstatus = await page.evaluate("window.__app.state.currentLobby?.status")
            print(f"   did not reach room (screen={cur!r} lobby={lstatus!r})")
            await page.screenshot(path="/tmp/e2e-fail-room.png", full_page=True)
            await browser.close()
            sys.exit(1)

        # Give the WebSocket a moment to attach and replay backlog.
        await page.wait_for_timeout(1500)
        await page.screenshot(path="/tmp/e2e-03-room.png", full_page=True)
        print("   e2e-03-room.png — room, ws attached")

        print("→ send a chat message")
        await page.locator("#composer-input").fill("E2E — hi squad 👋")
        await page.locator("#composer-input").press("Enter")
        # The server appends + fans out → we should see our own message
        # echoed back via the WebSocket.
        await page.wait_for_timeout(1500)
        msgs = await page.evaluate("window.__app.state.roomMessages.length")
        last = await page.evaluate("window.__app.state.roomMessages.at(-1)?.body")
        print(f"   roomMessages={msgs} last={last!r}")

        await page.screenshot(path="/tmp/e2e-04-room-msg.png", full_page=True)
        print("   e2e-04-room-msg.png — room with chat echo")

        if msgs < 1 or not (last and "E2E" in last):
            print("   WARN: chat echo did not arrive")

        await browser.close()
        print("done ✓")


asyncio.run(main())
