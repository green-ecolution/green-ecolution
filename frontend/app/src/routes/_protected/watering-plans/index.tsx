import { wateringPlanQuery } from '@/api/queries'
import WateringPlanCard from '@/components/general/cards/WateringPlanCard'
import { Loading } from '@green-ecolution/ui'
import ButtonLink from '@/components/general/links/ButtonLink'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import Pagination from '@/components/general/Pagination'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { z } from 'zod'
import { ListCardHeader } from '@green-ecolution/ui'
import { pendingLoading, prefetch } from '@/lib/router'

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  pendingComponent: pendingLoading('Daten werden geladen'),
  validateSearch: z.object({
    page: z.number().int().min(1).catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    prefetch(queryClient, wateringPlanQuery({ page, perPage: 5 }), 'wateringPlanQuery')
  },
})

function WateringPlans() {
  const { page } = Route.useSearch()
  const {
    data: wateringPlanRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...wateringPlanQuery({ page, perPage: 5 }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  return (
    <div className="container mt-6">
      <ListPageHeader
        title="Alle Einsatzpläne"
        description="Hier finden Sie eine Übersicht aller Einsatzpläne. Ein Einsatzplan beschreibt eine Bewässerungsfahrt mehrerer Bewässerungsgruppen. Die Bewässerungsfahrten können dadurch dynamisch und schnell geplant"
        action={
          <ButtonLink
            icon={Plus}
            label="Neuen Einsatzplan erstellen"
            link={{ to: '/watering-plans/new' }}
          />
        }
      />

      <section className="mt-10">
        <ListCardHeader columns="1.3fr 1.5fr 1fr 1.5fr 1.5fr">
          <p>Status</p>
          <p>Datum & Fahrzeug</p>
          <p>Länge</p>
          <p>Mitarbeitenden</p>
          <p>Bewässerungsgruppen</p>
        </ListCardHeader>
        {!wateringPlanRes ? (
          <Loading className="mt-10 justify-center" label="Daten werden geladen" />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <EntityList
              items={wateringPlanRes.data}
              getKey={(wateringPlan) => wateringPlan.id}
              emptyMessage="Es wurden leider keine Einsatzpläne gefunden."
              renderItem={(wateringPlan) => <WateringPlanCard wateringPlan={wateringPlan} />}
            />
            {wateringPlanRes.pagination && wateringPlanRes.pagination?.totalPages > 1 && (
              <Pagination pagination={wateringPlanRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}
