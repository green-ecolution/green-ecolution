import { sensorQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import SensorList from '@/components/sensor/SensorList'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, useLoaderData } from '@tanstack/react-router'
import { z } from 'zod'
import { ListCardHeader } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/sensors/')({
  component: Sensors,
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Sensoren werden geladen" />,
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
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Auflistung aller verfügbaren Sensoren
        </h1>
        <p className="mb-4">
          In diesem Bereich werden alle im System registrierten Sensoren angezeigt. Neue Sensoren
          werden automatisch erstellt, sobald über das TTN (The Things Network) neue Daten empfangen
          werden, die keinem vorhandenen Sensor zugeordnet sind. TTN ist ein globales Funknetzwerk,
          das auf der LoRa-Funktechnik basiert. Mit dieser Technologie können die Messdaten der
          vergrabenen Sensoren direkt in das System übertragen werden.
        </p>
        <p>
          Wenn ein neuer Sensor angelegt wird, überprüft das System automatisch, ob die
          mitgesendeten GPS-Koordinaten mit einem im System registrierten Baum übereinstimmen. Falls
          ein passender Baum gefunden wird, wird der Sensor automatisch mit diesem Baum verknüpft.
          Diese Verknüpfung kann bei Bedarf auch manuell angepasst werden.
        </p>
      </article>

      <section className="mt-10">
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
