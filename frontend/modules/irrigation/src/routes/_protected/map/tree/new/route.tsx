import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/map/tree/new')(
  moduleRoute(
    'irrigation',
    guardedRoute(['tree:create'], qualifiedCrumbRoute('irrigation:crumb.treeLocationNew')),
  ),
)
