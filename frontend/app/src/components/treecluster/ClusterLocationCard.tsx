import type { LngLatBoundsLike } from 'maplibre-gl'
import { Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import MapPreview from '@/components/map-gl/MapPreview'
import useTreeMarkerLayer from '@/components/map-gl/layers/useTreeMarkerLayer'
import GeneralLink from '@/components/general/links/GeneralLink'
import type { Tree, TreeCluster } from '@/api/backendApi'

interface ClusterLocationCardProps {
  treecluster: TreeCluster
}

const ClusterTreesLayer = ({ trees }: { trees: Tree[] }) => {
  useTreeMarkerLayer({
    trees: trees.map((tree) => ({
      id: tree.id,
      longitude: tree.longitude,
      latitude: tree.latitude,
      status: tree.wateringStatus,
    })),
    sourceId: 'cluster-detail-trees',
    circleLayerId: 'cluster-detail-trees-circle',
    iconLayerId: 'cluster-detail-trees-icon',
    interactive: false,
  })
  return null
}

const clusterTreeBounds = (trees: Tree[]): LngLatBoundsLike | undefined => {
  if (trees.length < 2) return undefined
  const longitudes = trees.map((tree) => tree.longitude)
  const latitudes = trees.map((tree) => tree.latitude)
  return [
    [Math.min(...longitudes), Math.min(...latitudes)],
    [Math.max(...longitudes), Math.max(...latitudes)],
  ]
}

const ClusterLocationCard = ({ treecluster }: ClusterLocationCardProps) => {
  const trees = treecluster.trees
  const hasTrees = trees.length > 0
  const bounds = clusterTreeBounds(trees)

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>Standort</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {hasTrees ? (
          <MapPreview
            center={bounds ? undefined : [treecluster.longitude, treecluster.latitude]}
            bounds={bounds}
            ariaLabel="Karte mit den Bäumen der Bewässerungsgruppe"
            className="h-56"
          >
            <ClusterTreesLayer trees={trees} />
          </MapPreview>
        ) : (
          <p className="flex h-56 items-center justify-center rounded-2xl border border-dark-100 bg-dark-50/40 text-center text-sm text-muted-foreground">
            Keine Bäume — kein Standort.
          </p>
        )}
        {hasTrees && (
          <GeneralLink
            link={{
              to: '/map',
              search: {
                lat: treecluster.latitude,
                lng: treecluster.longitude,
                zoom: 16,
                cluster: treecluster.id,
              },
            }}
            label="Auf der Karte anzeigen"
          />
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterLocationCard
