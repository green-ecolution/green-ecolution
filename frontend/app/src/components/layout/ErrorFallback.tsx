import Lottie from 'lottie-react'
import cableAnimation from '../../animations/cableAnimation.json'
import React, { useCallback, useEffect, useState } from 'react'
import ButtonLink from '../general/links/ButtonLink'
import { MoveRight, RefreshCw } from 'lucide-react'
import { useAuthSession } from '@/lib/auth/authSessionContext'
import { ResponseError } from '@green-ecolution/backend-client'
import { resolveApiError } from '@/lib/apiError'
import { getI18n } from '@/lib/i18n'
import { Button } from '@green-ecolution/ui'

interface ErrorFallbackProps {
  error: Error
  resetErrorBoundary: () => void
}

const ErrorFallback: React.FC<ErrorFallbackProps> = ({ error, resetErrorBoundary }) => {
  // The router can render this as its defaultErrorComponent outside the
  // I18nextProvider, so useTranslation would throw here; getI18n() reads the
  // module-level instance directly.
  const [errorMessage, setErrorMessage] = useState(() => {
    if (error instanceof ResponseError) return getI18n().t('errors:frame.requestFailed')
    const { message } = error
    return message
  })
  const [errorCode] = useState(() => {
    if (error instanceof ResponseError) {
      return error.response.status
    }
  })

  const { isAuthenticated } = useAuthSession()

  const resolveErrorMessage = useCallback(async () => {
    const { message, detail } = await resolveApiError(error)
    if (detail && detail !== message) {
      console.error('Request failed:', detail)
    }
    setErrorMessage(message)
  }, [error])

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- async data fetching pattern
    resolveErrorMessage().catch(() => setErrorMessage(getI18n().t('errors:unknown')))
  }, [resolveErrorMessage])

  return (
    <div className="relative">
      <figure aria-hidden="true" className="absolute top-0 inset-x-0 z-0">
        <Lottie className="h-[50vh]" animationData={cableAnimation} />
      </figure>
      <div className="mt-[45vh] mx-auto max-w-208 sm:mt-[50vh] xl:max-w-screen-lg">
        <section className="my-28 px-4 md:px-6 lg:my-36 xl:my-52">
          <h1 className="font-lato font-bold text-4xl mb-4 lg:mb-6 lg:text-5xl lg:text-center xl:text-6xl">
            Upps, hier ist etwas schief gegangen!
          </h1>
          <p className="lg:text-center">{errorMessage}</p>
          {errorCode && <p className="lg:text-center mb-5">Fehlercode: {errorCode}</p>}
          <div className="flex flex-col gap-y-4 lg:flex-row lg:items-center lg:justify-center lg:gap-x-4">
            <Button onClick={resetErrorBoundary}>
              Zurück
              <RefreshCw />
            </Button>
            {isAuthenticated ? (
              <ButtonLink link={{ to: '/dashboard' }} label="Zum Dashboard" icon={MoveRight} />
            ) : (
              <ButtonLink link={{ to: '/' }} label="Zur Startseite" icon={MoveRight} />
            )}
          </div>
        </section>
      </div>
    </div>
  )
}

export default ErrorFallback
