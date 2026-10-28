import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { z } from 'zod'
import { Button, Loading } from '@green-ecolution/ui'
import { FillStateDto } from '@green-ecolution/backend-client'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import Pagination from '@/components/general/Pagination'
import ActiveFilterChips from '@/components/general/list/ActiveFilterChips'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListToolbar from '@/components/general/list/ListToolbar'
import ButtonLink from '@/components/general/links/ButtonLink'
import { Can } from '@/lib/auth/Can'
import { pendingLoading, prefetch } from '@/lib/router'
import { wasteBinQueries } from '../../../api/queries'
import WasteBinCard from '../../../components/WasteBinCard'

export const PER_PAGE = 25

const FILL_STATES = Object.values(FillStateDto)

const wasteBinSearchSchema = z.object({
  page: z.number().int().min(1).catch(1),
  q: z.string().optional().catch(undefined),
  fillState: z.enum(FillStateDto).optional().catch(undefined),
})

type WasteBinSearch = z.infer<typeof wasteBinSearchSchema>

const listParams = (search: WasteBinSearch) => ({
  page: search.page,
  perPage: PER_PAGE,
  number: search.q,
  fillState: search.fillState,
})

function WasteBins() {
  const { t } = useTranslation('waste')
  const search = Route.useSearch()
  const navigate = Route.useNavigate()
  const {
    data: binsRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...wasteBinQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  // Every change resets `page`: a narrower filter can leave the current page empty.
  const patch = (next: Partial<WasteBinSearch>, replace = false) => {
    navigate({ search: (prev) => ({ ...prev, ...next, page: 1 }), replace }).catch((err) =>
      console.error('Navigation failed:', err),
    )
  }
  const resetFilters = () => patch({ q: undefined, fillState: undefined })

  const fillStateLabel = (state: FillStateDto) => t(`fillState.${state}`)
  const isFiltered = (search.q ?? '').length > 0 || search.fillState !== undefined
  const filtered = binsRes?.pagination.totalRecords ?? 0
  const total = binsRes?.pagination.totalUnfiltered ?? undefined
  const resultLabel =
    total !== undefined && isFiltered
      ? t('list.resultCountFiltered', { filtered, total })
      : t('list.resultCount', { count: total ?? filtered })

  return (
    <div className="container mt-6">
      <ListPageHeader title={t('list.title')} description={t('list.description')} />

      <section className="mt-10">
        <ListToolbar
          search={
            <ListSearchInput
              value={search.q ?? ''}
              onChange={(q) => patch({ q: q.length > 0 ? q : undefined }, true)}
              label={t('list.searchLabel')}
              placeholder={t('list.searchPlaceholder')}
            />
          }
          filters={
            <ListFilterDropdown
              label={t('list.filterFillState')}
              mode="single"
              options={FILL_STATES.map((state) => ({ value: state, label: fillStateLabel(state) }))}
              value={search.fillState ? [search.fillState] : []}
              onChange={(values) => patch({ fillState: values[0] as FillStateDto | undefined })}
              emptyText={t('list.filterNoOptions')}
            />
          }
          sort={null}
          action={
            <Can permission={['waste_bin:create']}>
              <ButtonLink
                icon={Plus}
                label={t('list.createButton')}
                link={{ to: '/waste-bins/new' }}
              />
            </Can>
          }
        />
        <ActiveFilterChips
          chips={
            search.fillState
              ? [
                  {
                    id: `fill-state-${search.fillState}`,
                    label: fillStateLabel(search.fillState),
                    onRemove: () => patch({ fillState: undefined }),
                  },
                ]
              : []
          }
          onReset={resetFilters}
          resultLabel={resultLabel}
        />

        {!binsRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="mt-6 transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {binsRes.data.length === 0 && isFiltered ? (
              <div className="mt-10 text-center">
                <p className="text-dark-600">{t('list.emptyFilteredMessage')}</p>
                <Button variant="outline" className="mt-4" onClick={resetFilters}>
                  {t('list.emptyFilteredAction')}
                </Button>
              </div>
            ) : (
              <EntityList
                items={binsRes.data}
                getKey={(bin) => bin.id}
                emptyMessage={t('list.emptyMessage')}
                renderItem={(bin) => <WasteBinCard bin={bin} query={search.q ?? ''} />}
              />
            )}
            {binsRes.pagination.totalPages > 1 && <Pagination pagination={binsRes.pagination} />}
          </div>
        )}
      </section>
    </div>
  )
}

export const Route = createFileRoute('/_protected/waste-bins/')({
  component: WasteBins,
  validateSearch: wasteBinSearchSchema,
  pendingComponent: pendingLoading({ key: 'waste:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({ page: search.page, q: search.q, fillState: search.fillState }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, wasteBinQueries.list(listParams(deps)), 'wasteBinQueries.list')
  },
})
