import { useRegisterSW } from 'virtual:pwa-register/react'

const UPDATE_CHECK_INTERVAL = 60 * 60 * 1000 // 60 minutes

export default function useAppUpdate() {
  const {
    needRefresh: [needRefresh, setNeedRefresh],
    updateServiceWorker,
  } = useRegisterSW({
    onRegisteredSW(_swUrl, registration) {
      if (!registration) return
      setInterval(() => {
        void registration.update()
      }, UPDATE_CHECK_INTERVAL)
    },
  })

  const performUpdate = async () => {
    const keys = await caches.keys()
    await Promise.all(keys.map((key) => caches.delete(key)))
    localStorage.clear()
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
