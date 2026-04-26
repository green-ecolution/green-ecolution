import { userApi } from '@/api/backendApi'
import useStore from '@/store/store'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected')({
  beforeLoad: async ({ location }) => {
    const isAuthenticated = useStore.getState().isAuthenticated
    if (!isAuthenticated) {
      const loginUrl = await userApi
        .loginUser({
          redirectUrl: `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(location.pathname + location.searchStr)}`,
        })
        .then((res: { loginUrl: string }) => res.loginUrl)

      window.location.href = loginUrl
    }
  },
})
