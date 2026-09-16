import { createFileRoute } from '@tanstack/react-router'
import OrganizationPage from '@/components/settings/organization/OrganizationPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/')(
  guardedRoute(['organization:read'], { component: OrganizationPage }),
)
