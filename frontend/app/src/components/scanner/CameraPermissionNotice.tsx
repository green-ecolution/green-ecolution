import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
} from '@green-ecolution/ui'

interface CameraPermissionNoticeProps {
  status: 'denied' | 'unsupported' | 'error'
  onRetry?: () => void
}

const COPY = {
  denied: {
    variant: 'destructive' as const,
    title: 'Kamerazugriff verweigert',
    description:
      'Um QR-Codes zu scannen, benötigt die App Zugriff auf deine Kamera. Erlaube den Zugriff in den Einstellungen deines Browsers und versuche es erneut.',
  },
  unsupported: {
    variant: 'warning' as const,
    title: 'Kamera nicht verfügbar',
    description:
      'Dein Browser oder Gerät unterstützt den Kamerazugriff in diesem Kontext nicht. Stelle sicher, dass du die App über HTTPS aufrufst.',
  },
  error: {
    variant: 'destructive' as const,
    title: 'Kamera konnte nicht gestartet werden',
    description:
      'Beim Zugriff auf die Kamera ist ein Fehler aufgetreten. Prüfe, ob eine andere Anwendung die Kamera bereits nutzt.',
  },
} as const

const CameraPermissionNotice = ({ status, onRetry }: CameraPermissionNoticeProps) => {
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
          Erneut versuchen
        </Button>
      )}
    </div>
  )
}

export default CameraPermissionNotice
