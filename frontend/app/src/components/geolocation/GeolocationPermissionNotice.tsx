import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'

interface GeolocationPermissionNoticeProps {
  status: 'denied' | 'unsupported' | 'error'
  errorMessage?: string | null
  onRetry?: () => void
}

const GeolocationPermissionNotice = ({
  status,
  errorMessage,
  onRetry,
}: GeolocationPermissionNoticeProps) => {
  const { t } = useTranslation('common')

  const COPY = {
    denied: {
      variant: 'destructive' as const,
      title: t('geo.permission.denied.title'),
      description: t('geo.permission.denied.description'),
    },
    unsupported: {
      variant: 'warning' as const,
      title: t('geo.permission.unsupported.title'),
      description: t('geo.permission.unsupported.description'),
    },
    error: {
      variant: 'destructive' as const,
      title: t('geo.permission.error.title'),
      description: t('geo.permission.error.description'),
    },
  } as const

  const copy = COPY[status]
  const canRetry = (status === 'denied' || status === 'error') && onRetry

  return (
    <div className="flex flex-col gap-3 items-center">
      <Alert variant={copy.variant} className="w-full">
        <div className="flex items-start gap-3">
          <AlertIcon variant={copy.variant} />
          <AlertContent>
            <AlertTitle>{copy.title}</AlertTitle>
            <AlertDescription>{copy.description}</AlertDescription>
            {errorMessage && (
              <AlertDescription className="font-mono text-xs mt-2 opacity-80">
                {errorMessage}
              </AlertDescription>
            )}
          </AlertContent>
        </div>
      </Alert>
      {canRetry && (
        <Button variant="outline" size="sm" onClick={onRetry}>
          {t('actions.retry')}
        </Button>
      )}
    </div>
  )
}

export default GeolocationPermissionNotice
