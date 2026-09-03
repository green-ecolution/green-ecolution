#import "theme.typ": *
#import "blocks.typ": *
#import "../generated/typst/colors.typ": colors

#import "../generated/typst/meta.typ": build-date, version

#set page(paper: "a4", fill: colors.at("light"), margin: 0pt, header: none, footer: none)

// Title page
#page[
  #block(inset: (x: 3cm, y: 4cm))[
    #image("assets/logo.svg", width: 7cm)
    #block(above: 3cm)[
      #text(font: display-font, weight: "black", size: 44pt, fill: colors.at("green-dark"))[
        Nutzerhandbuch
      ]
    ]
    #block(above: 0.8em)[
      #text(font: display-font, weight: "regular", size: 16pt, fill: colors.at("dark-600"))[
        Smartes Grünflächenmanagement — Bedienung von Green Ecolution
      ]
    ]
    #block(above: 2.5cm)[
      #text(font: mono-font, size: 9pt, fill: colors.at("dark-500"))[
        #version · Stand #build-date
      ]
    ]
  ]
]

// Imprint
#page(fill: colors.at("green-light-50"))[
  #block(inset: (x: 3cm, y: 4cm))[
    #text(font: display-font, weight: "bold", size: 16pt, fill: colors.at("green-dark"))[
      Über dieses Handbuch
    ]
    #block(above: 1em)[
      #text(font: body-font, size: 10.5pt, fill: colors.at("dark"))[
        Dieses Handbuch beschreibt die Bedienung von Green Ecolution für alle Rollen —
        vom täglichen Erfassen und Planen bis zur Verwaltung von Organisationen und
        Rechten. Es entsteht aus derselben Quelle wie die Hilfe in der Anwendung und
        ist damit immer auf demselben Stand.
      ]
    ]
    #block(above: 2em)[
      #text(font: body-font, size: 9pt, fill: colors.at("dark-600"))[
        Green Ecolution ist ein Projekt der Hochschule Flensburg, der PROGEEK GmbH und
        der Stadt Flensburg. Die Software steht unter der AGPL-3.0-only.
        #linebreak()
        #link("https://green-ecolution.de")[green-ecolution.de]
      ]
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
