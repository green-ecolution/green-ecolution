import { createFileRoute, redirect } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { permissionsOf, satisfies, UNRESTRICTED } from '@/lib/auth/permissions'

export const Route = createFileRoute('/_protected/settings/team/')({
  beforeLoad: async ({ context: { queryClient } }) => {
    const perms = readAuthBypass()
      ? UNRESTRICTED
      : permissionsOf(await queryClient.ensureQueryData(userQueries.me()))

    throw redirect({
      to: satisfies(perms, ['user:read']) ? '/settings/team/members' : '/settings/team/roles',
    })
  },
})
