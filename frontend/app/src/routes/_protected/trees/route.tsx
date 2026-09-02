import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/trees')(
  guardedRoute(['tree:read'], crumbRoute('trees')),
)
