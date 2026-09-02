import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/map/tree/new')(
  guardedRoute(['tree:create'], crumbRoute('treeLocationNew')),
)
