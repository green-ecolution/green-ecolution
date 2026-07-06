import { createFileRoute } from '@tanstack/react-router'
import { crumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/vehicles/_formular/new')(
  crumbRoute('Neues Fahrzeug'),
)
