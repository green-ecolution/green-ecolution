import { Suspense, useMemo, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Badge } from '@green-ecolution/ui'
import { TreeDeciduous, Trees } from 'lucide-react'
import type { SensorResponse } from '@green-ecolution/backend-client'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import useViewportBBox from '@/components/map-gl/hooks/useViewportBBox'
import PlacementRow from '@/components/sensor/detail/PlacementRow'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { clusterQueries, treeQueries } from '../../api/queries'
import useTreeMarkerLayer, { type TreeMarkerPoint } from '../map-gl/layers/useTreeMarkerLayer'
import useClusterBoundaryLayer from '../map-gl/layers/useClusterBoundaryLayer'
import { useWateringStatusDetails } from '../../hooks/details/useDetailsForWateringStatus'
import { isTreeTarget } from '../../lib/targets'

const LocationTreeLayer = () => {
  const bbox = useViewportBBox()
  const { data } = useQuery(treeQueries.markers({ bbox }))
  const trees = useMemo<TreeMarkerPoint[]>(
    () =>
      (data?.data ?? []).map((t) => ({
        id: t.id,
        longitude: t.longitude,
        latitude: t.latitude,
        status: t.wateringStatus,
      })),
    [data],
  )
  useTreeMarkerLayer({
    trees,
    sourceId: 'gec-location-trees',
    circleLayerId: 'gec-location-tree-circle',
    iconLayerId: 'gec-location-tree-icon',
    interactive: false,
  })
  return null
}

const LocationClusterBoundaries = () => {
  useClusterBoundaryLayer({ interactive: false })
  return null
}

const ClusterRow = ({ clusterId, fallbackName }: { clusterId: string; fallbackName: string }) => {
  const { t } = useTranslation('irrigation')
  const getWateringStatusDetails = useWateringStatusDetails()
  const { data: cluster } = useQuery(clusterQueries.detail(clusterId))
  const status = cluster ? getWateringStatusDetails(cluster.wateringStatus) : null

  return (
    <PlacementRow
      icon={<Trees />}
      caption={t('sensor.placement.clusterLabel')}
      title={cluster?.name ?? fallbackName}
      meta={
        cluster &&
        status && (
          <>
            <Badge variant={status.color}>{status.label}</Badge>
            <span>{t('sensor.placement.treeCount', { count: cluster.trees.length })}</span>
          </>
        )
      }
      link={{ to: '/treecluster/$treeclusterId', params: { treeclusterId: clusterId } }}
    />
  )
}

const TreeRow = ({ treeId }: { treeId: string }) => {
  const { t } = useTranslation('irrigation')
  const { data: tree, isLoading, isError } = useQuery(treeQueries.detail(treeId))

  if (isLoading) {
    return <div className="mx-3 my-2.5 h-11 rounded-lg bg-dark-50 animate-pulse" />
  }
  if (isError || !tree) {
    return (
      <PlacementRow
        icon={<TreeDeciduous />}
        caption={t('sensor.linkedTree.loadFailedTitle')}
        title={t('sensor.linkedTree.loadFailedDescription', { id: treeId })}
      />
    )
  }

  return (
    <PlacementRow
      icon={<TreeDeciduous />}
      caption={t('sensor.placement.treeLabel')}
      title={t('sensor.linkedTree.treeNumberLabel', { number: tree.number })}
      meta={
        <>
          <span>{tree.species || t('sensor.linkedTree.unknownSpecies')}</span>
          <span>{t('sensor.linkedTree.plantedLabel', { year: tree.plantingYear })}</span>
        </>
      }
      link={{ to: '/trees/$treeId', params: { treeId: String(tree.id) } }}
    />
  )
}

const TreePlacement = ({ sensor, children }: { sensor: SensorResponse; children?: ReactNode }) => {
  const { t } = useTranslation('irrigation')
  const coord = sensor.location
  // On touch screens a pannable preview swallows the swipe meant to scroll the page.
  const mapInteractive = useMediaQuery('(pointer: fine)')
  const treeId = isTreeTarget(sensor.target) ? sensor.target.id : ''
  const { data: linkedTree, isLoading: treeLoading } = useQuery(treeQueries.detail(treeId))
  const clusterId = linkedTree?.treeClusterId ?? null

  return (
    <>
      {coord && (
        <MapPreview
          center={[coord.longitude, coord.latitude]}
          zoom={17}
          interactive={mapInteractive}
          ariaLabel={t('sensor.placement.mapAriaLabel')}
          className="h-48 sm:h-56 xl:h-52"
        >
          <Suspense fallback={null}>
            <LocationClusterBoundaries />
          </Suspense>
          <LocationTreeLayer />
          <SensorMarker lng={coord.longitude} lat={coord.latitude} />
        </MapPreview>
      )}

      {children}

      <div className="divide-y divide-dark-100 overflow-hidden rounded-xl border border-dark-100">
        {clusterId ? (
          <ClusterRow clusterId={clusterId} fallbackName={sensor.targetLabel ?? ''} />
        ) : treeLoading ? (
          <div className="mx-3 my-2.5 h-11 rounded-lg bg-dark-50 animate-pulse" />
        ) : (
          <PlacementRow
            icon={<Trees />}
            caption={t('sensor.placement.clusterLabel')}
            title={t('sensor.placement.noClusterDescription')}
          />
        )}
        <TreeRow treeId={treeId} />
      </div>
    </>
  )
}

export default TreePlacement
