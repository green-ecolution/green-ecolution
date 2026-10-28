import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { MapPin, Plus } from 'lucide-react'
import { Button, Loading } from '@green-ecolution/ui'
import type { StartPointResponse } from '@/api/backendApi'
import { organizationQueries, routingStartPointsQuery, userQueries } from '@/api/queries'
import { useStationMutations } from '@/hooks/useStationMutations'
import { Can } from '@/lib/auth/Can'
import StationDialog, { type StationPayload } from '@/components/settings/stations/StationDialog'
import StationList from '@/components/settings/stations/StationList'

export const Route = createFileRoute('/_protected/settings/stations/')({
  component: StationsView,
})

type DialogState = { mode: 'create' } | { mode: 'edit'; station: StartPointResponse } | null

function StationsView() {
  const { t } = useTranslation('settings')
  const { data: stations, isPending } = useQuery(routingStartPointsQuery())
  const { data: organizations } = useQuery(organizationQueries.list())
  const { data: mapView } = useQuery(userQueries.mapView())
  const { create, update, remove, setDefault, transfer } = useStationMutations()
  const [dialog, setDialog] = useState<DialogState>(null)

  const handleSubmit = (payload: StationPayload) => {
    const onSuccess = () => setDialog(null)
    if (dialog?.mode === 'edit') {
      update.mutate({ id: dialog.station.id, body: payload }, { onSuccess })
      return
    }
    create.mutate(payload, { onSuccess })
  }

  return (
    <div className="flex flex-col gap-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <article className="flex-1">
          <h2 className="font-lato text-xl font-bold">{t('stations.title')}</h2>
          <p className="mt-2 max-w-prose text-sm text-dark-600">{t('stations.intro')}</p>
        </article>
        {stations && (
          <Can permission={['start_point:create']}>
            <Button
              type="button"
              size="sm"
              className="w-full sm:w-auto sm:shrink-0"
              onClick={() => setDialog({ mode: 'create' })}
            >
              <Plus className="size-4" aria-hidden />
              {t('stations.createButton')}
            </Button>
          </Can>
        )}
      </div>

      {isPending ? (
        <Loading className="mt-10 justify-center" label={t('stations.loading')} />
      ) : stations === null || stations === undefined ? (
        <div className="rounded-xl border border-dashed border-dark-200 bg-white p-8 text-center">
          <MapPin className="mx-auto size-8 text-dark-400" aria-hidden />
          <p className="mx-auto mt-4 max-w-prose text-sm text-dark-600">
            {t('stations.routingDisabled')}
          </p>
        </div>
      ) : (
        <StationList
          stations={stations}
          organizations={organizations ?? []}
          onEdit={(station) => setDialog({ mode: 'edit', station })}
          onSetDefault={(station) => setDefault.mutate(station.id)}
          onDelete={(station) => remove.mutate(station.id)}
          onTransfer={(station, organizationId) =>
            transfer.mutate({ id: station.id, organizationId })
          }
        />
      )}

      <StationDialog
        open={dialog !== null}
        mode={dialog?.mode ?? 'create'}
        station={dialog?.mode === 'edit' ? dialog.station : undefined}
        organizations={organizations ?? []}
        initialPosition={mapView && { lat: mapView.center[0], lng: mapView.center[1] }}
        saving={create.isPending || update.isPending}
        onOpenChange={(open) => {
          if (!open) setDialog(null)
        }}
        onSubmit={handleSubmit}
      />
    </div>
  )
}
