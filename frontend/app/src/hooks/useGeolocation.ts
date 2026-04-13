import { useCallback, useEffect, useRef, useState } from 'react'

export type GeolocationStatus =
  | 'idle'
  | 'requesting'
  | 'watching'
  | 'denied'
  | 'unsupported'
  | 'error'

export interface GeolocationFix {
  latitude: number
  longitude: number
  accuracy: number
  altitude: number | null
  altitudeAccuracy: number | null
  heading: number | null
  speed: number | null
  timestamp: number
}

interface UseGeolocationOptions {
  /** Capture every position update in `history` (e.g. for the debug view). */
  trackHistory?: boolean
  /** Auto-start watching on mount. */
  autoStart?: boolean
  /** Forwarded to the Geolocation API. Defaults: high accuracy, 15s timeout. */
  enableHighAccuracy?: boolean
  timeout?: number
  maximumAge?: number
  /** Invoked once the first fix is delivered. */
  onLocated?: (fix: GeolocationFix) => void
}

interface UseGeolocationReturn {
  status: GeolocationStatus
  position: GeolocationFix | null
  history: GeolocationFix[]
  errorMessage: string | null
  start: () => Promise<void>
  stop: () => void
  reset: () => void
}

const toFix = (position: GeolocationPosition): GeolocationFix => ({
  latitude: position.coords.latitude,
  longitude: position.coords.longitude,
  accuracy: position.coords.accuracy,
  altitude: position.coords.altitude,
  altitudeAccuracy: position.coords.altitudeAccuracy,
  heading: position.coords.heading,
  speed: position.coords.speed,
  timestamp: position.timestamp,
})

const useGeolocation = ({
  trackHistory = false,
  autoStart = false,
  enableHighAccuracy = true,
  timeout = 15000,
  maximumAge = 0,
  onLocated,
}: UseGeolocationOptions = {}): UseGeolocationReturn => {
  const watchIdRef = useRef<number | null>(null)
  const startingRef = useRef(false)
  const locatedRef = useRef(false)

  const onLocatedRef = useRef(onLocated)
  useEffect(() => {
    onLocatedRef.current = onLocated
  }, [onLocated])

  const [status, setStatus] = useState<GeolocationStatus>('idle')
  const [position, setPosition] = useState<GeolocationFix | null>(null)
  const [history, setHistory] = useState<GeolocationFix[]>([])
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  const clearWatch = useCallback(() => {
    if (watchIdRef.current !== null && navigator.geolocation) {
      navigator.geolocation.clearWatch(watchIdRef.current)
    }
    watchIdRef.current = null
  }, [])

  const handleSuccess = useCallback(
    (raw: GeolocationPosition) => {
      const fix = toFix(raw)
      setPosition(fix)
      setStatus('watching')
      if (trackHistory) {
        setHistory((prev) => [fix, ...prev].slice(0, 200))
      }
      if (!locatedRef.current) {
        locatedRef.current = true
        onLocatedRef.current?.(fix)
      }
    },
    [trackHistory],
  )

  const handleError = useCallback(
    (err: GeolocationPositionError) => {
      setErrorMessage(err.message || null)
      if (err.code === err.PERMISSION_DENIED) {
        setStatus('denied')
        clearWatch()
      } else {
        // POSITION_UNAVAILABLE / TIMEOUT — keep watching, surface as error state
        // only when we have no fix yet; otherwise stay in 'watching' with last value.
        if (!locatedRef.current) {
          setStatus('error')
        }
      }
    },
    [clearWatch],
  )

  const start = useCallback(async (): Promise<void> => {
    if (startingRef.current || watchIdRef.current !== null) return
    startingRef.current = true

    setStatus('requesting')
    setErrorMessage(null)

    if (typeof navigator === 'undefined' || !navigator.geolocation) {
      setStatus('unsupported')
      startingRef.current = false
      return
    }

    // Optional permission pre-flight (not supported everywhere — Safari throws).
    try {
      const perm = await navigator.permissions.query({
        name: 'geolocation' as PermissionName,
      })
      if (perm.state === 'denied') {
        setStatus('denied')
        startingRef.current = false
        return
      }
    } catch {
      // ignore
    }

    locatedRef.current = false

    try {
      watchIdRef.current = navigator.geolocation.watchPosition(handleSuccess, handleError, {
        enableHighAccuracy,
        timeout,
        maximumAge,
      })
    } catch (err) {
      console.error('Failed to start geolocation watch', err)
      setStatus('error')
      setErrorMessage(err instanceof Error ? err.message : String(err))
    } finally {
      startingRef.current = false
    }
  }, [enableHighAccuracy, timeout, maximumAge, handleSuccess, handleError])

  const stop = useCallback(() => {
    clearWatch()
    setStatus('idle')
  }, [clearWatch])

  const reset = useCallback(() => {
    clearWatch()
    locatedRef.current = false
    setPosition(null)
    setHistory([])
    setErrorMessage(null)
    setStatus('idle')
  }, [clearWatch])

  // autoStart on mount
  useEffect(() => {
    if (autoStart) {
      void start()
    }
    return () => {
      clearWatch()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  return {
    status,
    position,
    history,
    errorMessage,
    start,
    stop,
    reset,
  }
}

export default useGeolocation
