#!/usr/bin/env python3
"""Build a fully inlined Live Steel panel from the ratified standalone.

This does not alter mf_proto_live_steel.html or the release evidence bundle.
It reads the panel loader, extracts the injected workshop script, and appends it
into a copy of the standalone HTML.
"""
from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SOURCE = ROOT / "mf_proto_live_steel.html"
DEFAULT_LOADER = ROOT / "live_steel_panel.html"
DEFAULT_OUT = ROOT / "live_steel_panel_built.html"


def read_panel_script(loader_html: str) -> str:
    match = re.search(
        r'<script\s+id=["\']live-steel-panel-injection["\']\s+type=["\']text/plain["\']>(.*?)</script>',
        loader_html,
        flags=re.IGNORECASE | re.DOTALL,
    )
    if not match:
        raise SystemExit("could not find live-steel-panel-injection in loader")
    return match.group(1).strip()


def inject(source_html: str, panel_js: str) -> str:
    if "__WASM_B64__" in source_html:
        raise SystemExit("source still has __WASM_B64__; build mf_proto_live_steel.html first")
    if "id=\"wasmb64\"" not in source_html and "id='wasmb64'" not in source_html:
        raise SystemExit("source does not look like mf_proto_live_steel.html")
    script = "\n<script>\n" + panel_js.replace("</script", "<\\/script") + "\n</script>\n"
    if re.search(r"</body>", source_html, flags=re.IGNORECASE):
        return re.sub(r"</body>", script + "</body>", source_html, count=1, flags=re.IGNORECASE)
    return source_html + script


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source", default=str(DEFAULT_SOURCE), help="Built canonical standalone")
    parser.add_argument("--loader", default=str(DEFAULT_LOADER), help="Panel loader HTML")
    parser.add_argument("--out", default=str(DEFAULT_OUT), help="Generated panel standalone")
    args = parser.parse_args()

    source = Path(args.source)
    loader = Path(args.loader)
    out = Path(args.out)
    panel_js = read_panel_script(loader.read_text(encoding="utf-8"))
    html = inject(source.read_text(encoding="utf-8"), panel_js)
    out.write_text(html, encoding="utf-8", newline="\n")
    print(f"wrote {out} ({len(html.encode('utf-8'))} bytes)")


if __name__ == "__main__":
    main()
