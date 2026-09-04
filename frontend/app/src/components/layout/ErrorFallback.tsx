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

  // Same reasoning as above: the surrounding chrome uses getI18n() rather
  // than useTranslation, so it keeps working on the same outside-the-provider
  // path the error message itself relies on.
  const t = getI18n().t

  return (
    <div>
      <figure aria-hidden="true">
        <Lottie className="h-[40vh] max-h-96" animationData={cableAnimation} />
      </figure>
      <div className="mx-auto max-w-208 xl:max-w-screen-lg">
        <section className="mb-28 px-4 md:px-6 lg:mb-36">
          <h1 className="font-lato font-bold text-4xl mb-4 lg:mb-6 lg:text-5xl lg:text-center xl:text-6xl">
            {t('navigation:errorFallback.title')}
          </h1>
          <p className="lg:text-center">{errorMessage}</p>
          {errorCode && (
            <p className="lg:text-center mb-5">
              {t('navigation:errorFallback.errorCode', { code: errorCode })}
            </p>
          )}
          <div className="flex flex-col gap-y-4 lg:flex-row lg:items-center lg:justify-center lg:gap-x-4">
            <Button onClick={resetErrorBoundary}>
              {t('navigation:errorFallback.back')}
              <RefreshCw />
            </Button>
            {isAuthenticated ? (
              <ButtonLink
                link={{ to: '/dashboard' }}
                label={t('navigation:errorFallback.toDashboard')}
                icon={MoveRight}
              />
            ) : (
              <ButtonLink
                link={{ to: '/' }}
                label={t('navigation:errorFallback.toHome')}
                icon={MoveRight}
              />
            )}
          </div>
        </section>
      </div>
    </div>
  )
}

export default ErrorFallback
