import { useQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import { useAuthSession } from './authSessionContext'
import { readAuthBypass } from './runtimeConfig'
import { permissionsOf, UNRESTRICTED, type Permissions } from './permissions'

/**
 * Effective permissions of the logged-in user. With auth bypassed the backend
 * grants unrestricted access, so the frontend must not hide anything.
 */
export function usePermissions(): Permissions {
  const { isAuthenticated } = useAuthSession()
  const bypass = readAuthBypass()
  const { data } = useQuery({ ...userQueries.me(), enabled: isAuthenticated && !bypass })

  return bypass ? UNRESTRICTED : permissionsOf(data)
}
