import Lottie from 'lottie-react'
import cableAnimation from '../../animations/cableAnimation.json'
import ButtonLink from '../general/links/ButtonLink'
import { MoveRight } from 'lucide-react'
import useStore from '@/store/store'

function NotFound() {
  const { isAuthenticated } = useStore((state) => ({
    isAuthenticated: state.auth.isAuthenticated,
  }))

  return (
    <>
      <figure aria-hidden="true" className="absolute top-0 inset-x-0 z-0">
        <Lottie className="h-[50vh]" animationData={cableAnimation} />
      </figure>
      <div className="mt-[45vh] mx-auto max-w-208 sm:mt-[50vh] xl:max-w-screen-lg">
        <section className="my-28 px-4 md:px-6 lg:my-36 xl:my-52">
          <h1 className="font-lato font-bold text-4xl mb-4 lg:mb-6 lg:text-5xl lg:text-center xl:text-6xl">
            Die Seite konnte nicht gefunden werden.
          </h1>
          <p className="lg:text-center mb-10">
            Die gewünschte Seite ist nicht erreichbar, da sie entweder nicht existiert oder es zu
            einem Fehler gekommen ist. Error-Code: 404
          </p>
          <div className="lg:flex lg:items-center lg:justify-center">
            {isAuthenticated ? (
              <ButtonLink link={{ to: '/dashboard' }} label="Zum Dashboard" icon={MoveRight} />
            ) : (
              <ButtonLink link={{ to: '/' }} label="Zur Startseite" icon={MoveRight} />
            )}
          </div>
        </section>
      </div>
    </>
  )
}

export default NotFound
