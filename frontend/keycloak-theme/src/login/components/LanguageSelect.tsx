import { useEffect, useRef, useState } from 'react'
import type { I18n } from '../i18n'

export default function LanguageSelect(props: { i18n: I18n }) {
  const { currentLanguage, enabledLanguages, msgStr } = props.i18n
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (!isOpen) {
      return
    }

    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        setIsOpen(false)
      }
    }

    const closeOnOutsideClick = (event: PointerEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        setIsOpen(false)
      }
    }

    document.addEventListener('keydown', closeOnEscape)
    document.addEventListener('pointerdown', closeOnOutsideClick)

    return () => {
      document.removeEventListener('keydown', closeOnEscape)
      document.removeEventListener('pointerdown', closeOnOutsideClick)
    }
  }, [isOpen])

  if (enabledLanguages.length <= 1) {
    return null
  }

  return (
    <div ref={containerRef} className="relative">
      <button
        type="button"
        id="kc-current-locale-link"
        aria-label={msgStr('languages')}
        aria-expanded={isOpen}
        aria-controls="language-switch"
        onClick={() => setIsOpen(open => !open)}
        className="rounded-lg px-3 py-1.5 text-sm font-semibold text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/40"
      >
        {currentLanguage.label}
      </button>
      <ul
        id="language-switch"
        hidden={!isOpen}
        aria-labelledby="kc-current-locale-link"
        className="absolute right-0 z-20 mt-1 min-w-32 list-none rounded-lg border border-border bg-card p-1 shadow-lg"
      >
        {enabledLanguages.map(({ languageTag, label, href }) => (
          <li key={languageTag}>
            <a
              href={href}
              className="block rounded-md px-3 py-1.5 text-sm text-card-foreground no-underline transition-colors hover:bg-accent"
            >
              {label}
            </a>
          </li>
        ))}
      </ul>
    </div>
  )
}
