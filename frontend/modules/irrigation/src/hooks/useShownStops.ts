import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { keepUnresolvedStops } from '@/modules/planning'
import { clusterQueries } from '../api/queries'

/**
 * The stops a selection list can show: clusters of other organizations are left
 * out, and `keepUnshown` hands them back to the core so a save never drops them.
 */
export function useShownStops(stops: string[]) {
  // While a changed selection loads, the previous answer still knows the hidden stops.
  const { data } = useQuery({ ...clusterQueries.stops(stops), placeholderData: keepPreviousData })
  const resolved = new Set(data?.clusters.map(({ id }) => id))
  const hidden = new Set(data?.stops.filter((id) => !resolved.has(id)))
  const shown = stops.filter((id) => !hidden.has(id))

  return { shown, keepUnshown: (next: string[]) => keepUnresolvedStops(next, stops, shown) }
}
