import { wateringPlanEntityRoute } from '@/routes/_protected/plans/$wateringPlanId/route'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/plans/_formular/$wateringPlanId')({
  ...wateringPlanEntityRoute,
})
