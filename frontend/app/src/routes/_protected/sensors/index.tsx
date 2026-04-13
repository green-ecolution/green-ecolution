import { sensorQuery } from '@/api/queries'
import { Button, ListCardHeader, Loading } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import SensorList from '@/components/sensor/SensorList'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, Link, useLoaderData } from '@tanstack/react-router'
import { Plus } from 'lucide-react'
import { z } from 'zod'

export const Route = createFileRoute('/_protected/sensors/')({
  component: Sensors,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Sensoren werden geladen" />
  ),
  validateSearch: z.object({
    page: z.number().catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    queryClient
      .prefetchQuery(sensorQuery({ page, limit: 5 }))
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
    return { page }
  },
})

function Sensors() {
  const { page } = useLoaderData({ from: '/_protected/sensors/' })
  const { data: sensorsRes } = useSuspenseQuery(sensorQuery({ page, limit: 5 }))

  return (
    <div className="container mt-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between 2xl:w-4/5">
        <article className="flex-1">
          <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">Sensoren</h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            Übersicht aller im System registrierten Sensoren. Neue Sensoren kannst du durch Scannen
            des QR-Codes auf der Sensoreinheit hinzufügen.
          </p>
        </article>
        <Button asChild size="sm" className="w-full sm:w-auto sm:shrink-0">
          <Link to="/sensors/new">
            <Plus />
            Sensor hinzufügen
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

        <SensorList data={sensorsRes.data} />
        {sensorsRes.pagination && sensorsRes.pagination?.totalPages > 1 && (
          <Pagination pagination={sensorsRes.pagination} />
        )}
      </section>
    </div>
  )
}
