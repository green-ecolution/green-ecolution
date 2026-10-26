import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { FolderPlus, Sprout } from 'lucide-react'
import { Button, KanbanColumn, KanbanColumnEmpty, KanbanColumnHeader } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { IRRIGATION } from '../../lib/targets'
import { clusterQueries } from '../../api/queries'
import ClusterSuggestionCard from './ClusterSuggestionCard'

const SuggestionsColumn = () => {
  const clustersQuery = useQuery(clusterQueries.suggested())
  const { data: clustersRes } = clustersQuery
  const [selected, setSelected] = useState<string[]>([])
  const draft = useWateringPlanDraft<WateringPlanForm>('create')
  const navigate = useNavigate()
  const canBundle = useHasPermission(['watering_plan:create'])
  const { t } = useTranslation(['irrigation', 'common'])

  const clusters = clustersRes?.data ?? []

  const toggle = (clusterId: string, isSelected: boolean) => {
    setSelected((prev) =>
      isSelected ? [...prev, clusterId] : prev.filter((id) => id !== clusterId),
    )
  }

  const bundleIntoPlan = () => {
    draft.setData({
      module: IRRIGATION,
      date: new Date(),
      description: '',
      transporterId: '',
      trailerId: undefined,
      transporterTankUsed: true,
      stopIds: selected,
      status: WateringPlanStatus.Planned,
      driverIds: [],
      startPointName: '',
    })
    navigate({ to: '/watering-plans/new' }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <KanbanColumn tone="suggestion" aria-label={t('planning.suggestions.ariaLabel')}>
      <KanbanColumnHeader
        icon={<Sprout />}
        title={t('planning.suggestions.title')}
        count={clusters.length}
      />
      {clustersQuery.isError && (
        <KanbanColumnEmpty>
          {t('planning.suggestions.loadError')}
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="mt-2 bg-white"
            onClick={() => void clustersQuery.refetch()}
          >
            {t('common:actions.retry')}
          </Button>
        </KanbanColumnEmpty>
      )}
      {!clustersQuery.isError && clusters.length === 0 && (
        <KanbanColumnEmpty>{t('planning.suggestions.emptyMessage')}</KanbanColumnEmpty>
      )}
      {clusters.map((cluster) => (
        <ClusterSuggestionCard
          key={cluster.id}
          cluster={cluster}
          selected={selected.includes(cluster.id)}
          onSelectedChange={canBundle ? (isSelected) => toggle(cluster.id, isSelected) : undefined}
        />
      ))}
      {clusters.length > 0 && canBundle && (
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

export default SuggestionsColumn
