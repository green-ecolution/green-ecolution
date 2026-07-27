import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId/edit')(
  guardedRoute(['vehicle:update'], crumbRoute('Fahrzeug editieren')),
)
