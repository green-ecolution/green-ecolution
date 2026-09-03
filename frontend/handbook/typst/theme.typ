#import "../generated/typst/colors.typ": colors

#let body-font = "Nunito Sans 10pt"
#let display-font = "Lato"
#let mono-font = "JetBrains Mono"

#let txt(value) = [#value]
#let tech(value) = text(font: mono-font, size: 0.92em, fill: colors.at("green-dark"))[#value]
#let para(..runs) = block(below: 0.9em)[#runs.pos().join()]

#let part-number = counter("handbook-part")

#let part(id, title) = {
  part-number.step()
  heading(level: 1, title)
}

#let chapter(slug, title) = {
  pagebreak(weak: true)
  [#heading(level: 2, title) #label("ch-" + slug)]
}

#let section(level, anchor, title) = [
  #heading(level: level + 1, title) #label("sec-" + anchor)
]

#let manual-theme(body) = {
  set document(title: "Green Ecolution — Nutzerhandbuch")
  set page(
    paper: "a4",
    fill: colors.at("light"),
    margin: (inside: 2.4cm, outside: 5.2cm, top: 2.6cm, bottom: 2.4cm),
    binding: left,
    header: context {
      let seen = query(selector(heading.where(level: 2)).before(here()))
      if seen.len() == 0 { return }
      grid(
        columns: (1fr, auto),
        text(font: mono-font, size: 8pt, fill: colors.at("dark-500"))[#upper(seen.last().body)],
        text(font: mono-font, size: 8pt, fill: colors.at("dark-500"))[#counter(page).display()],
      )
      line(length: 100%, stroke: 0.5pt + colors.at("dark-100"))
    },
  )
  set text(font: body-font, size: 10.5pt, fill: colors.at("dark"), lang: "de", hyphenate: true)
  set par(justify: true, leading: 0.72em, spacing: 0.9em)
  set heading(numbering: none)

  // A part divider owns its page, so the reader feels the section change
  // before reading it. page() inside the show rule provides that break.
  show heading.where(level: 1): it => page(
    fill: colors.at("green-light-50"),
    header: none,
    margin: (x: 3cm, y: 4cm),
    align(
      horizon + left,
      stack(
        spacing: 0.6em,
        text(font: display-font, weight: "black", size: 96pt, fill: colors.at("green-dark-200"))[
          #context part-number.display("I")
        ],
        text(font: display-font, weight: "bold", size: 34pt, fill: colors.at("green-dark"))[#it.body],
      ),
    ),
  )

  show heading.where(level: 2): it => block(below: 1.4em)[
    #text(font: mono-font, size: 8pt, tracking: 1.5pt, fill: colors.at("dark-500"))[KAPITEL]
    #block(above: 0.3em)[
      #text(font: display-font, weight: "bold", size: 26pt, fill: colors.at("green-dark"))[#it.body]
    ]
    #block(above: 0.6em)[#line(length: 100%, stroke: 0.6pt + colors.at("dark-100"))]
  ]

  show heading.where(level: 3): it => block(above: 1.6em, below: 0.7em)[
    #text(font: display-font, weight: "bold", size: 16pt, fill: colors.at("dark"))[#it.body]
  ]

  show heading.where(level: 4): it => block(above: 1.3em, below: 0.6em)[
    #text(font: display-font, weight: "bold", size: 13pt, fill: colors.at("dark"))[#it.body]
  ]

  body
}
