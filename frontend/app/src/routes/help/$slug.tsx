import { createFileRoute, Link, notFound } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { ArrowLeft } from 'lucide-react'
import { handbookIndex, loadChapter } from '@/lib/handbook'
import Blocks from '@/components/handbook/Blocks'
import ChapterToc from '@/components/handbook/ChapterToc'
import LanguageFallbackNotice from '@/components/handbook/LanguageFallbackNotice'

export const Route = createFileRoute('/help/$slug')({
  component: HandbookChapter,
  loader: async ({ params }) => {
    const meta = handbookIndex.chapters[params.slug]
    if (!meta) throw notFound()
    return { meta, content: await loadChapter(params.slug) }
  },
})

function HandbookChapter() {
  const { meta, content } = Route.useLoaderData()
  const { t } = useTranslation('help')

  return (
    <div className="container mt-6 mb-16 lg:grid lg:grid-cols-[minmax(0,1fr)_16rem] lg:gap-12">
      <article>
        <Link
          to="/help"
          className="inline-flex items-center gap-2 text-sm text-dark-600 transition-colors duration-quick ease-out hover:text-dark"
        >
          <ArrowLeft className="size-4" />
          {t('chapter.backToOverview')}
        </Link>
        <h1 className="font-lato font-bold text-3xl mt-4 lg:text-4xl">{meta.title}</h1>
        <p className="mt-2 text-dark-600">{meta.summary}</p>
        <LanguageFallbackNotice className="mt-6" />
        <Blocks blocks={content.blocks} />
      </article>

      <aside className="mt-10 lg:mt-24 lg:sticky lg:top-24 lg:self-start">
        <ChapterToc sections={meta.sections} />
      </aside>
    </div>
  )
}
