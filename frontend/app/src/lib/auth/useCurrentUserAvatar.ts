import { useQuery } from '@tanstack/react-query'
import { userQueries } from '@/api/queries'
import { useAuthSession } from './authSessionContext'

/**
 * Avatar URL of the logged-in user. Unlike the other profile fields this is
 * not part of the JWT (it lives in the app-owned user profile), so it has to
 * be fetched from the API.
 */
export function useCurrentUserAvatar(): string | undefined {
  const { isAuthenticated } = useAuthSession()
  const { data } = useQuery({ ...userQueries.me(), enabled: isAuthenticated })
  // the API sends an empty string when no avatar is set
  const url = data?.avatarUrl
  return url === '' ? undefined : url
}
