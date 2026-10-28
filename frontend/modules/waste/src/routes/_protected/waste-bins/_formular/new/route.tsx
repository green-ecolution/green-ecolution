import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/waste-bins/_formular/new')(
  guardedRoute(['waste_bin:create'], qualifiedCrumbRoute('waste:crumb.new')),
)
