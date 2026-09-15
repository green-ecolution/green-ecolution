import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'
import OrganizationPage from '@/components/settings/organization/OrganizationPage'
import { guardedRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/settings/organization/')(
  guardedRoute(['organization:read'], {
    component: OrganizationPage,
    validateSearch: z.object({ org: z.string().optional() }),
  }),
)
