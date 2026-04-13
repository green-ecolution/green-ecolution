import { useCallback, useEffect, useMemo, useState } from 'react'

interface BeforeInstallPromptEvent extends Event {
  readonly platforms: readonly string[]
  readonly userChoice: Promise<{ outcome: 'accepted' | 'dismissed'; platform: string }>
  prompt: () => Promise<void>
}

export type PWAPlatform = 'ios' | 'android' | 'desktop' | 'unknown'

// Module-level state so the beforeinstallprompt event — which fires at most once
// and typically before any component mounts — isn't lost.
let deferredPrompt: BeforeInstallPromptEvent | null = null
const changeListeners = new Set<() => void>()

const notify = () => {
  changeListeners.forEach((fn) => {
    fn()
  })
}

if (typeof window !== 'undefined') {
  window.addEventListener('beforeinstallprompt', (event: Event) => {
    event.preventDefault()
    deferredPrompt = event as BeforeInstallPromptEvent
    notify()
  })
  window.addEventListener('appinstalled', () => {
    deferredPrompt = null
    notify()
  })
}

const checkStandalone = (): boolean => {
  if (typeof window === 'undefined') return false
  return (
    window.matchMedia('(display-mode: standalone)').matches ||
    (window.navigator as Navigator & { standalone?: boolean }).standalone === true
  )
}

const detectPlatform = (): PWAPlatform => {
  if (typeof navigator === 'undefined') return 'unknown'
  const ua = navigator.userAgent
  if (/iPad|iPhone|iPod/.test(ua)) return 'ios'
  // Safari on iPadOS 13+ reports as Mac — fall back to touch detection
  if (ua.includes('Macintosh') && 'ontouchend' in document) return 'ios'
  if (/Android/i.test(ua)) return 'android'
  return 'desktop'
}

export interface UsePWAInstallReturn {
  /** True when the app runs as an installed PWA (standalone display mode) */
  isStandalone: boolean
  /** Best-effort platform detection for platform-specific instructions */
  platform: PWAPlatform
  /** True when beforeinstallprompt was captured and install can be triggered */
  canPromptInstall: boolean
  /** Trigger the browser's install prompt. Resolves once the user decided. */
  promptInstall: () => Promise<void>
}

const usePWAInstall = (): UsePWAInstallReturn => {
  const [canPromptInstall, setCanPromptInstall] = useState(() => deferredPrompt !== null)
  const [isStandalone, setIsStandalone] = useState(checkStandalone)

  useEffect(() => {
    const onChange = () => {
      setCanPromptInstall(deferredPrompt !== null)
    }
    changeListeners.add(onChange)
    return () => {
      changeListeners.delete(onChange)
    }
  }, [])

  useEffect(() => {
    if (typeof window === 'undefined') return
    const mq = window.matchMedia('(display-mode: standalone)')
    const handler = () => {
      setIsStandalone(checkStandalone())
    }
    mq.addEventListener('change', handler)
    return () => {
      mq.removeEventListener('change', handler)
    }
  }, [])

  const platform = useMemo(detectPlatform, [])

  const promptInstall = useCallback(async () => {
    if (!deferredPrompt) return
    const evt = deferredPrompt
    try {
      await evt.prompt()
      await evt.userChoice
    } finally {
      deferredPrompt = null
      notify()
    }
  }, [])

  return { isStandalone, platform, canPromptInstall, promptInstall }
}

export default usePWAInstall
