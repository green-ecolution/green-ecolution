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

const DISMISS_KEY = 'ge.pwa-install-hint.scanner.dismissed-at'
const RESHOW_AFTER_MS = 30 * 24 * 60 * 60 * 1000

const readDismissed = (): boolean => {
  if (typeof localStorage === 'undefined') return false
  try {
    const raw = localStorage.getItem(DISMISS_KEY)
    if (!raw) return false
    const dismissedAt = Number(raw)
    if (!Number.isFinite(dismissedAt)) return false
    return Date.now() - dismissedAt < RESHOW_AFTER_MS
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
      localStorage.setItem(DISMISS_KEY, String(Date.now()))
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
      ? 'Tippe unten in Safari auf das Teilen-Symbol und wähle "Zum Home-Bildschirm". So startet der Scanner direkt im Vollbild ohne Adressleiste.'
      : 'Installiere die App auf deinem Gerät. Der Scanner startet schneller und läuft im Vollbild ohne Browser-Leiste.'

  return (
    <Alert variant="info" className="relative w-full mb-6">
      <div className="flex items-start gap-3 pr-6">
        <AlertIcon variant="info" />
        <AlertContent>
          <AlertTitle>App installieren für die beste Scan-Erfahrung</AlertTitle>
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
