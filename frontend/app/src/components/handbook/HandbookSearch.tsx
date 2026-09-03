import { useEffect, useRef, useState } from 'react'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Search } from 'lucide-react'
import { Input } from '@green-ecolution/ui'
import { loadSearchEntries } from '@/lib/handbook'
import { isSearchableQuery, searchHandbook, type SearchHit } from '@/lib/handbook/search'
import type { SearchEntry } from '@/lib/handbook/types'

function HandbookSearch() {
  const { t } = useTranslation('help')
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

  const hits: SearchHit[] = entries ? searchHandbook(entries, query) : []
  const searching = isSearchableQuery(query)

  return (
    <section>
      <label htmlFor="handbook-search" className="font-lato font-semibold block mb-2">
        {t('search.label')}
      </label>
      <div className="relative max-w-xl">
        <Search className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 size-5 text-dark-400" />
        <Input
          id="handbook-search"
          type="search"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t('search.placeholder')}
          className="pl-10"
        />
      </div>

      {searching && loadError && <p className="mt-4 text-dark-600">{t('search.error')}</p>}

      {searching && !loadError && entries && hits.length === 0 && (
        <p className="mt-4 text-dark-600">{t('search.empty', { query })}</p>
      )}

      {hits.length > 0 && (
        <ul aria-label={t('search.resultsLabel')} className="mt-4 max-w-xl space-y-2">
          {hits.map((hit) => (
            <li key={`${hit.slug}#${hit.anchor}`}>
              <Link
                to="/help/$slug"
                params={{ slug: hit.slug }}
                hash={hit.anchor}
                className="block rounded-xl border border-dark-100 p-3 transition-colors duration-base ease-out hover:border-green-dark"
              >
                <span className="text-sm text-dark-600">{hit.chapterTitle}</span>
                <span className="font-lato font-semibold block">{hit.sectionTitle}</span>
                {hit.excerpt && (
                  <span className="mt-1 block text-sm text-dark-600">{hit.excerpt}</span>
                )}
              </Link>
            </li>
          ))}
        </ul>
      )}
    </section>
  )
}

export default HandbookSearch
