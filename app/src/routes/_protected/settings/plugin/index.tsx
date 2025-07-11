import { createFileRoute } from '@tanstack/react-router'
import { Suspense } from 'react'
import { useQuery } from '@tanstack/react-query'
import { pluginApi } from '@/api/backendApi'
import DashboardCard from '@/components/general/cards/DashboardCard'

export const Route = createFileRoute('/_protected/settings/plugin/')({
  component: PluginView,
})

function PluginView() {
  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Übersicht der Plugins
        </h1>
        <p>
          Hier finden Sie eine Übersicht aller installierten Plugins. Diese Plugins erweitern die
          Funktionalität der Anwendung und können eine Vielzahl nützlicher Features bieten. Klicken
          Sie auf eines der Plugins, um mehr darüber zu erfahren.
        </p>
      </article>

      <Suspense fallback={<div>Laden von Plugins...</div>}>
        <PluginList />
      </Suspense>
    </div>
  )
}

const PluginList = () => {
  const { data: pluginList } = useQuery({
    queryKey: ['pluginList'],
    queryFn: () => pluginApi.getPluginsList(),
  })

  return (
    <>
      <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
        {pluginList?.plugins.map((plugin, key) => (
          <li key={plugin.slug}>
            <DashboardCard
              headline={plugin.name}
              description={plugin.description}
              linkLabel={`${plugin.name} starten`}
              url={`/settings/plugin/${plugin.slug}`}
              theme={key % 2 ? 'dark' : 'light'}
            />
          </li>
        ))}
      </ul>

      {!pluginList ||
        (pluginList.plugins.length === 0 && (
          <div className="text-center mt-6">
            <p className="text-dark-500">Zur Zeit sind keine Plugins registriert.</p>
          </div>
        ))}
    </>
  )
}
