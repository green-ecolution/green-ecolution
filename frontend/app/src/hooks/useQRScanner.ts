import { useCallback, useEffect, useRef, useState } from 'react'
import type { BarcodeDetector as BarcodeDetectorType } from 'barcode-detector/pure'

export type ScannerStatus =
  | 'idle'
  | 'requesting'
  | 'scanning'
  | 'scanned'
  | 'denied'
  | 'unsupported'
  | 'error'

interface UseQRScannerOptions {
  /** Invoked once a QR code has been successfully decoded */
  onScan?: (value: string) => void
}

interface UseQRScannerReturn {
  videoRef: React.RefObject<HTMLVideoElement | null>
  status: ScannerStatus
  scannedData: string | null
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

const useQRScanner = ({ onScan }: UseQRScannerOptions = {}): UseQRScannerReturn => {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const detectorRef = useRef<InstanceType<typeof BarcodeDetectorType> | null>(null)
  const cancelledRef = useRef(false)
  const startingRef = useRef(false)

  const onScanRef = useRef(onScan)
  useEffect(() => {
    onScanRef.current = onScan
  }, [onScan])

  const [status, setStatus] = useState<ScannerStatus>('idle')
  const [scannedData, setScannedData] = useState<string | null>(null)

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
        name: 'camera' as PermissionName,
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
        console.error('Failed to initialise BarcodeDetector', err)
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
      const name = err instanceof DOMException ? err.name : ''
      if (name === 'NotAllowedError' || name === 'PermissionDeniedError') {
        setStatus('denied')
      } else {
        console.error('Camera start failed', err)
        setStatus('error')
      }
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
          if (codes.length > 0 && codes[0].rawValue) {
            handleSuccess(codes[0].rawValue)
            return
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
  }, [releaseStream])

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
    startScanning,
    stopScanning,
    resetScan,
  }
}

export default useQRScanner
