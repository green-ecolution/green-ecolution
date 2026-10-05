import { Suspense, useMemo, useState, type ReactNode } from 'react'
import { Link, type LinkProps } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  cn,
} from '@green-ecolution/ui'
import { ChevronDown, ChevronRight, Link2, TreeDeciduous, Trees } from 'lucide-react'
import { clusterQueries, treeQueries } from '@/api/queries'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import useViewportBBox from '@/components/map-gl/hooks/useViewportBBox'
import useTreeMarkerLayer, {
  type TreeMarkerPoint,
} from '@/components/map-gl/layers/useTreeMarkerLayer'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import type { Sensor } from '@/api/backendApi'
import { useSensorActions } from './SensorActionsContext'
import { compactCardContent, compactCardHeader, compactCardTitle } from './cardDensity'

interface SensorPlacementCardProps {
  sensor: Sensor
}

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

interface PlacementRowProps {
  icon: ReactNode
  caption: string
  title: ReactNode
  meta?: ReactNode
  link?: LinkProps
}

const PlacementRow = ({ icon, caption, title, meta, link }: PlacementRowProps) => {
  const body = (
    <>
      <span className="grid size-10 shrink-0 place-items-center rounded-lg bg-green-dark-50 text-green-dark [&_svg]:size-5">
        {icon}
      </span>
      <span className="flex min-w-0 flex-1 flex-col">
        <span className="text-xs text-muted-foreground">{caption}</span>
        <span className="truncate font-semibold">{title}</span>
        {meta && (
          <span className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
            {meta}
          </span>
        )}
      </span>
      {link && (
        <ChevronRight
          className="size-5 shrink-0 text-muted-foreground transition-colors group-hover:text-green-dark"
          aria-hidden
        />
      )}
    </>
  )
  const rowClass = 'flex min-h-16 items-center gap-3 px-3 py-2.5'

  return link ? (
    <Link
      {...link}
      className={cn(
        rowClass,
        'group transition-colors hover:bg-green-dark-50/50 focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-green-dark',
      )}
    >
      {body}
    </Link>
  ) : (
    <div className={rowClass}>{body}</div>
  )
}

const ClusterRow = ({ clusterId, fallbackName }: { clusterId: string; fallbackName: string }) => {
  const { t } = useTranslation('sensor')
  const getWateringStatusDetails = useWateringStatusDetails()
  const { data: cluster } = useQuery(clusterQueries.detail(clusterId))
  const status = cluster ? getWateringStatusDetails(cluster.wateringStatus) : null

  return (
    <PlacementRow
      icon={<Trees />}
      caption={t('placement.clusterLabel')}
      title={cluster?.name ?? fallbackName}
      meta={
        cluster &&
        status && (
          <>
            <Badge variant={status.color}>{status.label}</Badge>
            <span>{t('placement.treeCount', { count: cluster.trees.length })}</span>
          </>
        )
      }
      link={{ to: '/treecluster/$treeclusterId', params: { treeclusterId: clusterId } }}
    />
  )
}

const TreeRow = ({ treeId }: { treeId: string }) => {
  const { t } = useTranslation('sensor')
  const { data: tree, isLoading, isError } = useQuery(treeQueries.detail(treeId))

  if (isLoading) {
    return <div className="mx-3 my-2.5 h-11 rounded-lg bg-dark-50 animate-pulse" />
  }
  if (isError || !tree) {
    return (
      <PlacementRow
        icon={<TreeDeciduous />}
        caption={t('linkedTree.loadFailedTitle')}
        title={t('linkedTree.loadFailedDescription', { id: treeId })}
      />
    )
  }

  return (
    <PlacementRow
      icon={<TreeDeciduous />}
      caption={t('placement.treeLabel')}
      title={t('linkedTree.treeNumberLabel', { number: tree.number })}
      meta={
        <>
          <span>{tree.species || t('linkedTree.unknownSpecies')}</span>
          <span>{t('linkedTree.plantedLabel', { year: tree.plantingYear })}</span>
        </>
      }
      link={{ to: '/trees/$treeId', params: { treeId: String(tree.id) } }}
    />
  )
}

const SensorPlacementCard = ({ sensor }: SensorPlacementCardProps) => {
  const { t } = useTranslation('sensor')
  const actions = useSensorActions()
  const coord = sensor.coordinate
  const [showCoordinates, setShowCoordinates] = useState(false)
  // On touch screens a pannable preview swallows the swipe meant to scroll the page.
  const mapInteractive = useMediaQuery('(pointer: fine)')
  const treeId = sensor.linkedTreeId != null ? String(sensor.linkedTreeId) : null
  const clusterId = sensor.linkedClusterId != null ? String(sensor.linkedClusterId) : null

  return (
    <Card variant="outlined" className="h-full">
      <CardHeader className={cn(compactCardHeader, 'flex-row items-center justify-between gap-3')}>
        <CardTitle className={compactCardTitle}>{t('placement.title')}</CardTitle>
        {coord && (
          <button
            type="button"
            onClick={() => setShowCoordinates((v) => !v)}
            aria-expanded={showCoordinates}
            className="-my-1.5 inline-flex min-h-9 items-center gap-1 rounded-lg px-2.5 text-sm font-medium text-green-dark transition hover:bg-green-dark-50"
          >
            {t('location.coordinatesToggle')}
            <ChevronDown
              className={cn(
                'size-4 transition-transform duration-base ease-out motion-reduce:transition-none',
                showCoordinates && 'rotate-180',
              )}
              aria-hidden
            />
          </button>
        )}
      </CardHeader>
      <CardContent className={cn(compactCardContent, 'flex flex-col gap-4')}>
        {treeId === null ? (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>{t('linkedTree.noLinkTitle')}</AlertTitle>
                <AlertDescription>{t('linkedTree.noLinkDescription')}</AlertDescription>
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-3 gap-2 [&_svg]:size-4"
                  onClick={() => actions.requestActivate()}
                >
                  <Link2 />
                  {t('actions.activateAssignTree')}
                </Button>
              </AlertContent>
            </div>
          </Alert>
        ) : (
          <>
            {coord && (
              <MapPreview
                center={[coord.longitude, coord.latitude]}
                zoom={17}
                interactive={mapInteractive}
                ariaLabel={t('location.mapAriaLabel')}
                className="h-48 sm:h-56 xl:h-52"
              >
                <Suspense fallback={null}>
                  <LocationClusterBoundaries />
                </Suspense>
                <LocationTreeLayer />
                <SensorMarker lng={coord.longitude} lat={coord.latitude} />
              </MapPreview>
            )}

            {coord && showCoordinates && (
              <div className="rounded-xl bg-dark-50 px-4 py-3 text-sm">
                <dl className="grid grid-cols-2 gap-x-6">
                  <div>
                    <dt className="text-muted-foreground">{t('location.latitudeLabel')}</dt>
                    <dd className="font-mono font-semibold">{coord.latitude.toFixed(6)}°</dd>
                  </div>
                  <div>
                    <dt className="text-muted-foreground">{t('location.longitudeLabel')}</dt>
                    <dd className="font-mono font-semibold">{coord.longitude.toFixed(6)}°</dd>
                  </div>
                </dl>
                <p className="mt-2 text-xs text-muted-foreground">
                  {t('location.derivedFromTreeNotice')}
                </p>
              </div>
            )}

            <div className="divide-y divide-dark-100 overflow-hidden rounded-xl border border-dark-100">
              {clusterId ? (
                <ClusterRow clusterId={clusterId} fallbackName={sensor.linkedClusterName ?? ''} />
              ) : (
                <PlacementRow
                  icon={<Trees />}
                  caption={t('placement.clusterLabel')}
                  title={t('placement.noClusterDescription')}
                />
              )}
              <TreeRow treeId={treeId} />
            </div>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorPlacementCard
