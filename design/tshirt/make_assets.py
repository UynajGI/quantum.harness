#!/usr/bin/env python3
"""Build assets/*.svg by EXTRACTION only:
- logos: from downloaded files in refs/ (claude favicon, openai.svg, deepseek.svg, kimi favicon)
- poster elements (ape, human+staff, tab key, spark): cropped + vectorized from
  original_design.png (the school's poster bitmap) via potrace.
Nothing is hand-drawn here."""
import re, subprocess, sys, pathlib

HERE = pathlib.Path(__file__).parent
REFS = HERE / "refs"
OUT = HERE / "assets"
OUT.mkdir(exist_ok=True)
SVG = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}">\n{}\n</svg>\n'

def sh(*args):
    subprocess.run(args, check=True, cwd=HERE)

def paths_from(svg_text):
    return re.findall(r'<path[^>]*\sd="([^"]+)"', svg_text, re.S)

def tile_from_paths(name, paths, vb, fill):
    """tile rect + logo paths scaled/centered into 120x120 tile, 12pt margin."""
    vx, vy, vw, vh = vb
    s = 96.0 / max(vw, vh)
    tx = 60 - s * (vx + vw / 2)
    ty = 60 - s * (vy + vh / 2)
    ps = "\n".join(f'<path d="{d}"/>' for d in paths)
    body = ('<rect width="120" height="120" rx="26" fill="#151515" stroke="#2e2e2e" stroke-width="1.5"/>\n'
            f'<g transform="translate({tx:.2f},{ty:.2f}) scale({s:.4f})" fill="{fill}">{ps}</g>')
    (OUT / f"tile_{name}.svg").write_text(SVG.format("0 0 120 120", body))

# ---------- logos from downloads ----------
claude = (REFS / "claude_ai_favicon_svg.svg").read_text()
tile_from_paths("claude", paths_from(claude), (0, 0, 248, 248), "#D97757")
(OUT / "claude_star.svg").write_text(SVG.format(
    "0 0 248 248", f'<g fill="#D97757">' + "\n".join(f'<path d="{p}"/>' for p in paths_from(claude)) + "</g>"))

openai = (REFS / "openai.svg").read_text()
tile_from_paths("openai", paths_from(openai), (0, 0, 79.9, 81), "#FFFFFF")

deepseek = (REFS / "deepseek.svg").read_text()
tile_from_paths("deepseek", paths_from(deepseek), (0, 0, 24, 24), "#4D6BFE")

# kimi K: vectorize the official favicon (dark pass for K glyph, blue pass for dots)
sh("magick", str(REFS / "kimi_favicon.ico") + "[0]", "-flatten", str(REFS / "_kimi.png"))
# dark glyph: black K (and accents) on white bg - ready for potrace
sh("magick", str(REFS / "_kimi.png"), "-colorspace", "gray", "-threshold", "45%",
   str(REFS / "_kimi_dark.pbm"))
# blue dots: B minus R isolates blue; then black dots on white for potrace
sh("magick", "(", str(REFS / "_kimi.png"), "-channel", "B", "-separate", ")",
   "(", str(REFS / "_kimi.png"), "-channel", "R", "-separate", ")",
   "-compose", "minus-src", "-composite", "-threshold", "30%", "-negate",
   str(REFS / "_kimi_blue.pbm"))
for stem in ("dark", "blue"):
    sh("potrace", "-s", "--turdsize", "4", str(REFS / f"_kimi_{stem}.pbm"),
       "-o", str(REFS / f"_kimi_{stem}.svg"))

def traced_inner(stem):
    """potrace inner <g transform=...><path/></g>, fill stripped, plus natural pt size."""
    t = (REFS / f"_kimi_{stem}.svg").read_text()
    w = float(re.search(r'width="([\d.]+)pt', t).group(1))
    h = float(re.search(r'height="([\d.]+)pt', t).group(1))
    g = re.search(r"(<g transform.*?</g>)", t, re.S).group(1)
    g = re.sub(r'fill="[^"]*"', "", g)
    return g, w, h

gd, wpt, hpt = traced_inner("dark")
gb, _, _ = traced_inner("blue")
s = 84.0 / max(wpt, hpt)
body = ('<rect width="120" height="120" rx="26" fill="#151515" stroke="#2e2e2e" stroke-width="1.5"/>\n'
        f'<g transform="translate({60 - s * wpt / 2:.2f},{60 - s * hpt / 2:.2f}) scale({s:.4f})">'
        f'<g fill="#FFFFFF">{gd}</g><g fill="#2E7DF6">{gb}</g></g>')
(OUT / "tile_kimi.svg").write_text(SVG.format("0 0 120 120", body))

# ---------- poster elements from original_design.png ----------
src = HERE / "skills-poster.jpg"
if not src.exists():
    print("NOTE: skills-poster.jpg not found - poster elements skipped "
          "(ape/human_staff/tabkey/spark)")
    sys.exit(0)

# crop boxes measured on the 2245x1587 poster bitmap: name -> (x, y, w, h)
boxes = {
    "ape":         (610, 695, 170, 165),
    "human_staff": (605, 970, 180, 210),
    "tabkey":      (812, 693, 205, 155),
    "spark":       (1565, 1050, 60, 60),
}
for name, (x, y, w, h) in boxes.items():
    # potrace traces BLACK on WHITE -> negate so the figure itself is traced
    sh("magick", str(src), "-crop", f"{w}x{h}+{x}+{y}", "+repage",
       "-colorspace", "gray", "-blur", "0x0.8", "-threshold", "45%", "-negate",
       str(REFS / f"_{name}.pbm"))
    sh("potrace", "-s", "--turdsize", "8", str(REFS / f"_{name}.pbm"), "-o", str(REFS / f"_{name}.svg"))
    t = (REFS / f"_{name}.svg").read_text()
    wpt = re.search(r'width="([\d.]+)pt', t).group(1)
    hpt = re.search(r'height="([\d.]+)pt', t).group(1)
    g = re.search(r"(<g transform.*?</g>)", t, re.S).group(1)
    g = re.sub(r'fill="[^"]*"', "", g)
    (OUT / f"{name}.svg").write_text(SVG.format(f"0 0 {wpt} {hpt}", f'<g fill="#FFFFFF">{g}</g>'))

print("assets written:", sorted(p.name for p in OUT.glob("*.svg")))
