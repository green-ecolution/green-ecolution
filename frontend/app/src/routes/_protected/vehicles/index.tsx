import VehicleCard from '@/components/general/cards/VehicleCard'
import { Button, Loading } from '@green-ecolution/ui'
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
import {
  DrivingLicense,
  ListVehiclesArchiveEnum,
  ListVehiclesOrderEnum,
  ListVehiclesSortEnum,
  VehicleStatus,
  VehicleType,
} from '@green-ecolution/backend-client'
import VehicleListToolbar from '@/components/vehicle/list/VehicleListToolbar'
import { useVehicleListSearch } from '@/components/vehicle/list/useVehicleListSearch'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const PER_PAGE = 25

const vehicleFilterSchema = z.object({
  page: z.number().int().min(1).catch(1),
  q: z.string().optional().catch(undefined),
  statuses: z.array(z.enum(VehicleStatus)).optional().catch(undefined),
  types: z.array(z.enum(VehicleType)).optional().catch(undefined),
  drivingLicenses: z.array(z.enum(DrivingLicense)).optional().catch(undefined),
  archive: z.enum(ListVehiclesArchiveEnum).optional().catch(undefined),
  sort: z.enum(ListVehiclesSortEnum).optional().catch(undefined),
  order: z.enum(ListVehiclesOrderEnum).optional().catch(undefined),
})

const listParams = (search: z.infer<typeof vehicleFilterSchema>) => ({
  page: search.page,
  perPage: PER_PAGE,
  q: search.q,
  status: search.statuses,
  type: search.types,
  drivingLicense: search.drivingLicenses,
  archive: search.archive,
  sort: search.sort,
  order: search.order,
})

function Vehicles() {
  const { t } = useTranslation('vehicle')
  const search = Route.useSearch()
  const { resetFilters } = useVehicleListSearch()
  const {
    data: vehicleRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...vehicleQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  const isFiltered =
    (search.q ?? '').length > 0 ||
    (search.statuses?.length ?? 0) > 0 ||
    (search.types?.length ?? 0) > 0 ||
    (search.drivingLicenses?.length ?? 0) > 0 ||
    search.archive !== undefined

  return (
    <div className="container mt-6">
      <ListPageHeader title={t('list.title')} description={t('list.description')} />

      <section className="mt-10">
        <VehicleListToolbar
          filteredRecords={vehicleRes?.pagination?.totalRecords ?? 0}
          totalRecords={vehicleRes?.pagination?.totalUnfiltered ?? undefined}
          action={
            <Can permission={['vehicle:create']}>
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/vehicles/new' }}
              />
            </Can>
          }
        />

        {!vehicleRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="mt-6 transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {vehicleRes.data.length === 0 && isFiltered ? (
              <div className="mt-10 text-center">
                <p className="text-dark-600">{t('list.emptyFilteredMessage')}</p>
                <Button variant="outline" className="mt-4" onClick={resetFilters}>
                  {t('list.emptyFilteredAction')}
                </Button>
              </div>
            ) : (
              <EntityList
                items={vehicleRes.data}
                getKey={(vehicle) => vehicle.id}
                emptyMessage={t('list.emptyMessage')}
                renderItem={(vehicle) => <VehicleCard vehicle={vehicle} query={search.q ?? ''} />}
              />
            )}
            {vehicleRes.pagination && vehicleRes.pagination.totalPages > 1 && (
              <Pagination pagination={vehicleRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}

export const Route = createFileRoute('/_protected/vehicles/')({
  component: Vehicles,
  validateSearch: vehicleFilterSchema,
  pendingComponent: pendingLoading({ key: 'vehicle:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({
    page: search.page,
    q: search.q,
    statuses: search.statuses,
    types: search.types,
    drivingLicenses: search.drivingLicenses,
    archive: search.archive,
    sort: search.sort,
    order: search.order,
  }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, vehicleQueries.list(listParams(deps)), 'vehicleQueries.list')
  },
})
