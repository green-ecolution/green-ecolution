#import "theme.typ": *
#import "blocks.typ": *
#import "../generated/typst/colors.typ": colors

#import "../generated/typst/meta.typ": build-date, version

#set page(paper: "a4", fill: colors.at("light"), margin: 0pt, header: none, footer: none)

// The cover's own ground: the brand green is a mid tone, too light to carry
// white type across a full page, so it is deepened here rather than added to
// the generated palette — this is the only place that needs it.
#let cover-ink = colors.at("green-dark").darken(48%)
#let cover-accent = colors.at("green-light").darken(10%)

#let cover-label(body, fill: colors.at("dark-500"), size: 8pt) = text(
  font: mono-font,
  size: size,
  tracking: 2.2pt,
  fill: fill,
)[#upper(body)]

// Title page
#page(fill: cover-ink)[
  // The mark as a watermark: typst cannot fade an image, so the icon is
  // covered by a near-opaque veil of the ground it sits on.
  #place(right + bottom, dx: 3.6cm, dy: -4.4cm, box(width: 11.5cm, height: 11.5cm)[
    #image("assets/icon-white.svg", width: 100%)
    #place(top + left, rect(width: 11.5cm, height: 11.5cm, fill: cover-ink.transparentize(6%)))
  ])

  #block(inset: (x: 3.2cm, top: 3.2cm, bottom: 3.2cm), height: 100%)[
    #image("assets/logo-white.svg", width: 7.4cm)

    #place(left + horizon, dy: 0.4cm)[
      #cover-label("Green Ecolution", fill: cover-accent)
      #v(1.1em, weak: false)
      #text(
        font: display-font,
        weight: "regular",
        size: 52pt,
        fill: colors.at("light"),
        tracking: 0.5pt,
      )[Nutzerhandbuch]
      #v(1.6em, weak: false)
      #line(length: 7.2cm, stroke: 0.6pt + cover-accent)
      #v(1.6em, weak: false)
      #text(font: body-font, size: 12.5pt, fill: colors.at("green-dark-200"))[
        Smartes Grünflächenmanagement\
        Bedienung von Green Ecolution
      ]
    ]

    #place(left + bottom)[
      #line(length: 14.6cm, stroke: 0.4pt + colors.at("green-dark").darken(18%))
      #v(0.9em, weak: false)
      #grid(
        columns: (1fr, auto),
        cover-label("PROGEEK GmbH", fill: colors.at("green-dark-400"), size: 7.5pt),
        cover-label(
          version + "  ·  Stand " + build-date,
          fill: colors.at("green-dark-400"),
          size: 7.5pt,
        ),
      )
    ]
  ]
]

// Imprint. Same measure and same top/bottom hairlines as the cover, so the
// two opening pages read as one movement into the book.
#page(fill: colors.at("light"))[
  #block(inset: (x: 3.2cm, top: 3.2cm, bottom: 3.2cm), height: 100%)[
    #cover-label("Nutzerhandbuch")
    #v(0.9em, weak: false)
    #line(length: 100%, stroke: 0.5pt + colors.at("dark-200"))

    #place(left + horizon, dy: -0.4cm)[
      #block(width: 11.4cm)[
        #text(font: display-font, weight: "bold", size: 22pt, fill: colors.at("green-dark"))[
          Über dieses Handbuch
        ]
        #v(1.4em, weak: false)
        #line(length: 4.6cm, stroke: 0.6pt + colors.at("green-dark"))
        #v(1.5em, weak: false)
        #text(font: body-font, size: 11pt, fill: colors.at("dark"))[
          Dieses Handbuch beschreibt die Bedienung von Green Ecolution für alle Rollen,
          vom täglichen Erfassen und Planen bis zur Verwaltung von Organisationen und
          Rechten. Es entsteht aus derselben Quelle wie die Hilfe in der Anwendung und
          ist damit immer auf demselben Stand.
        ]
        #v(1.6em, weak: false)
        #text(font: body-font, size: 9.5pt, fill: colors.at("dark-600"))[
          Green Ecolution ist ein Projekt der PROGEEK GmbH. Die Software steht unter der
          AGPL-3.0-only.
        ]
      ]
    ]

    #place(left + bottom)[
      #line(length: 100%, stroke: 0.5pt + colors.at("dark-200"))
      #v(0.9em, weak: false)
      #grid(
        columns: (1fr, auto),
        cover-label("PROGEEK GmbH", size: 7.5pt),
        link("https://green-ecolution.de", cover-label("green-ecolution.de", size: 7.5pt)),
      )
    ]
  ]
]

#show: manual-theme

// Table of contents. "Inhalt" is set as plain styled text, not a heading,
// so the level-1 show rule in theme.typ only ever sees part dividers.
#block(below: 1em)[
  #text(font: display-font, weight: "bold", size: 22pt, fill: colors.at("green-dark"))[Inhalt]
]
#outline(title: none, depth: 2, indent: auto)

#include "../generated/typst/chapters.typ"
