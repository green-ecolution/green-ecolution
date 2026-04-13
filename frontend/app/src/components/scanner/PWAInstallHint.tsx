import usePWAInstall from '@/hooks/usePWAInstall'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
} from '@green-ecolution/ui'
import { Download, Share, X } from 'lucide-react'
import { useEffect, useState } from 'react'

const DISMISS_KEY = 'ge.pwa-install-hint.scanner.dismissed'

const readDismissed = (): boolean => {
  if (typeof localStorage === 'undefined') return false
  try {
    return localStorage.getItem(DISMISS_KEY) === 'true'
  } catch {
    return false
  }
}

const PWAInstallHint = () => {
  const { isStandalone, platform, canPromptInstall, promptInstall } = usePWAInstall()
  const [dismissed, setDismissed] = useState(readDismissed)

  // Cross-tab dismissal sync
  useEffect(() => {
    const onStorage = (e: StorageEvent) => {
      if (e.key === DISMISS_KEY) setDismissed(readDismissed())
    }
    window.addEventListener('storage', onStorage)
    return () => {
      window.removeEventListener('storage', onStorage)
    }
  }, [])

  if (isStandalone || dismissed) return null

  const handleDismiss = () => {
    try {
      localStorage.setItem(DISMISS_KEY, 'true')
    } catch {
      /* ignore */
    }
    setDismissed(true)
  }

  const handleInstall = async () => {
    await promptInstall()
  }

  const description =
    platform === 'ios'
      ? 'Tippe unten in Safari auf das Teilen-Symbol und wähle "Zum Home-Bildschirm", um die App zu installieren. So bleibt der Kamerazugriff dauerhaft gespeichert.'
      : 'Installiere die App auf deinem Gerät, damit der Kamerazugriff gespeichert bleibt und der Scanner schneller startet.'

  return (
    <Alert variant="info" className="relative w-full mb-6">
      <div className="flex items-start gap-3 pr-6">
        <AlertIcon variant="info" />
        <AlertContent>
          <AlertTitle>App installieren für dauerhaften Kamerazugriff</AlertTitle>
          <AlertDescription>{description}</AlertDescription>
          {(platform === 'ios' || canPromptInstall) && (
            <div className="mt-3 flex flex-wrap items-center gap-2">
              {platform === 'ios' ? (
                <span className="inline-flex items-center gap-1.5 text-xs text-muted-foreground">
                  <Share className="size-4" aria-hidden="true" />
                  Teilen → Zum Home-Bildschirm
                </span>
              ) : (
                <Button size="sm" onClick={() => void handleInstall()}>
                  <Download />
                  Installieren
                </Button>
              )}
            </div>
          )}
        </AlertContent>
      </div>
      <button
        type="button"
        onClick={handleDismiss}
        aria-label="Hinweis schließen"
        className="absolute top-2 right-2 rounded-md p-1 text-muted-foreground hover:text-foreground hover:bg-dark-100 transition-colors"
      >
        <X className="size-4" aria-hidden="true" />
      </button>
    </Alert>
  )
}

export default PWAInstallHint
