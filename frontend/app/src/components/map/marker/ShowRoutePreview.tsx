import { servicesInfoQuery } from '@/api/queries'
import { useQuery } from '@tanstack/react-query'

export interface ShowRoutePreview {
  transporterId: string
  trailerId?: string
  selectedClustersIds: string[]
}

const ShowRoutePreview = (_props: ShowRoutePreview) => {
  const { data: services } = useQuery(servicesInfoQuery())
  const routing = services?.items.find((item) => item.name === 'routing')
  if (!routing?.enabled) {
    return null
  }

  // TODO: when routing.enabled flips to true, restore the call to
  // wateringPlanApi.previewRoute() and render the GeoJSON layer.
  return null
}

export default ShowRoutePreview
