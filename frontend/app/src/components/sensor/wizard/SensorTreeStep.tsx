import { nearestTreeQuery, treeIdQuery } from '@/api/queries'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import NearestTreeMapPreview from '@/components/geolocation/NearestTreeMapPreview'
import NearestTreeList from '@/components/sensor/NearestTreeList'
import SensorTreePickerSheet from '@/components/sensor/SensorTreePickerSheet'
import type { GeolocationFix } from '@/hooks/useGeolocation'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertTitle,
  Button,
  InlineAlert,
  Loading,
} from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { Search } from 'lucide-react'
import { useEffect, useState } from 'react'

interface SensorTreeStepProps {
  position: GeolocationFix
  selectedTreeId: string | null
  onSelect: (treeId: string, number: string, species: string) => void
}

const SensorTreeStep = ({ position, selectedTreeId, onSelect }: SensorTreeStepProps) => {
  const [pickerOpen, setPickerOpen] = useState(false)

  const {
    data: nearestTrees,
    isLoading: treesLoading,
    isError: treesError,
    refetch: refetchTrees,
  } = useQuery({
    ...nearestTreeQuery({ lat: position.latitude, lng: position.longitude }),
  })

  const trees = nearestTrees?.data ?? []
  const selectedNearest = trees.find((t) => t.tree.id === selectedTreeId)
  const isSelectionInNearest = selectedTreeId !== null && Boolean(selectedNearest)

  const { data: outsideTree } = useQuery({
    ...treeIdQuery(selectedTreeId ?? ''),
    enabled: selectedTreeId !== null && !isSelectionInNearest,
  })

  useEffect(() => {
    if (outsideTree) {
      onSelect(outsideTree.id, outsideTree.number, outsideTree.species)
    }
  }, [outsideTree, onSelect])

  const handleNearestSelect = (treeId: string) => {
    const t = trees.find((x) => x.tree.id === treeId)?.tree
    if (t) onSelect(t.id, t.number, t.species)
  }

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato font-bold text-3xl lg:text-4xl">Baum zuordnen</h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Wähle den Baum aus, an dem der Sensor angebracht wird. Der Vorschlag basiert auf dem
          erfassten Standort.
        </p>
      </header>

      <div>
        {trees.length > 0 ? (
          <NearestTreeMapPreview
            sensorLat={position.latitude}
            sensorLng={position.longitude}
            sensorAccuracy={position.accuracy}
            trees={trees}
            selectedTreeId={selectedTreeId}
            onSelectTree={handleNearestSelect}
          />
        ) : (
          <LocationMapPreview
            latitude={position.latitude}
            longitude={position.longitude}
            accuracyMeters={position.accuracy}
          />
        )}
      </div>

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
          description="Es wurden keine Bäume in der Nähe gefunden. Wähle den Baum manuell aus."
        />
      )}

      {trees.length > 0 && (
        <NearestTreeList
          trees={trees}
          selectedTreeId={selectedTreeId}
          onSelect={handleNearestSelect}
        />
      )}

      <Button variant="outline" onClick={() => setPickerOpen(true)} className="w-full sm:w-auto">
        <Search className="size-4" />
        Anderen Baum auswählen
      </Button>

      <SensorTreePickerSheet
        open={pickerOpen}
        onOpenChange={setPickerOpen}
        selectedTreeId={selectedTreeId}
        onSelect={(id) => {
          const inNearest = trees.find((t) => t.tree.id === id)?.tree
          if (inNearest) onSelect(inNearest.id, inNearest.number, inNearest.species)
          else
            // Placeholder until treeIdQuery resolves and the effect re-dispatches with the full record.
            onSelect(id, '', '')
        }}
      />

      {selectedTreeId && !isSelectionInNearest && outsideTree && (
        <div className="rounded-xl border border-green-dark/30 bg-green-dark-50/30 p-4">
          <p className="text-xs uppercase tracking-wide font-semibold text-green-dark mb-1">
            Ausgewählter Baum
          </p>
          <div className="flex items-baseline gap-3 text-sm">
            <span className="font-semibold">{outsideTree.species}</span>
            <span className="font-mono text-xs text-dark-600">{outsideTree.number}</span>
          </div>
        </div>
      )}
    </div>
  )
}

export default SensorTreeStep
