# Handbook PDF

`just handbook-pdf` renders `typst/manual.typ` into
`../app/public/handbook/green-ecolution-handbuch.pdf`.

Chapter sources live in `../content/de/`; the `.typ` files under
`../generated/typst/` are produced by `../src/cli.mjs` and are not committed.

## Fonts

`fonts/` carries the weights the app repository does not ship. All four are
licensed under the SIL Open Font License 1.1:

- Lato Black, Lato Italic — Łukasz Dziedzic
- Nunito Sans Italic — Vernon Adams, Cyreal, Jacques Le Bailly
- JetBrains Mono Regular — JetBrains

They are vendored so the PDF build does not depend on system fonts.

### Provenance

- **Lato-Black.ttf**, **Lato-Italic.ttf** — copied from the `lato` package in
  the Nix store (`pname = "lato"`, `version = "2.0"`), which fetches
  `https://www.latofonts.com/files/Lato2OFL.zip` (the "Lato2OFL" package
  distributed by the type designer). Google Fonts' own repository
  (`google/fonts`, `ofl/lato/`) currently serves an identical-size release of
  these two weights, not the smaller subset already vendored for Lato
  Regular/Bold/SemiBold in `frontend/app/public/fonts/lato/`, so the Nix
  store copy was used instead as the legitimate canonical source. Built by
  nixpkgs revision `c5c4a43b0e8056328ec4529f735cabdb8f1942bb` (the `nixpkgs`
  flake registry entry at the time of vendoring; `nix build nixpkgs#lato`
  against it reproduces byte-identical files, confirmed with `sha256sum`).
  License: SIL Open Font License 1.1 (`OFL.txt` in the Lato2OFL package).
- **NunitoSans-Italic.ttf** — built from the Nunito Sans Italic variable font
  published at `google/fonts`, path
  `ofl/nunitosans/NunitoSans-Italic[YTLC,opsz,wdth,wght].ttf`, upstream
  source `https://github.com/googlefonts/NunitoSans` at commit
  `058bd7a2f33d6ad5ef1df985b3db403622016a8c` (version `3.101`,
  `gftools[0.9.27]`). License: SIL Open Font License 1.1. Reproduce with:

  1. Download that file and instance it to a static Regular-weight italic:
     `fonttools varLib.instancer -o out.ttf NunitoSans-Italic[YTLC,opsz,wdth,wght].ttf wght=400 wdth=100 opsz=10 YTLC=500`.
     This matches the opsz=10 pin already used for Regular/Medium/SemiBold/
     Bold in `frontend/app/public/fonts/nunito-sans/`. Don't pass
     `--update-name-table`: the instancer refuses it because `opsz=10` isn't
     a registered STAT named value (only `opsz=12` has one), so it can't
     derive names on its own.
  2. Rewrite the `name` table by hand to the values the other pinned weights
     in this family already use, replacing whatever the instancer carried
     over from the "12pt ExtraLight Italic" source instance (dump with
     `ttx -t name`, edit, recompile with `ttx -o result.ttf edited.ttx`):

     | Name ID | Field                        | Value                              |
     | ------- | ---------------------------- | ---------------------------------- |
     | 1       | Family                       | `Nunito Sans 10pt`                 |
     | 2       | Subfamily                    | `Italic`                           |
     | 3       | Unique ID                    | `3.101;NONE;NunitoSans10pt-Italic` |
     | 4       | Full name                    | `Nunito Sans 10pt Italic`          |
     | 6       | PostScript name              | `NunitoSans10pt-Italic`            |
     | 16      | Typographic family           | `Nunito Sans 10pt`                 |
     | 17      | Typographic subfamily        | `Italic`                           |
     | 25      | Variations PostScript prefix | `NunitoSans`                       |

     `OS/2.fsSelection` and `head.macStyle` already carry the correct italic
     bits after step 1 and need no further change. The family name is what
     Typst resolves against, so getting it wrong silently produces a face
     the theme can't find.

- **JetBrainsMono-Regular.ttf** — from the official JetBrains Mono GitHub
  release `v2.304`
  (`https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip`),
  file `fonts/ttf/JetBrainsMono-Regular.ttf`. This is the plain typeface, not
  the Nerd Font variant already present elsewhere on this machine, which
  bundles glyph sets far beyond what the handbook needs. License: SIL Open
  Font License 1.1 (`OFL.txt` in the release archive).
