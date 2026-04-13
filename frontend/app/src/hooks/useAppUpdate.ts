import { useEffect, useState } from 'react'
import { useRegisterSW } from 'virtual:pwa-register/react'

const UPDATE_CHECK_INTERVAL = 60 * 60 * 1000 // 60 minutes

export default function useAppUpdate() {
  const [registration, setRegistration] = useState<ServiceWorkerRegistration | null>(null)

  const {
    needRefresh: [needRefresh, setNeedRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegisteredSW(_swUrl, reg) {
      if (reg) setRegistration(reg)
    },
  })

  useEffect(() => {
    if (!registration) return
    void registration.update()
    const id = setInterval(() => {
      void registration.update()
    }, UPDATE_CHECK_INTERVAL)
    return () => clearInterval(id)
  }, [registration])

  const performUpdate = async () => {
    await updateServiceWorker(true)
  }

  const dismissUpdate = () => {
    setNeedRefresh(false)
  }

  return {
    updateAvailable: needRefresh,
    performUpdate,
    dismissUpdate,
  }
}
