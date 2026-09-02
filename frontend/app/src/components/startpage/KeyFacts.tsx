import { BadgeCheck } from 'lucide-react'
import { useTranslation } from 'react-i18next'

const KeyFacts = () => {
  const { t } = useTranslation('startpage')
  const numbers = [
    t('keyFacts.monitoring'),
    t('keyFacts.grouping'),
    t('keyFacts.interpretation'),
    t('keyFacts.planning'),
    t('keyFacts.routing'),
    t('keyFacts.evaluation'),
  ]
  return (
    <section className="container mt-20 lg:mt-28">
      <h2 className="font-semibold text-dark-800 mb-6">{t('keyFacts.heading')}</h2>
      <ul className="grid grid-cols-1 gap-y-4 md:grid-cols-2 md:gap-x-10">
        {numbers.map((number) => (
          <li key={number} className="flex items-center gap-x-4">
            <BadgeCheck className="text-green-light w-8 h-8 shrink-0" />
            <p className="font-semibold text-lg">{number}</p>
          </li>
        ))}
      </ul>
    </section>
  )
}

export default KeyFacts
