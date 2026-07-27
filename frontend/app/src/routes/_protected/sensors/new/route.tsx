import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/sensors/new')(
  guardedRoute(['sensor:create'], crumbRoute('Sensor aktivieren')),
)
