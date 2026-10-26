import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/map/tree/new')(
  moduleRoute('irrigation', guardedRoute(['tree:create'], crumbRoute('treeLocationNew'))),
)
