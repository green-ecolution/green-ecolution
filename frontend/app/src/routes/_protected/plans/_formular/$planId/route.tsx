import { planEntityRoute } from '@/routes/_protected/plans/$planId/route'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/plans/_formular/$planId')({
  ...planEntityRoute,
})
