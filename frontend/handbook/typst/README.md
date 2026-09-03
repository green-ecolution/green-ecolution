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
  store copy was used instead as the legitimate canonical source. License:
  SIL Open Font License 1.1 (`OFL.txt` in the Lato2OFL package).
- **NunitoSans-Italic.ttf** — built from the Nunito Sans Italic variable font
  published at `google/fonts`, path
  `ofl/nunitosans/NunitoSans-Italic[YTLC,opsz,wdth,wght].ttf`, upstream
  source `https://github.com/googlefonts/NunitoSans` at commit
  `058bd7a2f33d6ad5ef1df985b3db403622016a8c` (version `3.101`,
  `gftools[0.9.27]`). Instanced to a static Regular-weight italic
  (`wght=400`, `wdth=100`, `opsz=10`, `YTLC=500`) with
  `fonttools varLib.instancer`, matching the same opsz=10 pin and
  `Nunito Sans 10pt` family naming already used for Regular/Medium/SemiBold/
  Bold in `frontend/app/public/fonts/nunito-sans/`. License: SIL Open Font
  License 1.1.
- **JetBrainsMono-Regular.ttf** — from the official JetBrains Mono GitHub
  release `v2.304`
  (`https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip`),
  file `fonts/ttf/JetBrainsMono-Regular.ttf`. This is the plain typeface, not
  the Nerd Font variant already present elsewhere on this machine, which
  bundles glyph sets far beyond what the handbook needs. License: SIL Open
  Font License 1.1 (`OFL.txt` in the release archive).
