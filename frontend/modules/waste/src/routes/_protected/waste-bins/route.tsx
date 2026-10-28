import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/waste-bins')(
  moduleRoute('waste', guardedRoute(['waste_bin:read'], qualifiedCrumbRoute('waste:crumb.bins'))),
)
