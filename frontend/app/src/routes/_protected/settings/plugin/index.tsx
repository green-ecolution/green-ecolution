import { createFileRoute, Link } from '@tanstack/react-router'
import { Suspense } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { pluginsQuery } from '@/api/queries'
import type { PluginResponse } from '@/api/backendApi'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/settings/plugin/')({
  component: PluginView,
})

function PluginView() {
  const { t } = useTranslation('settings')

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('plugin.overviewTitle')}
        </h1>
        <p>{t('plugin.overviewIntro')}</p>
      </article>

      <Suspense fallback={<div>{t('plugin.loading')}</div>}>
        <PluginList />
      </Suspense>
    </div>
  )
}

const PluginList = () => {
  const { t } = useTranslation('settings')
  const { data: pluginList } = useQuery(pluginsQuery())

  return (
    <>
      <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
        {pluginList?.data.map((plugin: PluginResponse, key: number) => (
          <li key={plugin.slug}>
            <LinkCard variant={key % 2 ? 'dark' : 'light'} asChild>
              <Link
                to="/settings/plugin/$pluginName"
                params={{ pluginName: plugin.slug }}
                aria-label={t('plugin.startAriaLabel', { name: plugin.name })}
              >
                <LinkCardTitle>{plugin.name}</LinkCardTitle>
                <LinkCardDescription>{plugin.description}</LinkCardDescription>
                <LinkCardFooter>{t('plugin.startLabel', { name: plugin.name })}</LinkCardFooter>
              </Link>
            </LinkCard>
          </li>
        ))}
      </ul>

      {!pluginList ||
        (pluginList.data.length === 0 && (
          <div className="text-center mt-6">
            <p className="text-dark-500">{t('plugin.empty')}</p>
          </div>
        ))}
    </>
  )
}
