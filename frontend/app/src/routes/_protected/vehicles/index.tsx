import VehicleCard from '@/components/general/cards/VehicleCard'
import { Loading } from '@green-ecolution/ui'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Plus } from 'lucide-react'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import Pagination from '@/components/general/Pagination'
import { z } from 'zod'
import { vehicleQueries } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const Route = createFileRoute('/_protected/vehicles/')({
  component: Vehicles,
  pendingComponent: pendingLoading('Fahrzeuge wird geladen …'),
  validateSearch: z.object({
    page: z.number().int().min(1).catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    prefetch(queryClient, vehicleQueries.list({ page, perPage: 5 }), 'vehicleQuery')
  },
})

function Vehicles() {
  const { page } = Route.useSearch()
  const {
    data: vehicleRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...vehicleQueries.list({ page, perPage: 5 }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  return (
    <div className="container mt-6">
      <ListPageHeader
        title="Alle Fahrzeuge"
        description="Hier finden Sie eine Übersicht aller Fahrzeuge, welche für Einsätze verwendet werden können."
        action={
          <Can permission={['vehicle:create']}>
            <ButtonLink
              icon={Plus}
              label="Neues Fahrzeug erstellen"
              link={{ to: '/vehicles/new' }}
            />
          </Can>
        }
      />

      <section className="mt-10">
        <ListCardHeader columns="repeat(5, 1fr)">
          <p>Status</p>
          <p>Kennzeichen</p>
          <p>Wasserkapazität</p>
          <p>Modell</p>
          <p>Führerscheinklasse</p>
        </ListCardHeader>
        {!vehicleRes ? (
          <Loading className="mt-10 justify-center" label="Fahrzeuge wird geladen …" />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <EntityList
              items={vehicleRes.data}
              getKey={(vehicle) => vehicle.id}
              emptyMessage="Es wurden leider keine Fahrzeuge gefunden."
              renderItem={(vehicle) => <VehicleCard vehicle={vehicle} />}
            />
            {vehicleRes.pagination && vehicleRes.pagination?.totalPages > 1 && (
              <Pagination pagination={vehicleRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}
