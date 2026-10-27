import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster')(
  moduleRoute(
    'irrigation',
    guardedRoute(['watering_plan:create'], qualifiedCrumbRoute('irrigation:crumb.routeSelect')),
  ),
)
