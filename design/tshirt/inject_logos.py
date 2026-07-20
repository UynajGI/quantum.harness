#!/usr/bin/env python3
"""Inject real logo path data (refs/*.svg) into artwork.svg between REAL markers."""
import re, sys, pathlib

HERE = pathlib.Path(__file__).parent
ART = HERE / "artwork.svg"

def paths_from(svg_file):
    txt = svg_file.read_text()
    vb = re.search(r'viewBox="([^"]+)"', txt)
    vx, vy, vw, vh = [float(v) for v in vb.group(1).split()]
    ds = re.findall(r'<path[^>]*\sd="([^"]+)"', txt, re.S)
    if not ds:
        sys.exit(f"no paths in {svg_file}")
    return (vx, vy, vw, vh), ds

def group(name, ref, fill, cx, cy, target):
    (vx, vy, vw, vh), ds = paths_from(HERE / "refs" / ref)
    s = target / max(vw, vh)
    tx = cx - s * (vx + vw / 2)
    ty = cy - s * (vy + vh / 2)
    ps = "\n      ".join(f'<path d="{d}"/>' for d in ds)
    return (f'    <g transform="translate({tx:.2f},{ty:.2f}) scale({s:.4f})" fill="{fill}">\n'
            f'      {ps}\n    </g>')

art = ART.read_text()
for name, ref, fill, cx, cy, target in [
    ("CLAUDE",   "claude_ai_favicon_svg.svg", "#D97757", 770, 412, 68),
    ("OPENAI",   "openai.svg",                "#FFFFFF", 914, 412, 66),
    ("DEEPSEEK", "deepseek.svg",              "#4D6BFE", 770, 556, 68),
]:
    block = group(name, ref, fill, cx, cy, target)
    pat = re.compile(rf"(<!-- REAL:{name} begin -->).*?(<!-- REAL:{name} end -->)", re.S)
    art, n = pat.subn(lambda m: m.group(1) + "\n" + block + "\n    " + m.group(2), art)
    if n != 1:
        sys.exit(f"marker REAL:{name} not found exactly once")

ART.write_text(art)
print("injected: claude, openai, deepseek")
