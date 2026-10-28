import { createFileRoute, redirect } from '@tanstack/react-router'
import { ensureEffectiveRoles } from '@/lib/auth/scope/effectiveRoles'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { permissionsOf, satisfies, UNRESTRICTED } from '@/lib/auth/permissions'

export const Route = createFileRoute('/_protected/settings/team/')({
  beforeLoad: async ({ context: { queryClient } }) => {
    const perms = readAuthBypass()
      ? UNRESTRICTED
      : permissionsOf(await ensureEffectiveRoles(queryClient))

    throw redirect({
      to: satisfies(perms, ['user:read']) ? '/settings/team/members' : '/settings/team/roles',
    })
  },
})
