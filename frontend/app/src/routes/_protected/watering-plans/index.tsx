import { wateringPlanQuery } from '@/api/queries'
import WateringPlanCard from '@/components/general/cards/WateringPlanCard'
import { Loading } from '@green-ecolution/ui'
import ButtonLink from '@/components/general/links/ButtonLink'
import Pagination from '@/components/general/Pagination'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, useLoaderData } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { z } from 'zod'
import { ListCardHeader } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/watering-plans/')({
  component: WateringPlans,
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Daten werden geladen" />,
  validateSearch: z.object({
    page: z.number().default(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page: page || 1,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    queryClient
      .prefetchQuery(wateringPlanQuery({ page: page, limit: 5 }))
      .catch((error) => console.error('Prefetching "wateringPlanQuery" failed', error))
    return { page }
  },
})

function WateringPlans() {
  const search = useLoaderData({ from: '/_protected/watering-plans/' })
  const { data: wateringPlanRes } = useSuspenseQuery(
    wateringPlanQuery({ page: search.page, limit: 5 }),
  )

  return (
    <div className="container mt-6">
      <article className="mb-20 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Alle Einsatzpläne
        </h1>
        <p className="mb-5">
          Hier finden Sie eine Übersicht aller Einsatzpläne. Ein Einsatzplan beschreibt eine
          Bewässerungsfahrt mehrerer Bewässerungsgruppen. Die Bewässerungsfahrten können dadurch
          dynamisch und schnell geplant
        </p>
        <ButtonLink
          icon={Plus}
          label="Neuen Einsatzplan erstellen"
          link={{ to: '/watering-plans/new' }}
        />
      </article>

      <section className="mt-10">
        <ListCardHeader columns="1.3fr 1.5fr 1fr 1.5fr 1.5fr">
          <p>Status</p>
          <p>Datum & Fahrzeug</p>
          <p>Länge</p>
          <p>Mitarbeitenden</p>
          <p>Bewässerungsgruppen</p>
        </ListCardHeader>
        <ul>
          {wateringPlanRes.data?.length === 0 ? (
            <li className="text-center text-dark-600 mt-10">
              <p>Es wurden leider keine Einsatzpläne gefunden.</p>
            </li>
          ) : (
            wateringPlanRes.data?.map((wateringPlan) => (
              <li key={wateringPlan.id} className="mb-5 last:mb-0">
                <WateringPlanCard wateringPlan={wateringPlan} />
              </li>
            ))
          )}
        </ul>
        {wateringPlanRes.pagination && wateringPlanRes.pagination?.totalPages > 1 && (
          <Pagination pagination={wateringPlanRes.pagination} />
        )}
      </section>
    </div>
  )
}
