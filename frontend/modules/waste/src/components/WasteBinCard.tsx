import { Link } from '@tanstack/react-router'
import { Cpu, Package } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { ListCard } from '@green-ecolution/ui'
import { highlightMatch } from '@/lib/highlightMatch'
import type { WasteBin } from '../api/client'
import FillStateBadge from './FillStateBadge'

interface WasteBinCardProps {
  bin: WasteBin
  query?: string
}

const WasteBinCard = ({ bin, query = '' }: WasteBinCardProps) => {
  const { t } = useTranslation('waste')

  return (
    <ListCard asChild className="@container">
      <Link to="/waste-bins/$wasteBinId" params={{ wasteBinId: bin.id }}>
        <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
          <span className="font-lato text-lg font-bold tabular-nums">
            <span className="sr-only">{t('card.numberSrLabel')}</span>
            <span>{highlightMatch(bin.number, query)}</span>
          </span>
          <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
        </div>

        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm text-dark-600">
          <span className="flex items-center gap-2">
            <Package aria-hidden className="size-4" />
            <span className="sr-only">{t('card.volumeSrLabel')}</span>
            {t('units.liters', { value: bin.volumeLiters })}
          </span>
          <span className="flex min-w-0 items-center gap-2">
            <Cpu aria-hidden className="size-4 shrink-0" />
            <span className="sr-only">{t('card.sensorSrLabel')}</span>
            <span className="truncate">{bin.sensorId ?? t('card.noSensor')}</span>
          </span>
        </div>
      </Link>
    </ListCard>
  )
}

export default WasteBinCard
