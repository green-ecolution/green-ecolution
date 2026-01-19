import { createFileRoute, Link } from '@tanstack/react-router'
import { useUserStore } from '@/store/store'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'

export const Route = createFileRoute('/_protected/dashboard')({
  component: Dashboard,
})

function Dashboard() {
  const user = useUserStore()

  const cards = [
    {
      id: 1,
      url: '/map',
      description: 'Alle Bäume in Flensburg im Zuständigkeitsbereich des TBZ.',
      headline: 'Karte',
      linkLabel: 'Zur Karte',
    },
    {
      id: 2,
      url: '/treecluster',
      description: 'Listenansicht aller gruppierten Bäume, die mit Sensoren ausgestattet sind.',
      headline: 'Auflistung der Bewässerungsgruppen',
      linkLabel: 'Zu den Bewässerungsgruppen',
    },
    {
      id: 3,
      url: '/sensors',
      description: 'Zeigt alle verbauten Sensoren in Flensburg inkl. Akkustand, Standort,…',
      headline: 'Liste aller verbauten Sensoren',
      linkLabel: 'Zu den Sensoren',
    },
    {
      id: 4,
      url: '/watering-plans',
      description: 'Planung eines neuen Einsatzes zur Bewässerung von Baumgruppen',
      headline: 'Einsatzplanung',
      linkLabel: 'Zur Einsatzplanung',
    },
    {
      id: 5,
      url: '/settings',
      description: 'Hier können Sie Einstellungen vornehmen, da Sie Administrator sind.',
      headline: 'Einstellungen',
      linkLabel: 'Zu den Einstellungen',
    },
    {
      id: 6,
      url: '/profile',
      description: 'Hier können persönliche Informationen hinterlegt und angepasst werden.',
      headline: 'Eigenes Profil',
      linkLabel: 'Zum Profil',
    },
  ]

  return (
    <div className="container mt-6">
      <article className="mb-10 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Willkommen zurück, {`${user.firstName} ${user.lastName}`}!
        </h1>
        <p>
          Sie befinden sich auf dem Dashboard. Dies ist eine Übersichtsseite, um direkten Zugriff
          auf wichtige Bereiche zu erhalten.
        </p>
      </article>

      <h2 className="text-sm font-semibold text-dark-800 mb-4">Schnellverweise</h2>

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
