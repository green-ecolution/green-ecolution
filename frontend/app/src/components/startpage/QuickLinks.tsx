import DashboardCard from '../general/cards/DashboardCard'

const QuickLinks = () => {
  const cards = [
    {
      id: 1,
      url: '/map',
      description: 'Alle Bewässerungsgruppen & Bäume auf der Karte anzeigen lassen',
      headline: 'Karte inkl. Verortung aller Bäume',
      linkLabel: 'Zur Karte',
    },
    {
      id: 2,
      url: '/sensors',
      description: 'Zeigt alle verbauten Sensoren inkl. Akkustand, Messwerte, …',
      headline: 'Liste aller verbauten Sensoren',
      linkLabel: 'Zu den Sensoren',
    },
    {
      id: 3,
      url: '/watering-plans',
      description:
        'Listenansicht, aller Einsatzpläne, die geplant, abgeschlossen, aktiv etc. sind.',
      headline: 'Auflistung der Einsatzpläne & Routenplanung',
      linkLabel: 'Zu den Einsatzplänen',
    },
  ]

  return (
    <section className="container border-t border-t-dark-100 pt-10 mt-20 lg:pt-28 lg:mt-28">
      <article className="text-center max-w-screen-lg mx-auto">
        <h2 className="font-bold font-lato text-xl mb-6 lg:text-3xl">
          Ansichten für eine effiziente Bewässerungskontrolle
        </h2>
        <p className="mb-6 lg:mb-10">
          Unser System bietet verschiedene Ansichten, um den Bewässerungszustand der Bäume zu
          überwachen. Sie können alle Bäume auf einer Karte anzeigen, die verbauten Sensoren mit
          Akkustand und Messwerten einsehen und eine Übersicht der Einsatzpläne bisheriger und
          zukünftiger Bewässerungsfahrten abrufen.
        </p>
      </article>
      <ul className="grid grid-cols-1 gap-5 md:grid-cols-2 lg:grid-cols-3">
        {cards.map((card, key) => (
          <li key={card.id}>
            <DashboardCard
              headline={card.headline}
              description={card.description}
              linkLabel={card.linkLabel}
              url={card.url}
              theme={key % 2 ? 'dark' : 'light'}
            />
          </li>
        ))}
      </ul>
    </section>
  )
}

export default QuickLinks
