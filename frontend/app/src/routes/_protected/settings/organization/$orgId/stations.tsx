import { createFileRoute } from '@tanstack/react-router'
import StationsPage from '@/components/settings/stations/StationsPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/$orgId/stations')(
  guardedRoute(['start_point:read'], {
    component: StationsPage,
    loader: () => ({ crumb: { titleKey: 'settingsStations' as const } }),
  }),
)
