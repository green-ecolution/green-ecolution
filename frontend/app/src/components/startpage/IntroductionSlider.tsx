import { Splide, SplideSlide } from '@splidejs/react-splide'
import { Car, MapPin, PieChart, Route } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import IntroductionCard from '../general/cards/IntroductionCard'
import SensorIcon from '../icons/Sensor'
import TreeIcon from '../icons/Tree'
import { sliderLabels } from '../../lib/sliderTranslations'

const IntroductionSlider = () => {
  const { t } = useTranslation('startpage')
  const { t: tNav } = useTranslation('navigation')
  const facts = [
    {
      id: 1,
      label: t('introSlider.sensorSolution.label'),
      icon: SensorIcon,
      description: t('introSlider.sensorSolution.description'),
    },
    {
      id: 2,
      label: t('introSlider.dataEvaluation.label'),
      icon: PieChart,
      description: t('introSlider.dataEvaluation.description'),
    },
    {
      id: 3,
      label: t('introSlider.youngTrees.label'),
      icon: TreeIcon,
      description: t('introSlider.youngTrees.description'),
    },
    {
      id: 4,
      label: t('introSlider.multiSite.label'),
      icon: MapPin,
      description: t('introSlider.multiSite.description'),
    },
    {
      id: 5,
      label: t('introSlider.planning.label'),
      icon: Car,
      description: t('introSlider.planning.description'),
    },
    {
      id: 6,
      label: t('introSlider.routing.label'),
      icon: Route,
      description: t('introSlider.routing.description'),
    },
  ]

  const breakpoints = {
    640: {
      perPage: 2,
    },
    1024: {
      destroy: true,
    },
  }

  return (
    <section className="container my-20 lg:my-28">
      <div className="rounded-xl bg-green-dark-100 p-6 md:p-10 lg:pb-6">
        <h2 className="font-bold font-lato text-xl mb-6 text-green-dark md:px-2">
          {t('introSlider.heading')}
        </h2>
        <Splide
          options={{
            rewind: true,
            arrows: false,
            i18n: sliderLabels(tNav),
            mediaQuery: 'min',
            gap: '1rem',
            breakpoints: breakpoints,
          }}
          aria-label={t('introSlider.ariaLabel')}
          className="splide--grid-small md:px-2"
        >
          {facts.map((fact) => (
            <SplideSlide key={fact.id} className="pb-10 lg:pb-0">
              <IntroductionCard
                label={fact.label}
                icon={fact.icon}
                description={fact.description}
              />
            </SplideSlide>
          ))}
        </Splide>
      </div>
    </section>
  )
}

export default IntroductionSlider
