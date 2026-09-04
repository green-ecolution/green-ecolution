import indexJson from '../../../../handbook/generated/index.json'
import type { ChapterContent, HandbookIndex, SearchEntry } from './types'

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
