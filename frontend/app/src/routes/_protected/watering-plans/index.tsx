import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { suggestedClustersQuery, userQuery, wateringPlanBoardColumnQuery } from '@/api/queries'
import ButtonLink from '@/components/general/links/ButtonLink'
import ListPageHeader from '@/components/general/ListPageHeader'
import WateringPlanBoard from '@/components/watering-plan/board/WateringPlanBoard'
import { pendingLoading, prefetch } from '@/lib/router'

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  pendingComponent: pendingLoading('Daten werden geladen'),
  loader: ({ context: { queryClient } }) => {
    prefetch(
      queryClient,
      wateringPlanBoardColumnQuery([WateringPlanStatus.Planned]),
      'wateringPlanBoardColumnQuery(planned)',
    )
    prefetch(
      queryClient,
      wateringPlanBoardColumnQuery([WateringPlanStatus.Active]),
      'wateringPlanBoardColumnQuery(active)',
    )
    prefetch(queryClient, suggestedClustersQuery(), 'suggestedClustersQuery')
    prefetch(queryClient, userQuery({ page: 1, perPage: 100 }), 'userQuery')
  },
})

function WateringPlans() {
  return (
    <div className="mt-6">
      <div className="container">
        <ListPageHeader
          title="Einsatzpläne"
          description="Planen, starten und dokumentieren Sie Bewässerungsfahrten. Ziehen Sie einen Einsatz in die nächste Spalte, um seinen Status zu ändern."
          action={
            <ButtonLink
              icon={Plus}
              label="Neuen Einsatzplan erstellen"
              link={{ to: '/watering-plans/new' }}
            />
          }
        />
      </div>
      <section className="px-4 lg:px-10">
        <WateringPlanBoard />
      </section>
    </div>
  )
}
