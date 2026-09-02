import ButtonLink from '@/components/general/links/ButtonLink'
import IntroductionSlider from '@/components/startpage/IntroductionSlider'
import KeyFacts from '@/components/startpage/KeyFacts'
import { createFileRoute } from '@tanstack/react-router'
import { Mail, MoveRight } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import Lottie from 'lottie-react'
import dashboardAnimation from '../../src/animations/dashboardAnimation.json'
import QuickLinks from '@/components/startpage/QuickLinks'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { useTranslation } from 'react-i18next'

export const Route = createFileRoute('/')({
  component: Startpage,
})

function Startpage() {
  const { isAuthenticated } = useAuthSession()
  const { t } = useTranslation('startpage')

  return (
    <>
      <article className="container my-20 lg:my-24 xl:grid xl:grid-cols-2 xl:gap-x-16 xl:items-center">
        <div>
          {/* eslint-disable-next-line i18next/no-literal-string -- product name, not translated */}
          <p className="text-green-dark mb-4 text-lg font-semibold">Green Ecolution</p>
          <h1 className="font-lato font-bold text-4xl mb-4 lg:text-5xl xl:text-6xl">
            {t('hero.titleLine1')}
            <br />
            {t('hero.titleLine2')}
          </h1>
          <p>{t('hero.intro')}</p>
          <div className="flex flex-wrap items-center gap-6 mt-10">
            {isAuthenticated ? (
              <ButtonLink
                link={{ to: '/dashboard' }}
                label={t('hero.ctaDashboard')}
                icon={MoveRight}
              />
            ) : (
              <ButtonLink
                link={{ to: '/login', preload: false }}
                label={t('hero.ctaLogin')}
                icon={MoveRight}
              />
            )}
            <Button asChild>
              <a href="mailto:info@green-ecolution.de">
                {t('hero.ctaContact')}
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
