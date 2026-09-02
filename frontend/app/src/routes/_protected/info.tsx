import { infoQueries } from '@/api/queries'
import { pendingLoading, prefetch } from '@/lib/router'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@green-ecolution/ui'
import { useSuspenseQuery, useQuery } from '@tanstack/react-query'
import { createFileRoute, Link, useSearch } from '@tanstack/react-router'
import { z } from 'zod'
import { Database, Layers, Monitor, Server } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import DataTabContent from '@/components/info/DataTabContent'
import SystemTabContent from '@/components/info/SystemTabContent'
import SoftwareTabContent from '@/components/info/SoftwareTabContent'
import ServerTabContent from '@/components/info/ServerTabContent'

const tabSchema = z.enum(['system', 'data', 'software', 'server']).catch('system')

export const Route = createFileRoute('/_protected/info')({
  component: Info,
  pendingComponent: pendingLoading({ key: 'info:root.loadingLabel' }),
  validateSearch: z.object({
    tab: tabSchema.default('system'),
  }),
  loader: ({ context: { queryClient } }) => {
    prefetch(queryClient, infoQueries.app(), 'infoQueries.app')
    prefetch(queryClient, infoQueries.services(), 'infoQueries.services')
    return {
      crumb: {
        titleKey: 'info' as const,
      },
    }
  },
})

function Info() {
  const { t } = useTranslation('info')
  const { tab } = useSearch({ from: '/_protected/info' })
  const { data } = useSuspenseQuery(infoQueries.app())
  const { data: servicesData, isLoading: servicesLoading } = useQuery(infoQueries.services())
  const { data: serverData } = useQuery(infoQueries.server())
  const { data: statsData } = useQuery(infoQueries.statistics())

  const totalServices = servicesData?.items.length ?? 0
  const hasServerInfo = serverData?.hostname

  // Fallback to 'system' if tab requires server info but it's not available
  const activeTab = tab === 'server' && !hasServerInfo ? 'system' : tab

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('page.title')}
        </h1>
        <p>{t('page.description')}</p>
      </article>

      <Tabs value={activeTab}>
        <TabsList>
          <TabsTrigger value="system" asChild>
            <Link to="/info" search={{ tab: 'system' }}>
              <Monitor className="size-5" />
              {t('tabs.system')}
            </Link>
          </TabsTrigger>
          <TabsTrigger value="data" asChild>
            <Link to="/info" search={{ tab: 'data' }}>
              <Database className="size-5" />
              {t('tabs.data')}
            </Link>
          </TabsTrigger>
          <TabsTrigger value="software" asChild>
            <Link to="/info" search={{ tab: 'software' }}>
              <Layers className="size-5" />
              {t('tabs.software')}
            </Link>
          </TabsTrigger>
          {hasServerInfo && (
            <TabsTrigger value="server" asChild>
              <Link to="/info" search={{ tab: 'server' }}>
                <Server className="size-5" />
                {t('tabs.server')}
              </Link>
            </TabsTrigger>
          )}
        </TabsList>

        <TabsContent value="system">
          <SystemTabContent
            data={data}
            servicesData={servicesData}
            servicesLoading={servicesLoading}
            serverData={serverData}
            totalServices={totalServices}
          />
        </TabsContent>

        <TabsContent value="software">
          <SoftwareTabContent data={data} />
        </TabsContent>

        {hasServerInfo && serverData && (
          <TabsContent value="server">
            <ServerTabContent serverData={serverData} />
          </TabsContent>
        )}

        <TabsContent value="data">
          <DataTabContent statsData={statsData} />
        </TabsContent>
      </Tabs>
    </div>
  )
}
