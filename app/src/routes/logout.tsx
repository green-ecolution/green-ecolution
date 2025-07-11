import { userApi } from '@/api/backendApi'
import useStore from '@/store/store'
import { createFileRoute, redirect } from '@tanstack/react-router'

const logout = () => {
  useStore.getState().auth.clear()
  useStore.getState().user.clear()
}

export const Route = createFileRoute('/logout')({
  beforeLoad: async () => {
    const store = useStore.getState()

    if (!store.auth.isAuthenticated) {
      throw redirect({ to: '/', replace: true })
    }

    await userApi
      .v1UserLogoutPost({
        body: {
          refreshToken: store.auth.token?.refreshToken ?? '',
        },
      })
      .then(logout)
      .catch((err) => {
        if (err instanceof Error) {
          console.error(err.message)
          throw new Error(err.message)
        } else {
          console.error('An unknown error occurred', err)
          throw new Error('An unknown error occurred')
        }
      })

    throw redirect({ to: '/', replace: true })
  },
})
