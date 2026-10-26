import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute, qualifiedCrumbRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'
import { EVALUATION_READ } from '../../../resources'

export const Route = createFileRoute('/_protected/evaluations')(
  moduleRoute(
    'irrigation',
    guardedRoute(EVALUATION_READ, qualifiedCrumbRoute('irrigation:crumb.evaluations')),
  ),
)
