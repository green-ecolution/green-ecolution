import GeolocationPermissionNotice from '@/components/geolocation/GeolocationPermissionNotice'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import type { TargetWizardStepProps } from '@/modules/types'
import {
  AccuracyBadge,
  Alert,
  AlertContent,
  AlertDescription,
  AlertTitle,
  Button,
  InlineAlert,
  Loading,
} from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2, MapPin, Search } from 'lucide-react'
import { useEffect, useState } from 'react'
import { treeQueries } from '../../../api/queries'
import { isTreeTarget } from '../../../lib/targets'
import NearestTreeMapPreview from '../../geolocation/NearestTreeMapPreview'
import NearestTreeList from '../NearestTreeList'
import SensorTreePickerSheet from '../SensorTreePickerSheet'
import { useTreeSelection } from '../useTreeSelection'

const SensorTreeStep = ({
  current,
  onSelect,
  position,
  geolocationStatus: status,
  geolocationError: errorMessage,
  onRelocate,
}: TargetWizardStepProps) => {
  const { t } = useTranslation(['irrigation', 'common'])
  const [pickerOpen, setPickerOpen] = useState(false)
  const treeSelection = useTreeSelection()
  const selectedTreeId = isTreeTarget(current) ? current.id : null

  const {
    data: nearestTrees,
    isLoading: treesLoading,
    isError: treesError,
    refetch: refetchTrees,
  } = useQuery({
    ...treeQueries.nearest({ lat: position?.latitude ?? 0, lng: position?.longitude ?? 0 }),
    enabled: !!position,
  })

  const trees = nearestTrees?.data ?? []
  const selectedNearest = trees.find((t) => t.tree.id === selectedTreeId)
  const isSelectionInNearest = selectedTreeId !== null && Boolean(selectedNearest)

  const { data: outsideTree } = useQuery({
    ...treeQueries.detail(selectedTreeId ?? ''),
    enabled: selectedTreeId !== null && !isSelectionInNearest,
  })

  useEffect(() => {
    if (outsideTree) onSelect(treeSelection(outsideTree))
  }, [outsideTree, onSelect, treeSelection])

  const handleNearestSelect = (treeId: string) => {
    const tree = trees.find((x) => x.tree.id === treeId)?.tree
    if (tree) onSelect(treeSelection(tree))
  }

  const gpsNotice: 'denied' | 'unsupported' | 'error' | null =
    status === 'denied' || status === 'unsupported' || status === 'error' ? status : null
  const gpsPending = !position && !gpsNotice

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato font-bold text-3xl lg:text-4xl">
          {t('sensor.wizard.tree.header.title')}
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          {t('sensor.wizard.tree.header.description')}
        </p>
      </header>

      {position && (
        <div className="flex items-center gap-3 text-sm">
          <AccuracyBadge accuracyMeters={position.accuracy} />
          <Button variant="link" size="sm" className="h-auto px-0" onClick={onRelocate}>
            <MapPin className="size-4" />
            {t('sensor.wizard.tree.relocateButton')}
          </Button>
        </div>
      )}

      {gpsPending && (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-3 text-sm text-muted-foreground"
        >
          <Loader2 className="size-5 animate-spin" aria-hidden />
          {t('sensor.wizard.tree.locatingNotice')}
        </div>
      )}

      {gpsNotice && (
        <GeolocationPermissionNotice
          status={gpsNotice}
          errorMessage={errorMessage}
          onRetry={gpsNotice === 'unsupported' ? undefined : onRelocate}
        />
      )}

      {position && (
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
      )}

      {position && treesLoading && (
        <Loading size="default" label={t('sensor.wizard.tree.loadingNearby')} />
      )}

      {position && treesError && (
        <Alert variant="destructive">
          <AlertContent>
            <AlertTitle>{t('sensor.wizard.tree.searchFailed.title')}</AlertTitle>
            <AlertDescription>{t('sensor.wizard.tree.searchFailed.description')}</AlertDescription>
          </AlertContent>
          <Button variant="outline" size="sm" onClick={() => void refetchTrees()}>
            {t('common:actions.retry')}
          </Button>
        </Alert>
      )}

      {position && !treesLoading && !treesError && trees.length === 0 && (
        <InlineAlert variant="warning" description={t('sensor.wizard.tree.noNearbyTrees')} />
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
        {t('sensor.treePickerSheet.title')}
      </Button>

      <SensorTreePickerSheet
        open={pickerOpen}
        onOpenChange={setPickerOpen}
        selectedTreeId={selectedTreeId}
        onSelect={(id) => {
          const inNearest = trees.find((x) => x.tree.id === id)?.tree
          // Placeholder until treeQueries.detail resolves and the effect re-dispatches with the full record.
          onSelect(treeSelection(inNearest ?? { id, number: '', species: '' }))
        }}
      />

      {selectedTreeId && !isSelectionInNearest && outsideTree && (
        <div className="rounded-xl border border-green-dark/30 bg-green-dark-50/30 p-4">
          <p className="text-xs uppercase tracking-wide font-semibold text-green-dark mb-1">
            {t('sensor.wizard.tree.selectedTreeLabel')}
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
