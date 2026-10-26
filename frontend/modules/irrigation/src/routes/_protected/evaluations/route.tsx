import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'
import { ANY_READ } from '@/lib/auth/permissions'

export const Route = createFileRoute('/_protected/evaluations')(
  moduleRoute('irrigation', guardedRoute(ANY_READ, crumbRoute('evaluations'))),
)
