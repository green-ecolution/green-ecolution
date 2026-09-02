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
import { useTranslation } from 'react-i18next'
import { vehicleQueries } from '@/api/queries'
import { ListCardHeader } from '@green-ecolution/ui'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const Route = createFileRoute('/_protected/vehicles/')({
  component: Vehicles,
  pendingComponent: pendingLoading({ key: 'vehicle:list.loadingLabel' }),
  validateSearch: z.object({
    page: z.number().int().min(1).catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    prefetch(queryClient, vehicleQueries.list({ page, perPage: 5 }), 'vehicleQueries.list')
  },
})

function Vehicles() {
  const { t } = useTranslation('vehicle')
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
        title={t('list.title')}
        description={t('list.description')}
        action={
          <Can permission={['vehicle:create']}>
            <ButtonLink icon={Plus} label={t('list.createButton')} link={{ to: '/vehicles/new' }} />
          </Can>
        }
      />

      <section className="mt-10">
        <ListCardHeader columns="repeat(5, 1fr)">
          <p>{t('list.columnStatus')}</p>
          <p>{t('list.columnNumberPlate')}</p>
          <p>{t('list.columnWaterCapacity')}</p>
          <p>{t('list.columnModel')}</p>
          <p>{t('list.columnDrivingLicense')}</p>
        </ListCardHeader>
        {!vehicleRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <EntityList
              items={vehicleRes.data}
              getKey={(vehicle) => vehicle.id}
              emptyMessage={t('list.emptyMessage')}
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
