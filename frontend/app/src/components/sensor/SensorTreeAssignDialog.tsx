import {
  Button,
  Checkbox,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  InlineAlert,
  cn,
} from '@green-ecolution/ui'
import { useEffect, useMemo, useState } from 'react'
import type { LngLatBoundsLike } from 'maplibre-gl'
import type { Sensor } from '@/api/backendApi'
import { useMapStore } from '@/store/store'
import { useTreeSearch } from '@/hooks/useTreeSearch'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import MapPreview from '@/components/map-gl/MapPreview'
import SelectableTreeMarkers from '@/components/map-gl/SelectableTreeMarkers'
import SensorTreeSearchInput from './SensorTreeSearchInput'
import SensorTreeSearchResults from './SensorTreeSearchResults'

export type AssignMode = 'activate' | 'reassign'

interface SensorTreeAssignDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: AssignMode
  sensor: Sensor
  isPending: boolean
  errorMessage: string | null
  onConfirm: (treeId: string) => void
}

const COPY: Record<AssignMode, { title: string; description: string; confirm: string }> = {
  activate: {
    title: 'Sensor aktivieren und Baum zuweisen',
    description: 'Suche den Baum nach Nummer oder Art und aktiviere den Sensor an diesem Baum.',
    confirm: 'Aktivieren',
  },
  reassign: {
    title: 'Baum ändern',
    description: 'Wähle den neuen Baum, an den dieser Sensor umgehängt werden soll.',
    confirm: 'Baum übernehmen',
  },
}

const FOCUS_ZOOM = 18

const FocusTree = ({ lng, lat }: { lng: number; lat: number }) => {
  const map = useMaplibreMap()
  useEffect(() => {
    map.flyTo({ center: [lng, lat], zoom: FOCUS_ZOOM })
  }, [map, lng, lat])
  return null
}

const DialogBody = ({
  sensor,
  selectedTreeId,
  onSelect,
}: {
  sensor: Sensor
  selectedTreeId: string | null
  onSelect: (treeId: string) => void
}) => {
  const [q, setQ] = useState('')
  const [showAll, setShowAll] = useState(false)
  const { items, enabled } = useTreeSearch(q, showAll)
  const { mapCenter, mapZoom } = useMapStore()

  const visibleItems = useMemo(() => (enabled ? items : []), [enabled, items])

  const selectedTree = useMemo(
    () => visibleItems.find((t) => t.id === selectedTreeId) ?? null,
    [visibleItems, selectedTreeId],
  )

  const bounds = useMemo<LngLatBoundsLike | undefined>(() => {
    if (selectedTree || visibleItems.length === 0) return undefined
    const lngs = visibleItems.map((t) => t.longitude)
    const lats = visibleItems.map((t) => t.latitude)
    let w = Math.min(...lngs)
    let e = Math.max(...lngs)
    let s = Math.min(...lats)
    let n = Math.max(...lats)
    const padX = (e - w) * 0.3 || 0.001
    const padY = (n - s) * 0.3 || 0.001
    w -= padX
    e += padX
    s -= padY
    n += padY
    return [
      [w, s],
      [e, n],
    ]
  }, [visibleItems, selectedTree])

  const center: [number, number] = sensor.coordinate
    ? [sensor.coordinate.longitude, sensor.coordinate.latitude]
    : [mapCenter[1], mapCenter[0]]

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-4 px-6 pb-2 sm:flex-row">
      <div className="flex min-h-0 flex-1 flex-col gap-3">
        <SensorTreeSearchInput value={q} onChange={setQ} />
        <label className="flex cursor-pointer select-none items-center gap-2 text-sm text-dark-700">
          <Checkbox
            checked={showAll}
            onCheckedChange={(value) => setShowAll(value === true)}
            aria-label="Alle Bäume anzeigen"
          />
          Alle Bäume anzeigen
        </label>
        <div className="min-h-0 flex-1 overflow-y-auto">
          <SensorTreeSearchResults
            q={q}
            selectedTreeId={selectedTreeId}
            onSelect={onSelect}
            showAll={showAll}
          />
        </div>
      </div>
      <div className="sm:w-1/2">
        <MapPreview
          bounds={bounds}
          center={center}
          zoom={mapZoom}
          interactive
          ariaLabel="Karte zur Baumauswahl"
          className={cn('aspect-[4/3] sm:aspect-auto sm:h-full')}
        >
          <SelectableTreeMarkers
            trees={visibleItems}
            selectedTreeId={selectedTreeId}
            onSelect={onSelect}
          />
          {selectedTree && <FocusTree lng={selectedTree.longitude} lat={selectedTree.latitude} />}
        </MapPreview>
      </div>
    </div>
  )
}

const SensorTreeAssignDialog = ({
  open,
  onOpenChange,
  mode,
  sensor,
  isPending,
  errorMessage,
  onConfirm,
}: SensorTreeAssignDialogProps) => {
  const [selectedTreeId, setSelectedTreeId] = useState<string | null>(null)
  const copy = COPY[mode]

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) setSelectedTreeId(null)
        onOpenChange(next)
      }}
    >
      <DialogContent className="max-w-md sm:max-w-3xl h-[88vh] sm:h-[78vh] flex flex-col gap-0 p-0">
        <DialogHeader className="px-6 pt-6 pb-3">
          <DialogTitle>{copy.title}</DialogTitle>
          <DialogDescription>{copy.description}</DialogDescription>
        </DialogHeader>

        <DialogBody
          key={open ? 'open' : 'closed'}
          sensor={sensor}
          selectedTreeId={selectedTreeId}
          onSelect={setSelectedTreeId}
        />

        <DialogFooter className="flex-col gap-3 px-6 py-4 sm:flex-row sm:items-center sm:justify-end">
          {errorMessage && <InlineAlert variant="destructive" description={errorMessage} />}
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isPending}>
            Abbrechen
          </Button>
          <Button
            disabled={!selectedTreeId || isPending}
            onClick={() => selectedTreeId && onConfirm(selectedTreeId)}
          >
            {isPending ? 'Wird gespeichert …' : copy.confirm}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default SensorTreeAssignDialog
export type { SensorTreeAssignDialogProps }
