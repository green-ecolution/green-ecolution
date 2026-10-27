import { useEffect, useRef, useState } from 'react'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Search } from 'lucide-react'
import { Input } from '@green-ecolution/ui'
import { loadSearchEntries } from '@/lib/handbook'
import { useChapterVisibility } from '@/lib/handbook/useChapterVisibility'
import {
  isSearchableQuery,
  searchHandbook,
  splitOnMatch,
  type SearchHit,
} from '@/lib/handbook/search'
import type { SearchEntry } from '@/lib/handbook/types'

function Marked({ text, query }: { text: string; query: string }) {
  const parts = splitOnMatch(text, query)
  if (!parts) return <>{text}</>

  return (
    <>
      {parts.before}
      <mark className="rounded-sm bg-green-light-200 text-dark">{parts.match}</mark>
      {parts.after}
    </>
  )
}

function HandbookSearch({ variant = 'page' }: { variant?: 'page' | 'sidebar' }) {
  const { t } = useTranslation('help')
  const isVisible = useChapterVisibility()
  const [query, setQuery] = useState('')
  const [entries, setEntries] = useState<SearchEntry[] | null>(null)
  const [loadError, setLoadError] = useState(false)
  const loadStarted = useRef(false)
  const cancelled = useRef(false)

  useEffect(() => {
    cancelled.current = false
    return () => {
      cancelled.current = true
    }
  }, [])

  // The search text is a separate chunk, fetched on first use so the overview
  // page does not carry the whole handbook body. Deliberately fires at most
  // once per mount (loadStarted), not once per keystroke.
  useEffect(() => {
    if (!isSearchableQuery(query) || loadStarted.current) return
    loadStarted.current = true
    loadSearchEntries()
      .then((loaded) => {
        if (!cancelled.current) setEntries(loaded)
      })
      .catch((error: unknown) => {
        if (cancelled.current) return
        console.error('handbook: failed to load search entries', error)
        setLoadError(true)
      })
  }, [query])

  const hits: SearchHit[] = entries
    ? searchHandbook(
        entries.filter((entry) => isVisible(entry.slug)),
        query,
      )
    : []
  const searching = isSearchableQuery(query)
  const inSidebar = variant === 'sidebar'

  const feedback = (
    <>
      {searching && loadError && <p className="text-sm text-dark-600">{t('search.error')}</p>}
      {searching && !loadError && entries && (
        <p role="status" className="text-sm text-dark-600">
          {hits.length > 0
            ? t('search.resultCount', { count: hits.length })
            : t('search.empty', { query })}
        </p>
      )}
      {hits.length > 0 && (
        <ul aria-label={t('search.resultsLabel')} className="mt-3 space-y-2">
          {hits.map((hit) => (
            <li key={`${hit.slug}#${hit.anchor}`}>
              <Link
                to="/help/$slug"
                params={{ slug: hit.slug }}
                hash={hit.anchor || undefined}
                className="block rounded-xl px-3 py-2 transition-colors duration-base ease-emphasized hover:bg-green-light-50"
              >
                <span className="text-xs text-dark-500">{hit.chapterTitle}</span>
                <span className="font-lato font-semibold block text-green-dark">
                  <Marked text={hit.sectionTitle} query={query} />
                </span>
                {hit.excerpt && (
                  <span className="mt-1 block text-sm text-dark-600">
                    <Marked text={hit.excerpt} query={query} />
                  </span>
                )}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </>
  )

  return (
    <section className="relative">
      <label
        htmlFor="handbook-search"
        className={inSidebar ? 'sr-only' : 'font-lato font-semibold block mb-2'}
      >
        {t('search.label')}
      </label>
      <div className={`relative ${inSidebar ? '' : 'max-w-xl'}`}>
        <Search className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 size-5 text-dark-400" />
        <Input
          id="handbook-search"
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Escape') setQuery('')
          }}
          placeholder={t('search.placeholder')}
          className="pl-10"
        />
      </div>

      {searching &&
        (inSidebar ? (
          // Wider than the sidebar column it hangs in, so a hit stays readable.
          <div className="absolute right-0 z-20 mt-2 w-80 max-w-[calc(100vw-2rem)] rounded-xl border border-dark-100 bg-white p-3 shadow-cards">
            {feedback}
          </div>
        ) : (
          <div className="mt-4 max-w-xl">{feedback}</div>
        ))}
    </section>
  )
}

export default HandbookSearch
