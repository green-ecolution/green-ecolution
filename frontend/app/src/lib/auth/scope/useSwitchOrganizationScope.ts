import { hashKey, useQuery, useQueryClient } from '@tanstack/react-query'
import { useRouter } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '../runtimeConfig'
import { DEMO_USER_ID, setOrganizationScope } from './organizationScopeStore'

// The backend answers these without the scope; resetting them would blank the
// user menu and navigation mid-switch.
const scopeIndependentHashes = () =>
  new Set([hashKey(userQueries.me().queryKey), hashKey(userQueries.myOrganizations().queryKey)])

export function useSwitchOrganizationScope(): (scope: string | null) => Promise<void> {
  const queryClient = useQueryClient()
  const router = useRouter()
  const bypass = readAuthBypass()
  const { data: me } = useQuery({ ...userQueries.me(), enabled: !bypass })
  const userId = bypass ? DEMO_USER_ID : me?.id

  return async (scope) => {
    if (!userId) return
    setOrganizationScope(userId, scope)
    // Unlike clear(), a reset notifies mounted observers, so components that
    // do not re-render on their own still drop the previous scope's data.
    const keep = scopeIndependentHashes()
    await queryClient.resetQueries({ predicate: (query) => !keep.has(query.queryHash) })
    await router.invalidate()
  }
}
