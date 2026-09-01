import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute, guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/vehicles/_formular/new')(
  guardedRoute(['vehicle:create'], crumbRoute('vehicleNew')),
)
