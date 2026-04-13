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
  /** Auto-start watching on mount. Options below are read once at start. */
  autoStart?: boolean
  /** Forwarded to the Geolocation API. Default: high accuracy. */
  enableHighAccuracy?: boolean
  /** Forwarded to the Geolocation API. Default: Infinity (no per-fix timeout). */
  timeout?: number
  /** Forwarded to the Geolocation API. Default: 0. */
  maximumAge?: number
  /** Invoked once the first fix is delivered. */
  onLocated?: (fix: GeolocationFix) => void
}

interface UseGeolocationReturn {
  status: GeolocationStatus
  position: GeolocationFix | null
  history: GeolocationFix[]
  errorMessage: string | null
  /** Register a watch and resolve with the first fix (or reject on fatal error). */
  start: () => Promise<GeolocationFix | null>
  /** Stop the active watch. Keeps the last known position. */
  stop: () => void
  /** Stop + clear state. */
  reset: () => void
  /** Atomic stop + start — prefer this over calling reset()+start() manually. */
  relocate: () => Promise<GeolocationFix | null>
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
  timeout,
  maximumAge = 0,
  onLocated,
}: UseGeolocationOptions = {}): UseGeolocationReturn => {
  const watchIdRef = useRef<number | null>(null)
  const startingRef = useRef(false)
  const locatedRef = useRef(false)
  // Pending start() promise — resolved on first fix, rejected on fatal error.
  const pendingResolveRef = useRef<((fix: GeolocationFix | null) => void) | null>(null)
  const pendingRejectRef = useRef<((reason: unknown) => void) | null>(null)

  // Always read the latest options via refs so the Effect only runs once.
  const optionsRef = useRef({ enableHighAccuracy, timeout, maximumAge })
  useEffect(() => {
    optionsRef.current = { enableHighAccuracy, timeout, maximumAge }
  }, [enableHighAccuracy, timeout, maximumAge])

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

  const settlePending = useCallback((fix: GeolocationFix | null, error?: unknown) => {
    if (error && pendingRejectRef.current) {
      pendingRejectRef.current(error)
    } else if (pendingResolveRef.current) {
      pendingResolveRef.current(fix)
    }
    pendingResolveRef.current = null
    pendingRejectRef.current = null
  }, [])

  const handleSuccess = useCallback(
    (raw: GeolocationPosition) => {
      const fix = toFix(raw)
      setPosition(fix)
      setStatus('watching')
      // A successful fix implicitly clears a stale transient error.
      setErrorMessage(null)
      if (trackHistory) {
        setHistory((prev) => [fix, ...prev].slice(0, 200))
      }
      if (!locatedRef.current) {
        locatedRef.current = true
        onLocatedRef.current?.(fix)
        settlePending(fix)
      }
    },
    [trackHistory, settlePending],
  )

  const handleError = useCallback(
    (err: GeolocationPositionError) => {
      setErrorMessage(err.message || null)
      if (err.code === err.PERMISSION_DENIED) {
        setStatus('denied')
        clearWatch()
        settlePending(null, err)
        return
      }
      // POSITION_UNAVAILABLE / TIMEOUT — keep watching, surface as error state
      // only when we have no fix yet; otherwise stay in 'watching' with last value.
      if (!locatedRef.current) {
        setStatus('error')
        settlePending(null, err)
      }
    },
    [clearWatch, settlePending],
  )

  const start = useCallback((): Promise<GeolocationFix | null> => {
    // Already watching — return a no-op promise resolving to the last fix.
    if (watchIdRef.current !== null) {
      return Promise.resolve(position)
    }
    // A start() is mid-flight (permissions pre-flight).
    if (startingRef.current) {
      return new Promise((resolve, reject) => {
        pendingResolveRef.current = resolve
        pendingRejectRef.current = reject
      })
    }

    startingRef.current = true
    setStatus('requesting')
    setErrorMessage(null)
    locatedRef.current = false

    return new Promise<GeolocationFix | null>((resolve, reject) => {
      pendingResolveRef.current = resolve
      pendingRejectRef.current = reject

      const run = async () => {
        if (typeof navigator === 'undefined' || !navigator.geolocation) {
          setStatus('unsupported')
          startingRef.current = false
          settlePending(null)
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
            settlePending(null)
            return
          }
        } catch {
          // ignore
        }

        try {
          const opts = optionsRef.current
          watchIdRef.current = navigator.geolocation.watchPosition(handleSuccess, handleError, {
            enableHighAccuracy: opts.enableHighAccuracy,
            timeout: opts.timeout,
            maximumAge: opts.maximumAge,
          })
        } catch (err) {
          console.error('Failed to start geolocation watch', err)
          setStatus('error')
          setErrorMessage(err instanceof Error ? err.message : String(err))
          settlePending(null, err)
        } finally {
          startingRef.current = false
        }
      }

      void run()
    })
  }, [handleSuccess, handleError, settlePending, position])

  const stop = useCallback(() => {
    clearWatch()
    setStatus('idle')
  }, [clearWatch])

  const reset = useCallback(() => {
    clearWatch()
    locatedRef.current = false
    settlePending(null)
    setPosition(null)
    setHistory([])
    setErrorMessage(null)
    setStatus('idle')
  }, [clearWatch, settlePending])

  const relocate = useCallback((): Promise<GeolocationFix | null> => {
    clearWatch()
    locatedRef.current = false
    settlePending(null)
    setErrorMessage(null)
    // Keep the stale `position` visible until the next fix arrives — feels
    // smoother than flashing an empty state.
    return start()
  }, [clearWatch, settlePending, start])

  // autoStart on mount; options are captured via optionsRef so this runs once.
  useEffect(() => {
    if (autoStart) {
      void start()
    }
    return () => {
      clearWatch()
      settlePending(null)
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
    relocate,
  }
}

export default useGeolocation
