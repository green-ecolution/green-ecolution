import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/team')(
  guardedRoute(['user:read'], crumbRoute('Mitarbeitende')),
)
