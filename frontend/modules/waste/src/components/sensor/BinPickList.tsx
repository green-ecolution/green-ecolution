import { useState } from 'react'
import { Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  Button,
  Input,
  Loading,
  cn,
} from '@green-ecolution/ui'
import type { WasteBinResponse } from '@green-ecolution/backend-client'
import FillStateBadge from '../FillStateBadge'

interface BinPickListProps {
  bins: WasteBinResponse[]
  loading: boolean
  failed: boolean
  onRetry: () => void
  selectedId: string | null
  /** A bin carrying this sensor stays selectable (reassign keeps its current bin in the list). */
  sensorId?: string
  onSelect: (bin: WasteBinResponse) => void
  initialQuery?: string
}

/** Searches the bins already loaded; an organization's bins fit one page of the marker query. */
const BinPickList = ({
  bins,
  loading,
  failed,
  onRetry,
  selectedId,
  sensorId,
  onSelect,
  initialQuery = '',
}: BinPickListProps) => {
  const { t } = useTranslation('waste')
  const [query, setQuery] = useState(initialQuery)
  const needle = query.trim().toLowerCase()
  const visible = needle ? bins.filter((bin) => bin.number.toLowerCase().includes(needle)) : bins

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3">
      <div className="relative flex items-center">
        <Search className="pointer-events-none absolute left-3 size-4 text-dark-500" aria-hidden />
        <Input
          role="searchbox"
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder={t('sensor.binSearch.placeholder')}
          aria-label={t('sensor.binSearch.label')}
          className="pl-9 text-base"
          autoComplete="off"
          inputMode="search"
        />
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto">
        {loading && <Loading size="default" label={t('sensor.binSearch.loading')} />}
        {failed && (
          <Alert variant="destructive">
            <AlertContent>
              <AlertDescription>{t('sensor.binSearch.failed')}</AlertDescription>
            </AlertContent>
            <Button variant="outline" size="sm" onClick={onRetry}>
              {t('sensor.binSearch.retry')}
            </Button>
          </Alert>
        )}
        {!loading && !failed && visible.length === 0 && (
          <p className="py-8 text-center text-sm text-dark-600">{t('sensor.binSearch.empty')}</p>
        )}
        <ul className="flex flex-col gap-2">
          {visible.map((bin) => {
            const taken = bin.sensorId != null && bin.sensorId !== sensorId
            const selected = bin.id === selectedId
            return (
              <li key={bin.id}>
                <button
                  type="button"
                  disabled={taken}
                  aria-pressed={selected}
                  onClick={() => onSelect(bin)}
                  className={cn(
                    'flex w-full items-center justify-between gap-3 rounded-xl border border-dark-100 px-4 py-3 text-left transition-colors',
                    'hover:border-green-dark/40 focus-visible:outline-2 focus-visible:outline-green-dark',
                    selected && 'border-green-dark bg-green-dark-50/40',
                    taken && 'cursor-not-allowed opacity-50',
                  )}
                >
                  <span className="flex flex-col">
                    <span className="font-lato font-bold tabular-nums">{bin.number}</span>
                    <span className="text-xs text-dark-600">
                      {taken
                        ? t('sensor.binSearch.taken')
                        : t('units.liters', { value: bin.volumeLiters })}
                    </span>
                  </span>
                  <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
                </button>
              </li>
            )
          })}
        </ul>
      </div>
    </div>
  )
}

export default BinPickList
