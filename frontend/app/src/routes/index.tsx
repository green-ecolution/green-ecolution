import ButtonLink from '@/components/general/links/ButtonLink'
import IntroductionSlider from '@/components/startpage/IntroductionSlider'
import KeyFacts from '@/components/startpage/KeyFacts'
import { createFileRoute } from '@tanstack/react-router'
import { Mail, MoveRight } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import Lottie from 'lottie-react'
import dashboardAnimation from '../../src/animations/dashboardAnimation.json'
import QuickLinks from '@/components/startpage/QuickLinks'
import useStore from '@/store/store'

export const Route = createFileRoute('/')({
  component: Startpage,
})

function Startpage() {
  const isAuthenticated = useStore((state) => state.isAuthenticated)

  return (
    <>
      <article className="container my-20 lg:my-24 xl:grid xl:grid-cols-2 xl:gap-x-16 xl:items-center">
        <div>
          <p className="text-green-dark mb-4 text-lg font-semibold">Green Ecolution</p>
          <h1 className="font-lato font-bold text-4xl mb-4 lg:text-5xl xl:text-6xl">
            Willkommen beim smarten
            <br />
            Grünflächen&shy;management
          </h1>
          <p>
            Dieses Projekt befasst sich mit der smarten Bewässerung von Bäumen im urbanen Kontext.
            Mittels sensorgestützter Überwachung werden Daten über das LoRaWan-Netz übermittelt und
            ausgewertet, sodass Handlungsempfehlungen für die Bewässerung abgegeben werden können.
            Anhand dieser Handlungsempfehlungen können Einsätze zu einer Bewässerungstour erstellt
            werden.
          </p>
          <div className="flex flex-wrap items-center gap-6 mt-10">
            {isAuthenticated ? (
              <ButtonLink link={{ to: '/dashboard' }} label="Zum Dashboard" icon={MoveRight} />
            ) : (
              <ButtonLink
                link={{ to: '/login', preload: false }}
                label="Anmelden"
                icon={MoveRight}
              />
            )}
            <Button asChild>
              <a href="mailto:info@green-ecolution.de">
                Kontakt
                <Mail />
              </a>
            </Button>
          </div>
        </div>

        <div className="mt-10 max-w-screen-md mx-auto">
          <Lottie animationData={dashboardAnimation} />
        </div>
      </article>

      <QuickLinks />
      <KeyFacts />
      <IntroductionSlider />
    </>
  )
}
