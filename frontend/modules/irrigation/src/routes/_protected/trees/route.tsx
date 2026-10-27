import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/trees')(
  moduleRoute(
    'irrigation',
    guardedRoute(['tree:read'], qualifiedCrumbRoute('irrigation:crumb.trees')),
  ),
)
