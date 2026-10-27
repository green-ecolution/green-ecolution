import indexJson from '../../../../handbook/generated/index.json'
import type {
  ChapterContent,
  ChapterLink,
  ChapterNeighbours,
  ChapterStep,
  HandbookIndex,
  SearchEntry,
} from './types'

export const handbookIndex = indexJson as HandbookIndex

const byChapterSlug = <T>(entries: [string, T][]): Record<string, T> =>
  Object.fromEntries(
    entries.map(([path, value]) => [
      path
        .split('/')
        .pop()!
        .replace(/\.json$/, ''),
      value,
    ]),
  )

const byFileName = <T>(entries: [string, T][]): Record<string, T> =>
  Object.fromEntries(entries.map(([path, value]) => [path.split('/').pop()!, value]))

const chapterLoaders = byChapterSlug(
  Object.entries(
    import.meta.glob<{ default: ChapterContent }>('../../../../handbook/generated/chapters/*.json'),
  ),
)

const images = byFileName(
  Object.entries(
    import.meta.glob<string>('../../../../handbook/images/*.png', {
      eager: true,
      query: '?url',
      import: 'default',
    }),
  ),
)

export async function loadChapter(slug: string): Promise<ChapterContent> {
  const load = chapterLoaders[slug]
  if (!load) throw new Error(`handbook: unknown chapter "${slug}"`)
  return (await load()).default
}

export async function loadSearchEntries(): Promise<SearchEntry[]> {
  const module = await import('../../../../handbook/generated/search.json')
  return module.default
}

export function imageUrl(file: string): string {
  const url = images[file]
  if (!url) throw new Error(`handbook: missing image "${file}"`)
  return url
}

/** The order the parts spell out, which is also the order the PDF sets the chapters in. */
const readingOrder = handbookIndex.parts.flatMap((part) => part.chapters)

const partTitles = new Map(handbookIndex.parts.map((part) => [part.id, part.title]))

function chapterLink(slug: string): ChapterLink {
  const meta = handbookIndex.chapters[slug]
  return {
    slug,
    title: meta.title,
    partTitle: partTitles.get(meta.part) ?? '',
  }
}

/** The chapter at hand stays in the order even when hidden, so a shared link still pages on. */
export function chapterNeighbours(
  slug: string,
  isVisible: (slug: string) => boolean = () => true,
): ChapterNeighbours {
  if (!readingOrder.includes(slug)) throw new Error(`handbook: unknown chapter "${slug}"`)
  const order = readingOrder.filter((candidate) => candidate === slug || isVisible(candidate))
  const position = order.indexOf(slug)

  const herePart = handbookIndex.chapters[slug].part
  const step = (neighbour: string | undefined): ChapterStep | null => {
    if (!neighbour) return null
    return {
      ...chapterLink(neighbour),
      entersNewPart: handbookIndex.chapters[neighbour].part !== herePart,
    }
  }

  return {
    previous: step(order[position - 1]),
    next: step(order[position + 1]),
  }
}

/** The chapter the handbook opens with, for the "read from the start" entry point. */
export function firstChapter(isVisible: (slug: string) => boolean = () => true): ChapterLink {
  return chapterLink(readingOrder.find(isVisible) ?? readingOrder[0])
}
