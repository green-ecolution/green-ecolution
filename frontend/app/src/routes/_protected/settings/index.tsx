import { createFileRoute, Link } from '@tanstack/react-router'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/settings/')({
  component: Settings,
})

const cards = [
  {
    id: 1,
    url: '/settings/plugin',
    description: 'Alle Plugins, die in der Anwendung installiert sind.',
    headline: 'Plugins',
    linkLabel: 'Zu den Plugins',
  },
]

function Settings() {
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
    </div>
  )
}
