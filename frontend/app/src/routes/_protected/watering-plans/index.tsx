import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { clusterQueries, userQueries, wateringPlanQueries } from '@/api/queries'
import ButtonLink from '@/components/general/links/ButtonLink'
import ListPageHeader from '@/components/general/ListPageHeader'
import WateringPlanBoard from '@/components/watering-plan/board/WateringPlanBoard'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  pendingComponent: pendingLoading('Daten werden geladen'),
  loader: ({ context: { queryClient } }) => {
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn([WateringPlanStatus.Planned]),
      'wateringPlanQueries.boardColumn(planned)',
    )
    prefetch(
      queryClient,
      wateringPlanQueries.boardColumn([WateringPlanStatus.Active]),
      'wateringPlanQueries.boardColumn(active)',
    )
    prefetch(queryClient, clusterQueries.suggested(), 'clusterQueries.suggested')
    prefetch(queryClient, userQueries.list({ page: 1, perPage: 100 }), 'userQueries.list')
  },
})

function WateringPlans() {
  const canModify = useHasPermission(['watering_plan:update'])

  return (
    <div className="mt-6">
      <div className="container">
        <ListPageHeader
          title="Einsatzpläne"
          description={
            canModify
              ? 'Planen, starten und dokumentieren Sie Bewässerungsfahrten. Ziehen Sie einen Einsatz in die nächste Spalte, um seinen Status zu ändern.'
              : 'Verfolgen Sie geplante, laufende und abgeschlossene Bewässerungsfahrten.'
          }
          action={
            <Can permission={['watering_plan:create']}>
              <ButtonLink
                icon={Plus}
                label="Neuen Einsatzplan erstellen"
                link={{ to: '/watering-plans/new' }}
              />
            </Can>
          }
        />
      </div>
      <section className="px-4 lg:px-10">
        <WateringPlanBoard />
      </section>
    </div>
  )
}
