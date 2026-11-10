import { Suspense } from 'react'
import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { ArrowRight } from 'lucide-react'
import { ListCard } from '@green-ecolution/ui'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { cardDescriptionKeys, coreCardsShown } from '@/modules/dashboard'
import { useActiveModules } from '@/modules/useActiveModules'
import PlansStatTile from '@/components/dashboard/PlansStatTile'
import SensorsStatTile from '@/components/dashboard/SensorsStatTile'

export const Route = createFileRoute('/_protected/dashboard')({
  component: Dashboard,
})

interface QuickLink {
  url: string
  headline: string
  description: string
}

function Dashboard() {
  const user = useCurrentUser()
  const { t, i18n } = useTranslation('dashboard')
  const { manifests } = useActiveModules()
  const tiles = manifests.flatMap(({ key, dashboardTiles = [] }) =>
    dashboardTiles.map((Tile, index) => ({ id: `${key}-${index}`, Tile })),
  )
  const descriptionKeys = cardDescriptionKeys(manifests)
  const shown = coreCardsShown(manifests)

  const quickLinks: QuickLink[] = [
    ...(shown.map
      ? [
          {
            url: '/map',
            headline: t('cards.map.headline'),
            description: descriptionKeys.map
              ? i18n.t(descriptionKeys.map)
              : t('cards.map.description'),
          },
        ]
      : []),
    ...(shown.plans
      ? [
          {
            url: '/plans',
            headline: t('cards.wateringPlans.headline'),
            description: descriptionKeys.plans
              ? i18n.t(descriptionKeys.plans)
              : t('cards.wateringPlans.description'),
          },
        ]
      : []),
    {
      url: '/sensors',
      headline: t('cards.sensors.headline'),
      description: t('cards.sensors.description'),
    },
    {
      url: '/settings',
      headline: t('cards.settings.headline'),
      description: t('cards.settings.description'),
    },
    {
      url: '/settings/profile',
      headline: t('cards.profile.headline'),
      description: t('cards.profile.description'),
    },
  ]

  return (
    <div className="container mt-6">
      <h1 className="mb-10 font-lato font-bold text-3xl lg:text-4xl xl:text-5xl">
        {t('welcomeTitle', { name: `${user.firstName} ${user.lastName}` })}
      </h1>

      <section aria-labelledby="dashboard-overview">
        <h2 id="dashboard-overview" className="mb-4 text-sm font-semibold text-dark-800">
          {t('overviewTitle')}
        </h2>
        <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
          {tiles.map(({ id, Tile }) => (
            <Suspense key={id} fallback={null}>
              <Tile />
            </Suspense>
          ))}
          <PlansStatTile />
          <SensorsStatTile />
        </ul>
      </section>

      <section aria-labelledby="dashboard-quick-links" className="mt-10">
        <h2 id="dashboard-quick-links" className="mb-4 text-sm font-semibold text-dark-800">
          {t('quickLinksTitle')}
        </h2>
        <ul className="grid grid-cols-1 gap-3 md:grid-cols-2 lg:grid-cols-3">
          {quickLinks.map((link) => (
            <li key={link.url}>
              <ListCard size="compact" asChild className="group h-full">
                <Link to={link.url}>
                  <span className="flex min-w-0 flex-1 flex-col">
                    <span className="font-lato font-semibold text-dark-900">{link.headline}</span>
                    <span className="text-sm text-dark-600">{link.description}</span>
                  </span>
                  <ArrowRight
                    aria-hidden
                    className="size-4 shrink-0 text-green-dark transition-transform duration-base ease-emphasized group-hover:translate-x-1 motion-reduce:transition-none"
                  />
                </Link>
              </ListCard>
            </li>
          ))}
        </ul>
      </section>
    </div>
  )
}
