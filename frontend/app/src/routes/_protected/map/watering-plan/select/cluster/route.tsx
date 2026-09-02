import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster')(
  guardedRoute(['watering_plan:create'], crumbRoute('routeSelect')),
)
