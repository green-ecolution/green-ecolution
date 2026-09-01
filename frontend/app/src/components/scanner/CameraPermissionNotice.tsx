import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'

interface CameraPermissionNoticeProps {
  status: 'denied' | 'unsupported' | 'error'
  onRetry?: () => void
}

const CameraPermissionNotice = ({ status, onRetry }: CameraPermissionNoticeProps) => {
  const { t } = useTranslation('common')

  const COPY = {
    denied: {
      variant: 'destructive' as const,
      title: t('scanner.permission.denied.title'),
      description: t('scanner.permission.denied.description'),
    },
    unsupported: {
      variant: 'warning' as const,
      title: t('scanner.permission.unsupported.title'),
      description: t('scanner.permission.unsupported.description'),
    },
    error: {
      variant: 'destructive' as const,
      title: t('scanner.permission.error.title'),
      description: t('scanner.permission.error.description'),
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

export default CameraPermissionNotice
