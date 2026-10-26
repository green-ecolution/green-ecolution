import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster')(
  moduleRoute('irrigation', guardedRoute(['watering_plan:create'], crumbRoute('routeSelect'))),
)
