import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/watering-plans/_formular/new')(
  guardedRoute(['watering_plan:create'], crumbRoute('Neuen Einsatzplan')),
)
