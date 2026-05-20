import { routePreviewQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'

export interface ShowRoutePreview {
  transporterId: string
  trailerId?: string
  selectedClustersIds: string[]
}

const ShowRoutePreview = ({ transporterId, trailerId, selectedClustersIds }: ShowRoutePreview) => {
  if (trailerId === '' || trailerId === '-1') {
    trailerId = undefined
  }
  // TODO: previewRoute() currently returns void (stub).
  // Once the Rust backend implements the route preview endpoint with proper GeoJSON response,
  // restore the route visualization (markers + GeoJSON layer).
  useSuspenseQuery(routePreviewQuery(transporterId, selectedClustersIds, trailerId))

  return <>{/* Route preview is not yet implemented in the Rust backend */}</>
}

export default ShowRoutePreview
