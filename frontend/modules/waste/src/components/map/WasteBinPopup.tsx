import type { ReactNode } from 'react'
import { X } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import { isValidUuid } from '@/api/queries'
import { wasteBinQueries } from '../../api/queries'
import FillStateBadge from '../FillStateBadge'

interface PopupFrameProps {
  label: string
  title: string
  onClose: () => void
  children: ReactNode
}

const PopupFrame = ({ label, title, onClose, children }: PopupFrameProps) => {
  const { t } = useTranslation('waste', { keyPrefix: 'map.popup' })
  return (
    <section
      aria-label={label}
      className="absolute bottom-[calc(1.5rem_+_var(--map-bottom-inset,0px))] left-1/2 z-10 flex w-72 -translate-x-1/2 flex-col gap-3 rounded-xl bg-white p-4 shadow-cards"
    >
      <div className="flex items-start justify-between gap-2">
        <h2 className="text-lg font-semibold">{title}</h2>
        <button
          type="button"
          onClick={onClose}
          aria-label={t('close')}
          className="rounded p-1 text-dark-600 hover:bg-dark-50"
        >
          <X aria-hidden className="size-4" />
        </button>
      </div>
      {children}
    </section>
  )
}

interface WasteBinPopupViewProps {
  bin: WasteBinResponse
  onClose: () => void
}

export const WasteBinPopupView = ({ bin, onClose }: WasteBinPopupViewProps) => {
  const { t } = useTranslation('waste', { keyPrefix: 'map.popup' })
  return (
    <PopupFrame
      label={t('ariaLabel', { number: bin.number })}
      title={t('title', { number: bin.number })}
      onClose={onClose}
    >
      <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
      <Link
        to="/waste-bins/$wasteBinId"
        params={{ wasteBinId: bin.id }}
        className="text-sm font-medium text-green-dark underline-offset-4 hover:underline"
      >
        {t('detailLink')}
      </Link>
    </PopupFrame>
  )
}

interface WasteBinPopupProps {
  binId: string
  onClose: () => void
}

// A query error must stay inside the popup: thrown, it would take down the whole map route.
const WasteBinPopup = ({ binId, onClose }: WasteBinPopupProps) => {
  const { t } = useTranslation('waste', { keyPrefix: 'map.popup' })
  const { data: bin, isError } = useQuery(wasteBinQueries.detail(binId))

  if (bin) return <WasteBinPopupView bin={bin} onClose={onClose} />
  if (!isValidUuid(binId) || isError) {
    return (
      <PopupFrame label={t('notFound')} title={t('notFound')} onClose={onClose}>
        <p className="text-sm text-dark-600">{t('notFoundHint')}</p>
      </PopupFrame>
    )
  }
  return null
}

export default WasteBinPopup
