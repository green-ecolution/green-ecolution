import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/treecluster')(
  moduleRoute(
    'irrigation',
    guardedRoute(['tree_cluster:read'], qualifiedCrumbRoute('irrigation:crumb.clusters')),
  ),
)
