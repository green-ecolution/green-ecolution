import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { MoveLeft, MoveRight } from 'lucide-react'
import { chapterNeighbours } from '@/lib/handbook'
import { useChapterVisibility } from '@/lib/handbook/useChapterVisibility'
import type { ChapterStep } from '@/lib/handbook/types'

interface StepProps {
  step: ChapterStep
  label: string
  direction: 'previous' | 'next'
  className?: string
}

function Step({ step, label, direction, className = '' }: StepProps) {
  const forward = direction === 'next'

  return (
    <Link
      to="/help/$slug"
      params={{ slug: step.slug }}
      className={`group block px-3 py-5 transition-colors duration-base ease-emphasized hover:bg-green-light-50 ${forward ? 'sm:text-right' : ''} ${className}`}
    >
      <span
        className={`flex items-center gap-1.5 text-xs font-semibold uppercase tracking-wider text-dark-500 ${forward ? 'sm:flex-row-reverse' : ''}`}
      >
        {forward ? (
          <MoveRight className="icon-arrow-animate size-4" />
        ) : (
          <MoveLeft className="icon-arrow-back-animate size-4" />
        )}
        {label}
      </span>
      {step.entersNewPart && (
        <span className="mt-1.5 block text-xs text-dark-500">{step.partTitle}</span>
      )}
      <span className="mt-1 block font-lato font-bold text-lg text-green-dark">{step.title}</span>
    </Link>
  )
}

function ChapterPager({ slug }: { slug: string }) {
  const { t } = useTranslation('help')
  const isVisible = useChapterVisibility()
  const { previous, next } = chapterNeighbours(slug, isVisible)

  if (!previous && !next) return null

  return (
    <nav
      aria-label={t('chapter.chapterNavigation')}
      className="mt-14 -mx-3 border-t border-dark-100"
    >
      <div className="grid divide-y divide-dark-100 sm:grid-cols-2 sm:divide-x sm:divide-y-0">
        {previous && <Step step={previous} label={t('chapter.previous')} direction="previous" />}
        {next && (
          <Step
            step={next}
            label={t('chapter.next')}
            direction="next"
            className={previous ? '' : 'sm:col-start-2'}
          />
        )}
      </div>
    </nav>
  )
}

export default ChapterPager
