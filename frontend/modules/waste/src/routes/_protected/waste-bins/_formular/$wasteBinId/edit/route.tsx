import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/waste-bins/_formular/$wasteBinId/edit')(
  guardedRoute(['waste_bin:update'], qualifiedCrumbRoute('waste:crumb.edit')),
)
