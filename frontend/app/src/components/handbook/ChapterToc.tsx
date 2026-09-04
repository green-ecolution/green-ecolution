import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useActiveSection } from '@/hooks/useActiveSection'
import type { ChapterSection } from '@/lib/handbook/types'

function ChapterToc({ sections }: { sections: ChapterSection[] }) {
  const { t } = useTranslation('help')
  const active = useActiveSection(sections.map((section) => section.anchor))

  // Deliberately the prototype method rather than window.history.replaceState:
  // TanStack Router overwrites the instance method and reports every call to its
  // subscribers as a navigation, so with `scrollRestoration` on the page jumps to
  // a restored offset mid-scroll. Going through the prototype reaches the original
  // implementation, and the address bar is all this has to change. The router
  // resyncs from window.location on the next popstate anyway.
  //
  // The hash is only ever set, never cleared. On mount the reader is above the
  // first heading, and clearing it there would drop the anchor of a cross-chapter
  // link before the browser has scrolled to it.
  useEffect(() => {
    if (active === null) return
    const url = new URL(window.location.href)
    if (url.hash === `#${active}`) return
    url.hash = `#${active}`
    History.prototype.replaceState.call(window.history, window.history.state, '', url)
  }, [active])

  if (sections.length === 0) return null

  return (
    <nav className="text-sm">
      <p className="font-lato font-semibold mb-2">{t('chapter.onThisPage')}</p>
      <ul className="space-y-1 border-l border-dark-100">
        {sections.map((section) => (
          <li key={section.anchor} className={section.level === 3 ? 'pl-4' : ''}>
            <a
              href={`#${section.anchor}`}
              aria-current={section.anchor === active ? 'location' : undefined}
              className={`block border-l-2 -ml-px pl-3 py-0.5 transition-colors duration-quick ease-out hover:border-green-dark hover:text-dark ${
                section.anchor === active
                  ? 'border-green-dark text-dark font-medium'
                  : 'border-transparent text-dark-600'
              }`}
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
