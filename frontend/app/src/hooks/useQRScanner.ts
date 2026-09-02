import { useCallback, useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import type { BarcodeDetector as BarcodeDetectorType } from 'barcode-detector/pure'

export type ScannerStatus =
  'idle' | 'requesting' | 'scanning' | 'scanned' | 'denied' | 'unsupported' | 'error'

export interface DetectionMeta {
  format: string
  boundingBox: DOMRectReadOnly
  timestamp: number
}

interface UseQRScannerOptions {
  /** Invoked once a QR code has been successfully decoded */
  onScan?: (value: string) => void
  /**
   * Continuous inspection mode (debug tooling): report every detected code with
   * metadata and keep the camera running instead of stopping on the first hit.
   * Takes precedence over `onScan`; `scannedData`/`status: 'scanned'` are never set.
   */
  onDetect?: (value: string, meta: DetectionMeta) => void
}

interface UseQRScannerReturn {
  videoRef: React.RefObject<HTMLVideoElement | null>
  status: ScannerStatus
  scannedData: string | null
  /** Translated message for the last failure; the raw browser error goes to console.warn. */
  errorMessage: string | null
  startScanning: () => Promise<void>
  stopScanning: () => void
  resetScan: () => void
}

// Resolve a BarcodeDetector ctor, preferring the native browser API.
const resolveBarcodeDetector = async (): Promise<typeof BarcodeDetectorType> => {
  if (typeof window !== 'undefined' && window.BarcodeDetector) {
    return window.BarcodeDetector
  }
  const mod = await import('barcode-detector/pure')
  return mod.BarcodeDetector
}

// Raw browser message, e.g. 'NotAllowedError: Permission denied' — vendor text,
// untranslated and inconsistent across browsers; kept for diagnosis only.
const formatError = (err: unknown): string => {
  if (!(err instanceof Error)) return String(err)
  const { name, message } = err
  return `${name}: ${message}`
}

const CAMERA_ERROR_KEYS: Record<string, string> = {
  NotAllowedError: 'browser.cameraDenied',
  PermissionDeniedError: 'browser.cameraDenied',
  NotFoundError: 'browser.cameraUnavailable',
}

const cameraErrorKey = (err: unknown): string => {
  const name = err instanceof DOMException ? err.name : err instanceof Error ? err.name : ''
  return CAMERA_ERROR_KEYS[name] ?? 'browser.scannerFailed'
}

// `t`'s generated overloads only accept the catalog's literal key union; a key
// looked up from CAMERA_ERROR_KEYS at runtime can't satisfy that statically
// (same issue as `lib/apiError.ts`'s `LooseTranslate`).
type LooseTranslate = (key: string) => string

const useQRScanner = ({ onScan, onDetect }: UseQRScannerOptions = {}): UseQRScannerReturn => {
  const { t } = useTranslation('errors')
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const detectorRef = useRef<InstanceType<typeof BarcodeDetectorType> | null>(null)
  const cancelledRef = useRef(false)
  const startingRef = useRef(false)

  const onScanRef = useRef(onScan)
  const onDetectRef = useRef(onDetect)
  useEffect(() => {
    onScanRef.current = onScan
    onDetectRef.current = onDetect
  }, [onScan, onDetect])

  const [status, setStatus] = useState<ScannerStatus>('idle')
  const [scannedData, setScannedData] = useState<string | null>(null)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)

  // Release camera + cancel any pending frame callbacks.
  const releaseStream = useCallback(() => {
    cancelledRef.current = true
    streamRef.current?.getTracks().forEach((track) => {
      track.stop()
    })
    streamRef.current = null
    if (videoRef.current) {
      videoRef.current.srcObject = null
    }
  }, [])

  const startScanning = useCallback(async (): Promise<void> => {
    if (!videoRef.current) return
    if (startingRef.current || streamRef.current) return
    startingRef.current = true

    setStatus('requesting')
    setErrorMessage(null)
    cancelledRef.current = false

    // Unsupported browser (no secure context, or very old)
    if (!navigator.mediaDevices?.getUserMedia) {
      setStatus('unsupported')
      startingRef.current = false
      return
    }

    // Optional permission pre-flight. Some browsers (Firefox) throw for 'camera'.
    try {
      const perm = await navigator.permissions.query({
        name: 'camera',
      })
      if (perm.state === 'denied') {
        setStatus('denied')
        startingRef.current = false
        return
      }
    } catch {
      // ignore — not supported everywhere
    }

    // Lazy detector setup
    if (!detectorRef.current) {
      try {
        const Detector = await resolveBarcodeDetector()
        detectorRef.current = new Detector({ formats: ['qr_code'] })
      } catch (err) {
        console.warn('Failed to initialise BarcodeDetector', formatError(err))
        setErrorMessage(t('browser.scannerFailed'))
        setStatus('error')
        startingRef.current = false
        return
      }
    }

    // Request camera stream
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: { ideal: 'environment' } },
        audio: false,
      })
      streamRef.current = stream
      if (!videoRef.current) {
        stream.getTracks().forEach((t) => t.stop())
        startingRef.current = false
        return
      }
      videoRef.current.srcObject = stream
    } catch (err) {
      const key = cameraErrorKey(err)
      console.warn('Camera start failed', formatError(err))
      setErrorMessage((t as LooseTranslate)(key))
      setStatus(key === 'browser.cameraDenied' ? 'denied' : 'error')
      startingRef.current = false
      return
    }

    setStatus('scanning')
    startingRef.current = false
    cancelledRef.current = false

    const useVFC = 'requestVideoFrameCallback' in HTMLVideoElement.prototype

    const handleSuccess = (value: string) => {
      setScannedData(value)
      setStatus('scanned')
      // Release camera — prevents duplicate scans & frees hardware
      releaseStream()
      onScanRef.current?.(value)
    }

    const tick = async (): Promise<void> => {
      if (cancelledRef.current) return
      const video = videoRef.current
      const detector = detectorRef.current
      if (!video || !detector) return

      if (video.readyState >= 2 && !video.paused) {
        try {
          const codes = await detector.detect(video)
          if (cancelledRef.current) return
          if (codes.length > 0) {
            const code = codes[0]
            if (onDetectRef.current) {
              onDetectRef.current(code.rawValue ?? '', {
                format: String(code.format),
                boundingBox: code.boundingBox,
                timestamp: Date.now(),
              })
            } else if (code.rawValue) {
              handleSuccess(code.rawValue)
              return
            }
          }
        } catch (err) {
          // Detection can fail transiently (e.g. mid-frame). Keep scanning.
          console.debug('detect() threw, continuing', err)
        }
      }
      scheduleNext()
    }

    const scheduleNext = () => {
      if (cancelledRef.current) return
      if (useVFC && videoRef.current) {
        videoRef.current.requestVideoFrameCallback(() => {
          void tick()
        })
      } else {
        requestAnimationFrame(() => {
          void tick()
        })
      }
    }

    scheduleNext()
  }, [releaseStream, t])

  const stopScanning = useCallback(() => {
    releaseStream()
    setStatus('idle')
  }, [releaseStream])

  const resetScan = useCallback(() => {
    setScannedData(null)
    setStatus('idle')
  }, [])

  // Always release the camera on unmount
  useEffect(() => {
    return () => {
      releaseStream()
    }
  }, [releaseStream])

  return {
    videoRef,
    status,
    scannedData,
    errorMessage,
    startScanning,
    stopScanning,
    resetScan,
  }
}

export default useQRScanner
