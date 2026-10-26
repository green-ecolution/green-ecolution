import { Fragment } from 'react'
import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'
import { useActiveModules } from '@/modules/useActiveModules'

export const Route = createFileRoute('/_protected/dashboard')({
  component: Dashboard,
})

function Dashboard() {
  const user = useCurrentUser()
  const { t } = useTranslation('dashboard')
  const { manifests } = useActiveModules()
  const tiles = manifests.flatMap((manifest) => manifest.dashboardTiles ?? [])

  const cards = [
    {
      id: 1,
      url: '/map',
      description: t('cards.map.description'),
      headline: t('cards.map.headline'),
      linkLabel: t('cards.map.linkLabel'),
    },
    {
      id: 2,
      url: '/sensors',
      description: t('cards.sensors.description'),
      headline: t('cards.sensors.headline'),
      linkLabel: t('cards.sensors.linkLabel'),
    },
    {
      id: 3,
      url: '/watering-plans',
      description: t('cards.wateringPlans.description'),
      headline: t('cards.wateringPlans.headline'),
      linkLabel: t('cards.wateringPlans.linkLabel'),
    },
    {
      id: 4,
      url: '/settings',
      description: t('cards.settings.description'),
      headline: t('cards.settings.headline'),
      linkLabel: t('cards.settings.linkLabel'),
    },
    {
      id: 5,
      url: '/settings/profile',
      description: t('cards.profile.description'),
      headline: t('cards.profile.headline'),
      linkLabel: t('cards.profile.linkLabel'),
    },
  ]

  // Module tiles slot in after the map card, where the irrigation card sat before.
  const [mapCard, ...otherCards] = cards
  const cardEntry = (card: DashboardCardProps['card']) => ({
    id: card.url,
    render: (variant: DashboardCardProps['variant']) => (
      <DashboardCard card={card} variant={variant} />
    ),
  })
  const entries = [
    cardEntry(mapCard),
    ...tiles.map((Tile, index) => ({
      id: `tile-${index}`,
      render: (variant: DashboardCardProps['variant']) => <Tile variant={variant} />,
    })),
    ...otherCards.map(cardEntry),
  ]

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          {t('welcomeTitle', { name: `${user.firstName} ${user.lastName}` })}
        </h1>
        <p>{t('welcomeDescription')}</p>
      </article>

      <h2 className="text-sm font-semibold text-dark-800 mb-4">{t('quickLinksTitle')}</h2>

      <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
        {entries.map((entry, position) => (
          <Fragment key={entry.id}>{entry.render(position % 2 ? 'dark' : 'light')}</Fragment>
        ))}
      </ul>
    </div>
  )
}

interface DashboardCardProps {
  card: { url: string; headline: string; description: string; linkLabel: string }
  variant: 'light' | 'dark'
}

function DashboardCard({ card, variant }: DashboardCardProps) {
  return (
    <li>
      <LinkCard variant={variant} asChild>
        <Link to={card.url} aria-label={card.linkLabel}>
          <LinkCardTitle>{card.headline}</LinkCardTitle>
          <LinkCardDescription>{card.description}</LinkCardDescription>
          <LinkCardFooter>{card.linkLabel}</LinkCardFooter>
        </Link>
      </LinkCard>
    </li>
  )
}
