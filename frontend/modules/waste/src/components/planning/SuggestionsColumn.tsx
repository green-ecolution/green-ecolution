import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { Link, useNavigate } from '@tanstack/react-router'
import { FolderPlus, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Checkbox,
  KanbanCard,
  KanbanColumn,
  KanbanColumnEmpty,
  KanbanColumnHeader,
} from '@green-ecolution/ui'
import { PlanStatus } from '@green-ecolution/backend-client'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { wasteBinQueries } from '../../api/queries'
import type { WasteBin } from '../../api/client'
import { WASTE } from '../../lib/targets'
import FillStateBadge from '../FillStateBadge'

const SuggestionsColumn = () => {
  const binsQuery = useQuery(wasteBinQueries.full())
  const [selected, setSelected] = useState<string[]>([])
  const draft = useWateringPlanDraft<WateringPlanForm>('create')
  const navigate = useNavigate()
  const canBundle = useHasPermission(['collection_plan:create'])
  const { t } = useTranslation(['waste', 'common'])

  const bins = binsQuery.data?.data ?? []

  const toggle = (binId: string, isSelected: boolean) =>
    setSelected((prev) => (isSelected ? [...prev, binId] : prev.filter((id) => id !== binId)))

  const bundleIntoPlan = () => {
    draft.setData({
      module: WASTE,
      date: new Date(),
      description: '',
      transporterId: '',
      trailerId: undefined,
      transporterTankUsed: true,
      stopIds: selected,
      status: PlanStatus.Planned,
      driverIds: [],
      startPointName: '',
    })
    navigate({ to: '/plans/new', search: { module: WASTE } }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <KanbanColumn tone="suggestion" aria-label={t('planning.suggestions.ariaLabel')}>
      <KanbanColumnHeader
        icon={<Trash2 />}
        title={t('planning.suggestions.title')}
        count={bins.length}
      />
      {binsQuery.isError && (
        <KanbanColumnEmpty>
          {t('planning.suggestions.loadError')}
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="mt-2 bg-white"
            onClick={() => void binsQuery.refetch()}
          >
            {t('common:actions.retry')}
          </Button>
        </KanbanColumnEmpty>
      )}
      {binsQuery.isSuccess && bins.length === 0 && (
        <KanbanColumnEmpty>{t('planning.suggestions.emptyMessage')}</KanbanColumnEmpty>
      )}
      {bins.map((bin) => (
        <BinSuggestionCard
          key={bin.id}
          bin={bin}
          selected={selected.includes(bin.id)}
          onSelectedChange={canBundle ? (isSelected) => toggle(bin.id, isSelected) : undefined}
        />
      ))}
      {bins.length > 0 && canBundle && (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={selected.length === 0}
          onClick={bundleIntoPlan}
          className="bg-white"
        >
          <FolderPlus className="size-4" />
          {t('planning.suggestions.bundleButtonLabel')}
          {selected.length > 0 && <span className="tabular-nums">({selected.length})</span>}
        </Button>
      )}
    </KanbanColumn>
  )
}

interface BinSuggestionCardProps {
  bin: WasteBin
  selected: boolean
  /** Omit to render the card read-only. */
  onSelectedChange?: (selected: boolean) => void
}

const BinSuggestionCard = ({ bin, selected, onSelectedChange }: BinSuggestionCardProps) => {
  const { t } = useTranslation('waste')

  const toggleOnCardClick = (event: React.MouseEvent) => {
    // The link keeps navigating and the checkbox toggles itself; don't toggle twice.
    if ((event.target as HTMLElement).closest('a, [role="checkbox"]')) return
    onSelectedChange?.(!selected)
  }

  return (
    <KanbanCard
      className={`flex items-start gap-3 ${onSelectedChange ? 'cursor-pointer' : ''}`}
      onClick={onSelectedChange ? toggleOnCardClick : undefined}
    >
      {onSelectedChange && (
        <Checkbox
          checked={selected}
          onCheckedChange={(checked) => onSelectedChange(checked === true)}
          aria-label={t('planning.suggestionCard.selectAriaLabel', { number: bin.number })}
          className="mt-0.5"
        />
      )}
      <div className="min-w-0">
        <Link
          to="/waste-bins/$wasteBinId"
          params={{ wasteBinId: bin.id }}
          className="font-lato font-semibold tabular-nums text-dark hover:underline"
        >
          {bin.number}
        </Link>
        <div className="mt-1 flex flex-wrap items-center gap-2 text-sm text-dark-600">
          <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
          <span className="tabular-nums">{t('units.liters', { value: bin.volumeLiters })}</span>
        </div>
      </div>
    </KanbanCard>
  )
}

export default SuggestionsColumn
