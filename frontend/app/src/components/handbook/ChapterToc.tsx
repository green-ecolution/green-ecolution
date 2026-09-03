import { useTranslation } from 'react-i18next'
import type { ChapterSection } from '@/lib/handbook/types'

function ChapterToc({ sections }: { sections: ChapterSection[] }) {
  const { t } = useTranslation('help')

  if (sections.length === 0) return null

  return (
    <nav aria-label={t('chapter.onThisPage')} className="text-sm">
      <p className="font-lato font-semibold mb-2">{t('chapter.onThisPage')}</p>
      <ul className="space-y-1 border-l border-dark-100">
        {sections.map((section) => (
          <li key={section.anchor} className={section.level === 3 ? 'pl-4' : ''}>
            <a
              href={`#${section.anchor}`}
              className="block border-l-2 border-transparent -ml-px pl-3 py-0.5 text-dark-600 transition-colors duration-quick ease-out hover:border-green-dark hover:text-dark"
            >
              {section.title}
            </a>
          </li>
        ))}
      </ul>
    </nav>
  )
}

export default ChapterToc
