import { createFileRoute, Link } from '@tanstack/react-router'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { servicesInfoQuery } from '@/api/queries'

export const Route = createFileRoute('/_protected/settings/')({
  component: Settings,
})

interface SettingsCard {
  id: number
  url: string
  description: string
  headline: string
  linkLabel: string
  featureKey?: string
}

const allCards: SettingsCard[] = [
  {
    id: 1,
    url: '/settings/plugin',
    description: 'Alle Plugins, die in der Anwendung installiert sind.',
    headline: 'Plugins',
    linkLabel: 'Zu den Plugins',
    featureKey: 'plugins',
  },
]

function Settings() {
  const { data: services } = useQuery(servicesInfoQuery())

  const cards = allCards.filter((card) => {
    if (!card.featureKey) return true
    const entry = services?.items.find((item) => item.name === card.featureKey)
    return entry?.enabled === true
  })

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Ihre persönlichen Einstellungen
        </h1>
        <p>
          In diesem Bereich können Sie die Systemeinstellungen einsehen. Dazu gehört unter anderem
          eine Übersicht darüber, welche Plugins bzw. Erweiterungen für das System freigeschaltet
          sind.
        </p>
      </article>

      {cards.length > 0 && (
        <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
          {cards.map((card, key) => (
            <li key={card.id}>
              <LinkCard variant={key % 2 ? 'dark' : 'light'} asChild>
                <Link to={card.url} aria-label={card.linkLabel}>
                  <LinkCardTitle>{card.headline}</LinkCardTitle>
                  <LinkCardDescription>{card.description}</LinkCardDescription>
                  <LinkCardFooter>{card.linkLabel}</LinkCardFooter>
                </Link>
              </LinkCard>
            </li>
          ))}
        </ul>
      )}
    </div>
  )
}
