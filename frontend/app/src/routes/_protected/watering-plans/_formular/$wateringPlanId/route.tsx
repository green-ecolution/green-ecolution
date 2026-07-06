import { wateringPlanEntityRoute } from '@/routes/_protected/watering-plans/$wateringPlanId/route'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId')({
  ...wateringPlanEntityRoute,
})
