import { useQuery, useQueryClient } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '../runtimeConfig'
import { DEMO_USER_ID, setOrganizationScope } from './organizationScopeStore'

export function useSwitchOrganizationScope(): (scope: string | null) => Promise<void> {
  const queryClient = useQueryClient()
  const router = useRouter()
  const bypass = readAuthBypass()
  const { data: me } = useQuery({ ...userQueries.me(), enabled: !bypass })
  const userId = bypass ? DEMO_USER_ID : me?.id

  return async (scope) => {
    if (!userId) return
    setOrganizationScope(userId, scope)
    // Route guards and cached data were resolved under the previous scope.
    queryClient.clear()
    await router.invalidate()
  }
}
