import { userApi } from '@/api/backendApi'
import App from '@/App'
import useStore from '@/store/store'
import { QueryClient } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { createRootRouteWithContext } from '@tanstack/react-router'
import React from 'react'

const TanStackRouterDevtools =
  process.env.NODE_ENV === 'production'
    ? () => null
    : React.lazy(() =>
        import('@tanstack/react-router-devtools').then((res) => ({
          default: res.TanStackRouterDevtools,
        })),
      )

interface RouterContext {
  queryClient: QueryClient
}

export const Route = createRootRouteWithContext<RouterContext>()({
  component: Root,
  beforeLoad: async () => {
    const state = useStore.getState()
    if (!state.isAuthenticated || !state.isUserEmpty()) {
      return
    }
    const token = await userApi.v1UserTokenRefreshPost({
      body: {
        refreshToken: state.token?.refreshToken ?? '',
      },
    })
    if (!token) {
      return
    }
    useStore.getState().setToken(token)
    useStore.getState().setUserFromJwt(token.accessToken)
  },
})

function Root() {
  return (
    <>
      <ReactQueryDevtools initialIsOpen={false} position="bottom" />
      <TanStackRouterDevtools initialIsOpen={false} position="bottom-right" />
      <App />
    </>
  )
}
