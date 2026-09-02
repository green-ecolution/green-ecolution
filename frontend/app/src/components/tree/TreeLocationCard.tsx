import { Suspense } from 'react'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import MapPreview from '@/components/map-gl/MapPreview'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useTreeMarkerLayer from '@/components/map-gl/layers/useTreeMarkerLayer'
import GeneralLink from '@/components/general/links/GeneralLink'
import type { Tree } from '@/api/backendApi'

interface TreeLocationCardProps {
  tree: Tree
}

const ClusterBoundaryLayer = ({ clusterId }: { clusterId: string }) => {
  useClusterBoundaryLayer({ clusterIds: [clusterId], interactive: false })
  return null
}

const TreeMarkerLayer = ({ tree }: { tree: Tree }) => {
  useTreeMarkerLayer({
    trees: [
      {
        id: tree.id,
        longitude: tree.longitude,
        latitude: tree.latitude,
        status: tree.wateringStatus,
      },
    ],
    sourceId: 'tree-detail-marker',
    circleLayerId: 'tree-detail-marker-circle',
    iconLayerId: 'tree-detail-marker-icon',
    interactive: false,
  })
  return null
}

const TreeLocationCard = ({ tree }: TreeLocationCardProps) => {
  const { t } = useTranslation('tree')

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('locationCard.title')}</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <MapPreview
          center={[tree.longitude, tree.latitude]}
          zoom={18}
          ariaLabel={t('locationCard.mapAriaLabel')}
          className="h-56"
        >
          {/* Boundary before the marker: mount order controls layer stacking. */}
          <Suspense fallback={null}>
            {tree.treeClusterId && <ClusterBoundaryLayer clusterId={tree.treeClusterId} />}
            <TreeMarkerLayer tree={tree} />
          </Suspense>
        </MapPreview>
        <GeneralLink
          link={{
            to: '/map',
            search: {
              lat: tree.latitude,
              lng: tree.longitude,
              zoom: 18,
              tree: tree.id,
            },
          }}
          label={t('locationCard.viewOnMapLabel')}
        />
      </CardContent>
    </Card>
  )
}

export default TreeLocationCard
