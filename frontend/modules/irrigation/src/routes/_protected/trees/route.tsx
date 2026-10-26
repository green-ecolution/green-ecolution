import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'
import { moduleRoute } from '@/modules/moduleRoute'

export const Route = createFileRoute('/_protected/trees')(
  moduleRoute('irrigation', guardedRoute(['tree:read'], crumbRoute('trees'))),
)
