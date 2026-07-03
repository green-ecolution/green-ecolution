import { sensorQuery } from '@/api/queries'
import { Button, ListCardHeader, Loading } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import SensorList from '@/components/sensor/SensorList'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute, Link } from '@tanstack/react-router'
import { Zap } from 'lucide-react'
import { z } from 'zod'
import { pendingLoading, prefetch } from '@/lib/router'

export const Route = createFileRoute('/_protected/sensors/')({
  component: Sensors,
  pendingComponent: pendingLoading('Sensoren werden geladen'),
  validateSearch: z.object({
    page: z.number().int().min(1).catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    prefetch(queryClient, sensorQuery({ page, perPage: 5 }), 'sensorQuery')
  },
})

function Sensors() {
  const { page } = Route.useSearch()
  const {
    data: sensorsRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...sensorQuery({ page, perPage: 5 }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  return (
    <div className="container mt-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <article className="flex-1">
          <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">Sensoren</h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            Übersicht aller im System registrierten Sensoren. Neue Sensoren kannst du durch Scannen
            des QR-Codes auf der Sensoreinheit hinzufügen.
          </p>
        </article>
        <Button asChild size="sm" className="w-full sm:w-auto sm:shrink-0">
          <Link to="/sensors/new">
            <Zap />
            Sensor aktivieren
          </Link>
        </Button>
      </div>

      <section className="mt-8">
        <ListCardHeader columns="1fr 2fr 1fr 1fr">
          <p>Status</p>
          <p>Name und Verknüpfung</p>
          <p>Erstelldatum</p>
          <p>Letztes Datenupdate</p>
        </ListCardHeader>

        {!sensorsRes ? (
          <Loading className="mt-10 justify-center" label="Sensoren werden geladen" />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <SensorList data={sensorsRes.data} />
            {sensorsRes.pagination && sensorsRes.pagination?.totalPages > 1 && (
              <Pagination pagination={sensorsRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}
