import { nearestTreeQuery } from '@/api/queries'
import GeolocationPermissionNotice from '@/components/geolocation/GeolocationPermissionNotice'
import GPSStatusCard from '@/components/geolocation/GPSStatusCard'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import NearestTreeMapPreview from '@/components/geolocation/NearestTreeMapPreview'
import NearestTreeList from '@/components/sensor/NearestTreeList'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import {
  Button,
  InlineAlert,
  Loading,
  Alert,
  AlertContent,
  AlertTitle,
  AlertDescription,
  CopyableText,
  toast,
} from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { CheckCircle2, Crosshair, Loader2, MapPin, RotateCw, TreeDeciduous } from 'lucide-react'
import { useCallback, useState } from 'react'

interface SensorGeolocationSummaryProps {
  sensorId: string
  position: GeolocationFix | null
  status: GeolocationStatus
  errorMessage: string | null
  onScanAgain: () => void
  onRelocate: () => void
  onConfirmTree?: (treeId: number) => void
}

const MapPlaceholder = ({ status }: { status: GeolocationStatus }) => {
  const isLoading = status === 'requesting' || status === 'idle'
  return (
    <div
      role="status"
      aria-live="polite"
      className="flex aspect-[4/3] sm:aspect-[16/10] w-full flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-dark-200 bg-dark-50 text-muted-foreground"
    >
      {isLoading ? (
        <>
          <Loader2 className="size-6 animate-spin" aria-hidden />
          <p className="text-sm">GPS wird ermittelt …</p>
        </>
      ) : (
        <>
          <Crosshair className="size-6" aria-hidden />
          <p className="text-sm">Keine Position verfügbar</p>
        </>
      )}
    </div>
  )
}

const SensorGeolocationSummary = ({
  sensorId,
  position,
  status,
  errorMessage,
  onScanAgain,
  onRelocate,
  onConfirmTree,
}: SensorGeolocationSummaryProps) => {
  const noticeStatus: 'denied' | 'unsupported' | 'error' | null =
    status === 'denied' || status === 'unsupported' || status === 'error' ? status : null

  const [selectedTreeId, setSelectedTreeId] = useState<number | null>(null)
  const [confirmed, setConfirmed] = useState(false)

  const {
    data: nearestTrees,
    isLoading: treesLoading,
    isError: treesError,
    refetch: refetchTrees,
  } = useQuery({
    ...nearestTreeQuery({
      lat: position?.latitude ?? 0,
      lng: position?.longitude ?? 0,
    }),
    enabled: !!position,
  })

  const trees = nearestTrees?.data ?? []
  const selectedTree = trees.find((t) => t.tree.id === selectedTreeId)

  const handleConfirm = useCallback(() => {
    if (!selectedTree) return
    setConfirmed(true)
    toast.success(`Sensor wird Baum ${selectedTree.tree.number} zugeordnet`)
    onConfirmTree?.(selectedTree.tree.id)
  }, [selectedTree, onConfirmTree])

  return (
    <div className="mx-auto w-full max-w-3xl pb-[env(safe-area-inset-bottom)]">
      {/* Status line */}
      <div className="flex items-center gap-2 text-sm font-medium text-green-dark mb-4">
        <CheckCircle2 className="size-4" aria-hidden />
        <span>Sensor erfasst</span>
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-6">
        {/* Sensor-ID — spans both columns */}
        <CopyableText value={sensorId} label="Sensor-ID" className="md:col-span-2" />

        {/* Map */}
        <div className="md:col-span-1">
          {position && trees.length > 0 ? (
            <NearestTreeMapPreview
              sensorLat={position.latitude}
              sensorLng={position.longitude}
              sensorAccuracy={position.accuracy}
              trees={trees}
              selectedTreeId={selectedTreeId}
              onSelectTree={setSelectedTreeId}
            />
          ) : position ? (
            <LocationMapPreview
              latitude={position.latitude}
              longitude={position.longitude}
              accuracyMeters={position.accuracy}
            />
          ) : noticeStatus ? (
            <GeolocationPermissionNotice
              status={noticeStatus}
              errorMessage={errorMessage}
              onRetry={noticeStatus === 'unsupported' ? undefined : onRelocate}
            />
          ) : (
            <MapPlaceholder status={status} />
          )}
        </div>

        {/* GPS status */}
        <div className="md:col-span-1">
          <GPSStatusCard fix={position} title="Erfasster Standort" />
        </div>

        {/* Nearest trees */}
        {position && (
          <div className="md:col-span-2">
            {treesLoading && <Loading size="default" label="Bäume in der Nähe werden gesucht…" />}

            {treesError && (
              <Alert variant="destructive">
                <AlertContent>
                  <AlertTitle>Baumsuche fehlgeschlagen</AlertTitle>
                  <AlertDescription>
                    Die Suche nach Bäumen in der Nähe ist fehlgeschlagen.
                  </AlertDescription>
                </AlertContent>
                <Button variant="outline" size="sm" onClick={() => void refetchTrees()}>
                  Erneut versuchen
                </Button>
              </Alert>
            )}

            {!treesLoading && !treesError && trees.length === 0 && (
              <InlineAlert
                variant="warning"
                description="Es wurden keine Bäume in der Nähe gefunden. Überprüfe den Standort oder ordne den Sensor manuell zu."
              />
            )}

            {trees.length > 0 && (
              <NearestTreeList
                trees={trees}
                selectedTreeId={selectedTreeId}
                onSelect={setSelectedTreeId}
              />
            )}
          </div>
        )}

        {/* Confirmed notice */}
        {confirmed && (
          <div className="md:col-span-2">
            <InlineAlert
              variant="info"
              description="Die Verknüpfung wird gespeichert, sobald die Sensor-Synchronisation verfügbar ist."
            />
          </div>
        )}

        {/* Actions */}
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-3 md:col-span-2">
          <Button variant="outline" onClick={onScanAgain}>
            <RotateCw className="size-4" />
            Erneut scannen
          </Button>
          <Button variant="outline" onClick={onRelocate} disabled={status === 'requesting'}>
            <MapPin className="size-4" />
            Erneut lokalisieren
          </Button>
          {trees.length > 0 && (
            <Button
              onClick={handleConfirm}
              disabled={!selectedTreeId || confirmed}
              variant={confirmed ? 'outline' : 'default'}
            >
              {confirmed ? (
                <>
                  <CheckCircle2 className="size-4" />
                  Zugeordnet
                </>
              ) : (
                <>
                  <TreeDeciduous className="size-4" />
                  Baum zuordnen
                </>
              )}
            </Button>
          )}
        </div>
      </div>
    </div>
  )
}

export default SensorGeolocationSummary
