import { userApi } from '@/api/backendApi'
import useAuthStore from '@/store/store'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected')({
  beforeLoad: async ({ location }) => {
    const isAuthenticated = useAuthStore.getState().auth.isAuthenticated
    if (!isAuthenticated) {
      const loginUrl = await userApi
        .v1UserLoginGet({
          redirectUrl: `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(location.pathname + location.searchStr)}`,
        })
        .then((res) => res.loginUrl)

      window.location.href = loginUrl
    }
  },
})
