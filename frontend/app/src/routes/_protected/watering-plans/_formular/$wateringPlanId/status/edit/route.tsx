import { createFileRoute } from '@tanstack/react-router'
import { planPermissionsFor } from '@/lib/auth/routePermissions'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { moduleManifests } from '@/modules/registry'

export const Route = createFileRoute(
  '/_protected/watering-plans/_formular/$wateringPlanId/status/edit',
)(guardedRoute(planPermissionsFor(moduleManifests, 'update'), crumbRoute('wateringPlanStatusEdit')))
