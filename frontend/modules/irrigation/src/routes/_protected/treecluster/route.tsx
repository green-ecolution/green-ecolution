import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/treecluster')(
  guardedRoute(['tree_cluster:read'], crumbRoute('clusters')),
)
