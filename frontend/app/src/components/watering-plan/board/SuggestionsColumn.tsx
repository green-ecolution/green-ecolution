import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import { FolderPlus, Sprout } from 'lucide-react'
import { Button, KanbanColumn, KanbanColumnEmpty, KanbanColumnHeader } from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { suggestedClustersQuery } from '@/api/queries'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'
import ClusterSuggestionCard from './ClusterSuggestionCard'

const SuggestionsColumn = () => {
  const clustersQuery = useQuery(suggestedClustersQuery())
  const { data: clustersRes } = clustersQuery
  const [selected, setSelected] = useState<string[]>([])
  const draft = useWateringPlanDraft<WateringPlanForm>('create')
  const navigate = useNavigate()

  const clusters = clustersRes?.data ?? []

  const toggle = (clusterId: string, isSelected: boolean) => {
    setSelected((prev) =>
      isSelected ? [...prev, clusterId] : prev.filter((id) => id !== clusterId),
    )
  }

  const bundleIntoPlan = () => {
    draft.setData({
      date: new Date(),
      description: '',
      transporterId: '',
      trailerId: undefined,
      clusterIds: selected,
      status: WateringPlanStatus.Planned,
      driverIds: [],
      startPointName: '',
    })
    navigate({ to: '/watering-plans/new' }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <KanbanColumn tone="suggestion" aria-label="Vorschläge">
      <KanbanColumnHeader icon={<Sprout />} title="Vorschläge" count={clusters.length} />
      {clustersQuery.isError && (
        <KanbanColumnEmpty>
          Die Vorschläge konnten nicht geladen werden.
          <Button
            type="button"
            size="sm"
            variant="outline"
            className="mt-2 bg-white"
            onClick={() => void clustersQuery.refetch()}
          >
            Erneut versuchen
          </Button>
        </KanbanColumnEmpty>
      )}
      {!clustersQuery.isError && clusters.length === 0 && (
        <KanbanColumnEmpty>Aktuell sind keine Bewässerungsgruppen sehr trocken.</KanbanColumnEmpty>
      )}
      {clusters.map((cluster) => (
        <ClusterSuggestionCard
          key={cluster.id}
          cluster={cluster}
          selected={selected.includes(cluster.id)}
          onSelectedChange={(isSelected) => toggle(cluster.id, isSelected)}
        />
      ))}
      {clusters.length > 0 && (
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={selected.length === 0}
          onClick={bundleIntoPlan}
          className="bg-white"
        >
          <FolderPlus className="size-4" />
          Zu Einsatzplan bündeln
          {selected.length > 0 && <span className="tabular-nums">({selected.length})</span>}
        </Button>
      )}
    </KanbanColumn>
  )
}

export default SuggestionsColumn
