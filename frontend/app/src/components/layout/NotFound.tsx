import Lottie from 'lottie-react'
import cableAnimation from '../../animations/cableAnimation.json'
import ButtonLink from '../general/links/ButtonLink'
import { MoveRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useAuthSession } from '@/lib/auth/authSessionContext'

function NotFound() {
  const { isAuthenticated } = useAuthSession()
  const { t } = useTranslation('navigation')

  return (
    <>
      <figure aria-hidden="true">
        <Lottie className="h-[40vh] max-h-96" animationData={cableAnimation} />
      </figure>
      <div className="mx-auto max-w-208 xl:max-w-screen-lg">
        <section className="mb-28 px-4 md:px-6 lg:mb-36">
          <h1 className="font-lato font-bold text-4xl mb-4 lg:mb-6 lg:text-5xl lg:text-center xl:text-6xl">
            {t('notFound.title')}
          </h1>
          <p className="lg:text-center mb-10">{t('notFound.description')}</p>
          <div className="lg:flex lg:items-center lg:justify-center">
            {isAuthenticated ? (
              <ButtonLink
                link={{ to: '/dashboard' }}
                label={t('notFound.toDashboard')}
                icon={MoveRight}
              />
            ) : (
              <ButtonLink link={{ to: '/' }} label={t('notFound.toHome')} icon={MoveRight} />
            )}
          </div>
        </section>
      </div>
    </>
  )
}

export default NotFound
