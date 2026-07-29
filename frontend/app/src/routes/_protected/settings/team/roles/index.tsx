import { createFileRoute } from '@tanstack/react-router'
import RolesPage from '@/components/settings/roles/RolesPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/team/roles/')(
  guardedRoute(['role:read'], { component: RolesPage }),
)
