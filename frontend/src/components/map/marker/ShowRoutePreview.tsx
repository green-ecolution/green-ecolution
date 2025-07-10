import { routePreviewQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { GeoJSON } from 'react-leaflet/GeoJSON'
import RouteMarker from './RouteMarker'

type BBox = [number, number, number, number] | [number, number, number, number, number, number]

export interface ShowRoutePreview {
  transporterId: number
  trailerId?: number
  selectedClustersIds: number[]
}

const ShowRoutePreview = ({ transporterId, trailerId, selectedClustersIds }: ShowRoutePreview) => {
  if (trailerId === -1) {
    trailerId = undefined
  }
  const { data } = useSuspenseQuery(
    routePreviewQuery(transporterId, selectedClustersIds, trailerId),
  )

  const sameStartAndEnd = (
    startPoint: { latitude: number; longitude: number },
    endPoint: { latitude: number; longitude: number },
  ): boolean => {
    return startPoint.latitude === endPoint.latitude && startPoint.longitude === endPoint.longitude
  }

  return (
    <>
      {sameStartAndEnd(data.metadata.startPoint, data.metadata.endPoint) ? (
        <RouteMarker label="Start/Ende" type="route" position={data.metadata.startPoint} />
      ) : (
        <>
          <RouteMarker label="Start" type="route" position={data.metadata.startPoint} />
          <RouteMarker label="Ende" type="route" position={data.metadata.endPoint} />
        </>
      )}
      <RouteMarker type="refill" position={data.metadata.wateringPoint} />
      {/* GeoJSON needs a unique key to rerender */}
      <GeoJSON
        key={selectedClustersIds.join('-') + transporterId + trailerId}
        data={data as Omit<typeof data, 'bbox'> & { bbox: BBox }}
      />
    </>
  )
}

export default ShowRoutePreview
