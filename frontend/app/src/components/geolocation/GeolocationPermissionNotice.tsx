import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
} from '@green-ecolution/ui'

interface GeolocationPermissionNoticeProps {
  status: 'denied' | 'unsupported' | 'error'
  errorMessage?: string | null
  onRetry?: () => void
}

const COPY = {
  denied: {
    variant: 'destructive' as const,
    title: 'Standortzugriff verweigert',
    description:
      'Um den Sensorstandort automatisch zu erfassen, benötigt die App Zugriff auf deinen GPS-Standort. Erlaube den Zugriff in den Einstellungen deines Browsers und versuche es erneut.',
  },
  unsupported: {
    variant: 'warning' as const,
    title: 'Standort nicht verfügbar',
    description:
      'Dein Browser oder Gerät unterstützt die Geolocation-API in diesem Kontext nicht. Stelle sicher, dass du die App über HTTPS aufrufst.',
  },
  error: {
    variant: 'destructive' as const,
    title: 'Standort konnte nicht ermittelt werden',
    description:
      'Beim Zugriff auf die Position ist ein Fehler aufgetreten. Prüfe, ob GPS aktiviert ist und du im Freien stehst.',
  },
} as const

const GeolocationPermissionNotice = ({
  status,
  errorMessage,
  onRetry,
}: GeolocationPermissionNoticeProps) => {
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
          Erneut versuchen
        </Button>
      )}
    </div>
  )
}

export default GeolocationPermissionNotice
