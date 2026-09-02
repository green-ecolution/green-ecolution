import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/watering-plans')(
  guardedRoute(['watering_plan:read'], crumbRoute('wateringPlans')),
)
