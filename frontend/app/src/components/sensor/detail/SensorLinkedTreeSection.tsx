import { Link } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@green-ecolution/ui'
import { ArrowUpRight, Link2, TreeDeciduous, Unlink2 } from 'lucide-react'
import { treeIdQuery } from '@/api/queries'
import type { Sensor } from '@/api/backendApi'

interface SensorLinkedTreeSectionProps {
  sensor: Sensor
}

const SensorLinkedTreeSection = ({ sensor }: SensorLinkedTreeSectionProps) => {
  const hasLink = sensor.linkedTreeId != null
  const treeIdStr = hasLink ? String(sensor.linkedTreeId) : ''
  const { data: tree, isLoading, isError } = useQuery(treeIdQuery(treeIdStr))

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            {hasLink ? <Link2 className="size-5" /> : <Unlink2 className="size-5" />}
          </div>
          <CardTitle>Verknüpfter Baum</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        {!hasLink ? (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>Noch keine Verknüpfung</AlertTitle>
                <AlertDescription>
                  Dieser Sensor ist mit keinem Baum verknüpft. Ohne Verknüpfung können seine
                  Messdaten keiner Vegetation zugeordnet werden.
                </AlertDescription>
                {sensor.coordinate && (
                  <div className="mt-3">
                    <Button variant="outline" size="sm" asChild className="gap-2 [&_svg]:size-4">
                      <Link
                        to="/map/sensor/select/tree"
                        search={{
                          lat: sensor.coordinate.latitude,
                          lng: sensor.coordinate.longitude,
                          zoom: 18,
                          sensorId: sensor.id,
                        }}
                      >
                        <Link2 />
                        Auf der Karte verknüpfen
                      </Link>
                    </Button>
                  </div>
                )}
              </AlertContent>
            </div>
          </Alert>
        ) : isLoading ? (
          <div className="h-24 rounded-2xl bg-dark-50 animate-pulse" />
        ) : isError || !tree ? (
          <Alert variant="destructive">
            <div className="flex gap-3">
              <AlertIcon variant="destructive" />
              <AlertContent>
                <AlertTitle>Baum nicht geladen</AlertTitle>
                <AlertDescription>
                  Der verknüpfte Baum (#{sensor.linkedTreeId}) konnte nicht abgerufen werden.
                </AlertDescription>
              </AlertContent>
            </div>
          </Alert>
        ) : (
          <Link
            to="/trees/$treeId"
            params={{ treeId: String(tree.id) }}
            className="group relative flex items-center gap-5 rounded-2xl border border-dark-100 bg-white p-5 transition hover:border-green-dark hover:shadow-cards"
          >
            <div className="grid place-items-center size-14 rounded-xl bg-green-dark text-white shrink-0">
              <TreeDeciduous className="size-7" strokeWidth={1.75} />
            </div>
            <div className="flex-1 min-w-0">
              <p className="text-xs uppercase tracking-widest text-muted-foreground">
                Baum-Nr. {tree.number}
              </p>
              <p className="font-lato font-bold text-xl mt-1 truncate">
                {tree.species || 'Unbekannte Art'}
              </p>
              <p className="text-sm text-muted-foreground mt-0.5">
                Gepflanzt {tree.plantingYear}
              </p>
            </div>
            <ArrowUpRight className="size-5 text-muted-foreground group-hover:text-green-dark transition shrink-0" />
          </Link>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorLinkedTreeSection
