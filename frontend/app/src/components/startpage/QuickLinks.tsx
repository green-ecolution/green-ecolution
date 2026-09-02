import { Link } from '@tanstack/react-router'
import { LinkCard, LinkCardTitle, LinkCardDescription, LinkCardFooter } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'

const QuickLinks = () => {
  const { t } = useTranslation('startpage')
  const cards = [
    {
      id: 1,
      url: '/map',
      description: t('quickLinks.map.description'),
      headline: t('quickLinks.map.headline'),
      linkLabel: t('quickLinks.map.linkLabel'),
    },
    {
      id: 2,
      url: '/sensors',
      description: t('quickLinks.sensors.description'),
      headline: t('quickLinks.sensors.headline'),
      linkLabel: t('quickLinks.sensors.linkLabel'),
    },
    {
      id: 3,
      url: '/watering-plans',
      description: t('quickLinks.wateringPlans.description'),
      headline: t('quickLinks.wateringPlans.headline'),
      linkLabel: t('quickLinks.wateringPlans.linkLabel'),
    },
  ]

  return (
    <section className="container border-t border-t-dark-100 pt-10 mt-20 lg:pt-28 lg:mt-28">
      <article className="text-center max-w-screen-lg mx-auto">
        <h2 className="font-bold font-lato text-xl mb-6 lg:text-3xl">{t('quickLinks.heading')}</h2>
        <p className="mb-6 lg:mb-10">{t('quickLinks.intro')}</p>
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
    </section>
  )
}

export default QuickLinks
