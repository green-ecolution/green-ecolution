import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { ANY_READ } from '@/lib/auth/permissions'

export const Route = createFileRoute('/_protected/evaluations')(
  guardedRoute(ANY_READ, crumbRoute('Auswertungen')),
)
