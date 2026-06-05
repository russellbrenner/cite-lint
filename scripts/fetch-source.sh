#!/usr/bin/env bash
# Fetch the source PDF for a cite-lint edition into the git-ignored cache and
# verify it against the pinned SHA256 in data/editions/<id>/meta.toml.
#
# The PDF is third-party copyright and is NEVER committed. This script only
# downloads it locally for the ingestion pipeline.
#
# The publisher's host (law.unimelb.edu.au) sits behind a WAF that 403s requests
# lacking a full browser header set. Plain `curl`/`wget` with a bare UA are
# blocked; the complete header set below passes. A Playwright (headless Chrome)
# fallback is provided for environments where even that is blocked.
#
# Usage: scripts/fetch-source.sh [edition-id]   (default: aglc4)
set -euo pipefail

EDITION="${1:-aglc4}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
META="$ROOT/data/editions/$EDITION/meta.toml"
CACHE="$ROOT/.cache/sources"
DEST="$CACHE/$EDITION.pdf"

[ -f "$META" ] || { echo "error: no meta.toml for edition '$EDITION' at $META" >&2; exit 1; }

# Minimal TOML field reads (url / sha256 live on their own lines under [source]).
field() { sed -n "s/^[[:space:]]*$1[[:space:]]*=[[:space:]]*\"\\([^\"]*\\)\".*/\\1/p" "$META" | head -1; }
URL="$(field url)"
EXPECT="$(field sha256)"
[ -n "$URL" ] || { echo "error: no source.url in $META" >&2; exit 1; }

mkdir -p "$CACHE"

sha_of() { sha256sum "$1" | cut -d' ' -f1; }

# Skip if already cached and matching.
if [ -f "$DEST" ] && [ -n "$EXPECT" ] && [ "$(sha_of "$DEST")" = "$EXPECT" ]; then
  echo "ok: $DEST already present and matches pinned sha256"
  exit 0
fi

UA="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36"

fetch_curl() {
  curl -fsSL -o "$DEST" \
    -A "$UA" \
    -H "Accept: text/html,application/xhtml+xml,application/xml;q=0.9,application/pdf,image/avif,image/webp,*/*;q=0.8" \
    -H "Accept-Language: en-AU,en;q=0.9" \
    -H "Accept-Encoding: gzip, deflate, br" \
    -H "Upgrade-Insecure-Requests: 1" \
    -H "sec-ch-ua: \"Chromium\";v=\"124\", \"Google Chrome\";v=\"124\", \"Not-A.Brand\";v=\"99\"" \
    -H "sec-ch-ua-mobile: ?0" \
    -H "sec-ch-ua-platform: \"Windows\"" \
    -H "Sec-Fetch-Dest: document" -H "Sec-Fetch-Mode: navigate" \
    -H "Sec-Fetch-Site: none" -H "Sec-Fetch-User: ?1" \
    -H "Referer: https://law.unimelb.edu.au/" \
    "$URL"
}

# Fallback: drive a real headless Chrome via Playwright. Uses the browser's own
# network stack (genuine TLS fingerprint), defeating fingerprint-based WAFs.
fetch_playwright() {
  command -v npx >/dev/null 2>&1 || return 1
  URL="$URL" DEST="$DEST" npx --yes playwright@latest install chromium >/dev/null 2>&1 || true
  URL="$URL" DEST="$DEST" node -e '
    const { chromium } = require("playwright");
    (async () => {
      const b = await chromium.launch();
      const ctx = await b.newContext();
      const res = await ctx.request.get(process.env.URL);
      if (!res.ok()) { console.error("playwright http " + res.status()); process.exit(1); }
      require("fs").writeFileSync(process.env.DEST, await res.body());
      await b.close();
    })().catch(e => { console.error(e); process.exit(1); });
  '
}

echo "fetching $EDITION source via curl (browser headers)..."
if ! fetch_curl; then
  echo "curl blocked; trying Playwright (headless Chrome) fallback..." >&2
  fetch_playwright || { echo "error: both curl and Playwright failed" >&2; exit 1; }
fi

GOT="$(sha_of "$DEST")"
if [ -z "$EXPECT" ]; then
  echo "warning: no pinned sha256 in $META; trust-on-first-use." >&2
  echo "         add this to [source]:  sha256 = \"$GOT\"" >&2
elif [ "$GOT" != "$EXPECT" ]; then
  echo "error: sha256 mismatch for $DEST" >&2
  echo "       expected $EXPECT" >&2
  echo "       got      $GOT" >&2
  exit 1
fi

echo "ok: $DEST ($(wc -c < "$DEST") bytes, sha256 $GOT)"
