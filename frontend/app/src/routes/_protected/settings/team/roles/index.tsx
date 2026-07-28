import { createFileRoute } from '@tanstack/react-router'
import { guardedRoute } from '@/lib/router'

const Roles = () => <p className="text-sm text-dark-600">Rollen</p>

export const Route = createFileRoute('/_protected/settings/team/roles/')(
  guardedRoute(['role:read'], { component: Roles }),
)
