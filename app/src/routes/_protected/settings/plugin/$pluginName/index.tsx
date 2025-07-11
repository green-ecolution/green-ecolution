import { createFileRoute } from '@tanstack/react-router'
import { lazy, Suspense, useMemo } from 'react'
import { PluginProvider } from '@green-ecolution/plugin-interface'
import useStore from '@/store/store'
import { init, loadRemote, registerRemotes } from '@module-federation/enhanced/runtime'
import { basePath } from '@/api/backendApi'

export const Route = createFileRoute('/_protected/settings/plugin/$pluginName/')({
  component: PluginView,
})

init({
  name: 'app',
  remotes: [],
})

function PluginView() {
  const pluginName = Route.useParams().pluginName
  const { authToken } = useStore((state) => ({
    authToken: state.auth.token,
  }))

  const Plugin = useMemo(
    () =>
      lazy(async () => {
        const { url, name } = {
          url: `${basePath}/v1/plugin/${pluginName}/plugin.js`,
          name: pluginName,
        }

        registerRemotes(
          [
            {
              name,
              entry: url,
              type: 'module',
            },
          ],
          { force: true },
        )

        // eslint-disable-next-line @typescript-eslint/no-explicit-any -- this is a dynamic import
        const remote = loadRemote<{ default: any }>(`${name}/app`, {
          from: 'runtime',
          // eslint-disable-next-line @typescript-eslint/no-explicit-any -- this is a dynamic import
        }) as Promise<{ default: any }>

        return remote
      }),
    [pluginName],
  )

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <PluginProvider authToken={authToken?.accessToken ?? ''}>
        <Plugin />
      </PluginProvider>
    </Suspense>
  )
}
