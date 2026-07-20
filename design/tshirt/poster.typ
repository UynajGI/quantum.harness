// Classical vs Modern Scientific Computing — poster graphic (Typst master)
// Rebuilt from the school's original poster; figures extracted from the poster
// bitmap, Claude star from the official claude.ai asset. Edit the words below:
//   typst compile poster.typ poster.pdf

// ===================== WORDS =====================
#let row1_title = "Classical Scientific Computing"
#let row2_title = "Modern Scientific Computing (with skills)"
#let key_label = "TAB"
#let term_prompt = "$ agent requests permission:"
#let term_pick = "❯ 1. YOLO"
#let term_note = "# the only mode"
// =================================================

#let coral = rgb("#D97757")
#let mono = "Menlo"
#let classic = rgb("#8a8a8a")   // dimmed classical row
#let modern = rgb("#ffffff")    // bright modern row

#set page(width: 1500pt, height: 980pt, margin: 0pt, fill: rgb("#000000"))

// ---- keycap (label is tunable) ----
#let keycap(label, dx, dy) = {
  place(top + left, dx: dx + 12pt, dy: dy - 14pt,
    rect(width: 170pt, height: 140pt, radius: 20pt, stroke: 5pt + rgb("#6b6b6b")))
  place(top + left, dx: dx, dy: dy,
    rect(width: 170pt, height: 140pt, radius: 20pt, fill: rgb("#0a0a0a"), stroke: 5pt + rgb("#c9c9c9")))
  place(top + left, dx: dx, dy: dy,
    box(width: 170pt, height: 140pt,
      align(center + horizon, text(font: mono, size: 42pt, fill: rgb("#c9c9c9"), label))))
}

// ---- row 1: classical ----
#place(top + left, dx: 60pt, dy: 55pt, text(font: mono, size: 48pt, fill: classic, row1_title))
#place(top + left, dx: 300pt, dy: 215pt, image("assets/ape.svg", height: 135pt))
#keycap(key_label, 560pt, 215pt)
#keycap(key_label, 810pt, 215pt)
#keycap(key_label, 1060pt, 215pt)

// ---- divider ----
#place(top + left, dx: 60pt, dy: 470pt, line(start: (0pt, 0pt), end: (1380pt, 0pt), stroke: 2pt + rgb("#262626")))

// ---- row 2: modern ----
#place(top + left, dx: 60pt, dy: 525pt, text(font: mono, size: 48pt, fill: modern, row2_title))
#place(top + left, dx: 300pt, dy: 685pt, image("assets/human_staff.svg", height: 220pt))

// terminal
#place(top + left, dx: 540pt, dy: 650pt,
  rect(width: 640pt, height: 250pt, radius: 14pt, fill: rgb("#0d1117"), stroke: 1.5pt + rgb("#2a2f36")))
#for i in range(3) {
  place(top + left, dx: (566 + i * 20) * 1pt, dy: 676pt, circle(radius: 6pt, fill: rgb("#3a3a3a")))
}
#place(top + left, dx: 570pt, dy: 715pt, text(font: mono, size: 32pt, fill: rgb("#8b949e"), term_prompt))
#place(top + left, dx: 582pt, dy: 772pt, text(font: mono, size: 42pt, weight: "bold", fill: coral, term_pick))
#place(top + left, dx: 582pt, dy: 838pt, text(font: mono, size: 28pt, fill: rgb("#5a6169"), term_note))

// spark + Claude star (extracted from poster / official asset)
#place(top + left, dx: 1252pt, dy: 684pt, image("assets/spark.svg", width: 44pt))
#place(top + left, dx: 1310pt, dy: 715pt, image("assets/claude_star.svg", height: 140pt))
