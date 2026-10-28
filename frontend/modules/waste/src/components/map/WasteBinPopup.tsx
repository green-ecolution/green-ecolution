import { X } from 'lucide-react'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import { wasteBinQueries } from '../../api/queries'
import FillStateBadge from '../FillStateBadge'

interface WasteBinPopupViewProps {
  bin: WasteBinResponse
  onClose: () => void
}

export const WasteBinPopupView = ({ bin, onClose }: WasteBinPopupViewProps) => {
  const { t } = useTranslation('waste', { keyPrefix: 'map.popup' })
  return (
    <section
      aria-label={t('ariaLabel', { number: bin.number })}
      className="absolute bottom-[calc(1.5rem_+_var(--map-bottom-inset,0px))] left-1/2 z-10 flex w-72 -translate-x-1/2 flex-col gap-3 rounded-xl bg-white p-4 shadow-cards"
    >
      <div className="flex items-start justify-between gap-2">
        <h2 className="text-lg font-semibold">{t('title', { number: bin.number })}</h2>
        <button
          type="button"
          onClick={onClose}
          aria-label={t('close')}
          className="rounded p-1 text-dark-600 hover:bg-dark-50"
        >
          <X aria-hidden className="size-4" />
        </button>
      </div>
      <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
      <Link
        to="/waste-bins/$wasteBinId"
        params={{ wasteBinId: bin.id }}
        className="text-sm font-medium text-green-dark underline-offset-4 hover:underline"
      >
        {t('detailLink')}
      </Link>
    </section>
  )
}

interface WasteBinPopupProps {
  binId: string
  onClose: () => void
}

const WasteBinPopup = ({ binId, onClose }: WasteBinPopupProps) => {
  const { data: bin } = useSuspenseQuery(wasteBinQueries.detail(binId))
  return <WasteBinPopupView bin={bin} onClose={onClose} />
}

export default WasteBinPopup
