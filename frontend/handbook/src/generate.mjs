import { mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { parseChapter } from './chapter.mjs'
import { emitChapter, typstString } from './typst.mjs'

function chapterLinksInRuns(runs, seen) {
  for (const run of runs) {
    if (run.kind === 'link') {
      if (run.target.kind === 'chapter') seen.push(run.target.slug)
      chapterLinksInRuns(run.children, seen)
      continue
    }
    if (run.kind === 'strong' || run.kind === 'emphasis') chapterLinksInRuns(run.children, seen)
  }
}

function chapterLinks(blocks, seen = []) {
  for (const block of blocks) {
    switch (block.kind) {
      case 'paragraph':
        chapterLinksInRuns(block.children, seen)
        break
      case 'list':
      case 'steps':
        for (const item of block.items) chapterLinksInRuns(item, seen)
        break
      case 'callout':
        chapterLinks(block.children, seen)
        break
      case 'table':
        for (const cell of block.head) chapterLinksInRuns(cell, seen)
        for (const row of block.rows) for (const cell of row) chapterLinksInRuns(cell, seen)
        break
      default:
        break
    }
  }
  return seen
}

export async function generate({ root, language }) {
  const contentDir = join(root, 'content', language)
  const parts = JSON.parse(await readFile(join(contentDir, 'parts.json'), 'utf8'))
  const files = (await readdir(contentDir)).filter((file) => file.endsWith('.md')).sort()

  const chapters = {}
  const contents = {}
  const search = []

  for (const file of files) {
    const source = await readFile(join(contentDir, file), 'utf8')
    const { meta, blocks, search: entries } = parseChapter(source, { file })

    if (chapters[meta.slug]) throw new Error(`handbook: ${file}: duplicate slug "${meta.slug}"`)
    if (!parts.some((part) => part.id === meta.part)) {
      throw new Error(`handbook: ${file}: unknown part "${meta.part}"`)
    }

    chapters[meta.slug] = meta
    contents[meta.slug] = { slug: meta.slug, blocks }
    search.push(...entries)
  }

  for (const [slug, content] of Object.entries(contents)) {
    for (const target of chapterLinks(content.blocks)) {
      if (!chapters[target]) throw new Error(`handbook: ${slug}: unknown chapter "${target}"`)
    }
  }

  const index = {
    parts: parts.map((part) => ({
      ...part,
      chapters: Object.values(chapters)
        .filter((meta) => meta.part === part.id)
        .map((meta) => meta.slug),
    })),
    chapters,
  }

  const out = join(root, 'generated')
  await rm(out, { recursive: true, force: true })
  await mkdir(join(out, 'chapters'), { recursive: true })
  await writeFile(join(out, 'index.json'), `${JSON.stringify(index, null, 2)}\n`)
  await writeFile(join(out, 'search.json'), `${JSON.stringify(search, null, 2)}\n`)
  for (const content of Object.values(contents)) {
    await writeFile(
      join(out, 'chapters', `${content.slug}.json`),
      `${JSON.stringify(content, null, 2)}\n`,
    )
  }

  await mkdir(join(out, 'typst'), { recursive: true })
  for (const content of Object.values(contents)) {
    await writeFile(
      join(out, 'typst', `${content.slug}.typ`),
      emitChapter(content, chapters[content.slug]),
    )
  }
  const order = index.parts.flatMap((part) => [
    `#part(${typstString(part.id)}, ${typstString(part.title)})`,
    ...part.chapters.map((slug) => `#include "${slug}.typ"`),
  ])
  await writeFile(
    join(out, 'typst', 'chapters.typ'),
    `#import "../../typst/blocks.typ": *\n${order.join('\n')}\n`,
  )

  return { index, chapters: contents, search }
}
