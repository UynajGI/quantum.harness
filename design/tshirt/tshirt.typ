// Harnessing Quantum 2026 — evolution T-shirt (Typst master)
// Edit the words below and recompile:
//   typst compile tshirt.typ tshirt.pdf
//   typst compile --input transparent=1 tshirt.typ tshirt.png   (transparent bg)

// ===================== WORDS =====================
#let title_main = "HARNESSING QUANTUM"
#let title_year = "2026"
#let subtitle = [AI AGENTS × QUANTUM SIMULATION]
#let term_prompt = "$ agent requests permission:"
#let term_pick = "❯ 1. YOLO"
#let term_note = "# the only mode"
#let footer = [EMBRACE AGENTS · HEFEI NATIONAL LABORATORY · JUL 27–31]
// =================================================

#let gold = rgb("#E6C15A")
#let coral = rgb("#D97757")
#let dim = rgb("#8a8a8a")
#let mono = "Menlo"
#let sans = "Helvetica Neue"

#let bg = if sys.inputs.at("transparent", default: "0") == "1" { none } else { rgb("#000000") }
#set page(width: 1200pt, height: 1160pt, margin: 0pt, fill: bg)

// ---- title ----
#place(top + center, dy: 90pt,
  text(font: sans, size: 62pt, weight: "bold", tracking: 10pt,
    [#text(fill: gold, title_main)#h(18pt)#text(fill: coral, title_year)]))
#place(top + center, dy: 172pt,
  text(font: sans, size: 24pt, weight: "medium", tracking: 8pt, fill: rgb("#7d7d7d"), subtitle))

// ---- evolution: ape -> human -> agents ----
#place(top + left, dx: 130pt, dy: 495pt, image("assets/ape.svg", height: 115pt))
#place(top + left, dx: 505pt, dy: 400pt, image("assets/human_staff.svg", height: 210pt))
#place(top + left, dx: 610pt, dy: 478pt, line(start: (0pt, 0pt), end: (58pt, 0pt), stroke: 5pt + dim))
#place(top + left, dx: 662pt, dy: 468pt, polygon(fill: dim, (0pt, 0pt), (22pt, 10pt), (0pt, 20pt)))

// ---- 2x2 agent tiles (extracted real marks) ----
#place(top + left, dx: 710pt, dy: 352pt, image("assets/tile_claude.svg", width: 120pt))
#place(top + left, dx: 854pt, dy: 352pt, image("assets/tile_openai.svg", width: 120pt))
#place(top + left, dx: 710pt, dy: 496pt, image("assets/tile_deepseek.svg", width: 120pt))
#place(top + left, dx: 854pt, dy: 496pt, image("assets/tile_kimi.svg", width: 120pt))

// ---- terminal punchline ----
#place(top + left, dx: 270pt, dy: 740pt,
  rect(width: 660pt, height: 240pt, radius: 14pt, fill: rgb("#0d1117"), stroke: 1.5pt + rgb("#2a2f36")))
#for i in range(3) {
  place(top + left, dx: (296 + i * 20) * 1pt, dy: 764pt, circle(radius: 6pt, fill: rgb("#3a3a3a")))
}
#place(top + left, dx: 300pt, dy: 800pt,
  text(font: mono, size: 30pt, fill: rgb("#8b949e"), term_prompt))
#place(top + left, dx: 312pt, dy: 850pt,
  text(font: mono, size: 38pt, weight: "bold", fill: coral, term_pick))
#place(top + left, dx: 312pt, dy: 916pt,
  text(font: mono, size: 28pt, fill: rgb("#5a6169"), term_note))

// ---- footer ----
#place(top + center, dy: 1070pt,
  text(font: sans, size: 25pt, weight: "medium", tracking: 7pt, fill: dim, footer))
