import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Download } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import { handbookIndex } from '@/lib/handbook'
import HandbookSearch from '@/components/handbook/HandbookSearch'

export const Route = createFileRoute('/help/')({
  component: HandbookOverview,
})

function HandbookOverview() {
  const { t } = useTranslation('help')

  return (
    <div className="container mt-6 mb-16">
      <article className="mb-8 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('page.title')}
        </h1>
        <p className="mb-6">{t('page.description')}</p>
        <Button asChild variant="outline">
          <a href="/handbook/green-ecolution-handbuch.pdf" download>
            <Download className="size-5" />
            {t('page.pdfDownload')}
          </a>
        </Button>
      </article>

      <HandbookSearch />

      <div className="mt-10 space-y-10">
        {handbookIndex.parts.map((part) => (
          <section key={part.id}>
            <h2 className="font-lato font-bold text-xl mb-4">{part.title}</h2>
            <ul className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
              {part.chapters.map((slug) => {
                const chapter = handbookIndex.chapters[slug]
                return (
                  <li key={slug}>
                    <Link
                      // @ts-expect-error -- /help/$slug is registered by a later task in this plan; the route does not exist yet
                      to="/help/$slug"
                      // @ts-expect-error -- see above, follows from the same not-yet-registered route
                      params={{ slug }}
                      className="block h-full rounded-2xl border border-dark-100 p-4 shadow-cards transition-colors duration-base ease-out hover:border-green-dark"
                    >
                      <span className="font-lato font-semibold block">{chapter.title}</span>
                      <span className="mt-1 block text-sm text-dark-600">{chapter.summary}</span>
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
