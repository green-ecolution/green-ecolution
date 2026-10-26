import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Download, MoveRight } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import { firstChapter, handbookIndex } from '@/lib/handbook'
import { useChapterVisibility } from '@/lib/handbook/useChapterVisibility'
import { visibleParts } from '@/lib/handbook/visibility'
import HandbookSearch from '@/components/handbook/HandbookSearch'
import LanguageFallbackNotice from '@/components/handbook/LanguageFallbackNotice'
import type { ChapterMeta } from '@/lib/handbook/types'

export const Route = createFileRoute('/help/')({
  component: HandbookOverview,
})

function topics(chapter: ChapterMeta): string {
  return chapter.sections
    .filter((section) => section.level === 2)
    .map((section) => section.title)
    .join(' · ')
}

function HandbookOverview() {
  const { t } = useTranslation('help')
  const isVisible = useChapterVisibility()
  const opening = firstChapter(isVisible)
  const parts = visibleParts(handbookIndex.parts, isVisible)
  const chapterCount = parts.reduce((sum, part) => sum + part.chapters.length, 0)

  return (
    <div className="container mt-6 mb-16">
      <article className="mb-8 2xl:w-4/5">
        {/* min-w on the text keeps the download beside the heading while the row
            has room for both, and drops it underneath once it does not. */}
        <div className="flex flex-wrap items-center justify-between gap-x-8 gap-y-4">
          <div className="flex-1 min-w-[20rem]">
            <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
              {t('page.title')}
            </h1>
            <p>{t('page.description')}</p>
            <p className="mt-2 text-sm text-dark-500">
              {t('page.scale', { chapters: chapterCount, parts: parts.length })}
            </p>
          </div>
          <Button asChild variant="outline" className="shrink-0">
            <a href="/handbook/green-ecolution-handbuch.pdf" download>
              <Download className="size-5" />
              {t('page.pdfDownload')}
            </a>
          </Button>
        </div>
      </article>

      <LanguageFallbackNotice className="mb-8 2xl:w-4/5" />

      <HandbookSearch />

      <Link
        to="/help/$slug"
        params={{ slug: opening.slug }}
        className="group mt-10 flex items-center justify-between gap-4 rounded-2xl bg-green-light-50 p-5 transition-colors duration-base ease-emphasized hover:bg-green-light-100"
      >
        <span>
          <span className="block text-xs font-semibold uppercase tracking-wider text-dark-500">
            {t('page.startReading')}
          </span>
          <span className="mt-1 block font-lato font-bold text-lg text-green-dark">
            {opening.title}
          </span>
        </span>
        <MoveRight className="icon-arrow-animate size-5 shrink-0 text-green-dark" />
      </Link>

      <div className="mt-10 space-y-10">
        {parts.map((part) => (
          <section key={part.id}>
            <h2 className="font-lato font-bold text-xl border-b border-dark-100 pb-2 mb-4">
              {part.title}
            </h2>
            <ul className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
              {part.chapters.map((slug) => {
                const chapter = handbookIndex.chapters[slug]
                const inside = topics(chapter)
                return (
                  <li key={slug}>
                    <Link
                      to="/help/$slug"
                      params={{ slug }}
                      className="flex h-full flex-col rounded-2xl border border-dark-100 p-4 shadow-cards transition-colors duration-base ease-out hover:border-green-dark"
                    >
                      <span className="font-lato font-bold block text-green-dark">
                        {chapter.title}
                      </span>
                      <span className="mt-1 block text-sm text-dark-600">{chapter.summary}</span>
                      {inside && (
                        <span className="mt-3 block text-xs text-dark-500 line-clamp-2">
                          {inside}
                        </span>
                      )}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </section>
        ))}
      </div>
    </div>
  )
}
