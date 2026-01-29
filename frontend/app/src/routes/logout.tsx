import { userApi } from '@/api/backendApi'
import useStore from '@/store/store'
import { createFileRoute, redirect } from '@tanstack/react-router'

const logout = () => {
  useStore.getState().clearAuth()
  useStore.getState().clearUser()
}

export const Route = createFileRoute('/logout')({
  beforeLoad: async () => {
    const state = useStore.getState()

    if (!state.isAuthenticated) {
      throw redirect({ to: '/', replace: true })
    }

    await userApi
      .logout({
        body: {
          refreshToken: state.token?.refreshToken ?? '',
        },
      })
      .then(logout)
      .catch((err: unknown) => {
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
