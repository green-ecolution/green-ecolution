import { createFileRoute } from '@tanstack/react-router'
import { MapOverviews } from '@/components/map/MapModuleSlots'

export const Route = createFileRoute('/_protected/map/')({
  component: MapOverviews,
})
