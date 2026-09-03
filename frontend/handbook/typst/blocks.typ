#import "theme.typ": *
#import "../generated/typst/colors.typ": colors

#let figure-counter = counter("handbook-figure")

#let bullets(items) = block(below: 0.9em)[
  #for item in items [
    #grid(
      columns: (0.9em, 1fr),
      column-gutter: 0.4em,
      row-gutter: 0.35em,
      text(fill: colors.at("green-dark"))[•], item,
    )
  ]
]

#let steps(items) = block(below: 0.9em)[
  #for (i, item) in items.enumerate() [
    #grid(
      columns: (1.5em, 1fr),
      column-gutter: 0.6em,
      row-gutter: 0.5em,
      align(center + top)[
        #circle(
          radius: 0.62em,
          fill: colors.at("green-dark"),
          align(center + horizon, text(size: 0.72em, fill: colors.at("light"), weight: "bold")[#(i + 1)]),
        )
      ],
      item,
    )
  ]
]

#let callout-tones = (
  note: ("green-light-200", "green-light-50", "Hinweis"),
  tip: ("green-dark-100", "green-dark-50", "Tipp"),
  important: ("dark-200", "dark-50", "Wichtig"),
  warning: ("yellow-100", "yellow-50", "Achtung"),
)

#let callout(tone, body) = {
  let (border, background, label-text) = callout-tones.at(tone)
  block(
    width: 100%,
    above: 1.1em,
    below: 1.1em,
    fill: colors.at(background),
    stroke: (left: 2.5pt + colors.at(border)),
    radius: (rest: 4pt, left: 0pt),
    inset: (x: 0.9em, y: 0.8em),
  )[
    #block(below: 0.5em)[
      #text(font: display-font, weight: "bold", size: 8pt, tracking: 1.2pt, fill: colors.at("dark-700"))[
        #upper(label-text)
      ]
    ]
    #body
  ]
}

#let figure-image(file, caption) = {
  figure-counter.step()
  block(above: 1.4em, below: 1.4em)[
    #block(
      radius: 6pt,
      clip: true,
      stroke: 0.5pt + colors.at("dark-100"),
      image("../images/" + file, width: 100%),
    )
    #block(above: 0.5em)[
      #text(font: body-font, style: "italic", size: 8.5pt, fill: colors.at("dark-600"))[
        #context [Abb. #figure-counter.display() — ] #caption
      ]
    ]
  ]
}

#let data-table(head, rows) = block(above: 1.2em, below: 1.2em)[
  #table(
    columns: (1fr,) * head.len(),
    stroke: (x, y) => if y > 0 { (top: 0.5pt + colors.at("dark-50")) },
    inset: (x: 0.6em, y: 0.5em),
    fill: (_, y) => if y == 0 { colors.at("green-light-50") } else { none },
    table.header(..head.map((cell) => text(weight: "bold")[#cell])),
    ..rows.flatten(),
  )
]

#let code-block(language, value) = block(
  width: 100%,
  above: 1.1em,
  below: 1.1em,
  fill: colors.at("dark-50"),
  radius: 4pt,
  inset: 0.8em,
)[
  #text(font: mono-font, size: 8.5pt, fill: colors.at("dark-900"))[#raw(value, lang: language)]
]

#let link-external(href, body) = link(href)[#underline(text(fill: colors.at("green-dark"))[#body])]

#let xref-chapter(slug, anchor, body) = {
  let target = label(if anchor == none { "ch-" + slug } else { "sec-" + anchor })
  [#body #h(0.25em) #context {
    let hits = query(target)
    if hits.len() > 0 {
      text(size: 0.85em, fill: colors.at("dark-600"))[(siehe S. #hits.first().location().page())]
    }
  }]
}

#let app-route(to, body) = [#body #h(0.2em) #tech(to)]
