import createToast from '@/hooks/createToast'
import useQRScanner, { type ScannerStatus } from '@/hooks/useQRScanner'
import { CameraViewport, Loading, type CameraViewportState } from '@green-ecolution/ui'
import { CameraOff, CircleAlert, ShieldAlert } from 'lucide-react'
import { useEffect } from 'react'
import CameraPermissionNotice from './CameraPermissionNotice'
import PWAInstallHint from './PWAInstallHint'
import QRScanResult from './QRScanResult'

const STATUS_LABELS: Record<ScannerStatus, string> = {
  idle: '',
  requesting: 'Kamera wird gestartet',
  scanning: 'Suche nach QR-Code',
  scanned: 'Erfolgreich erkannt',
  denied: 'Zugriff verweigert',
  unsupported: 'Nicht verfügbar',
  error: 'Fehler',
}

const STATUS_TO_VIEWPORT: Record<ScannerStatus, CameraViewportState> = {
  idle: 'inactive',
  requesting: 'loading',
  scanning: 'scanning',
  scanned: 'success',
  denied: 'error',
  unsupported: 'error',
  error: 'error',
}

interface QRScannerViewProps {
  /** Label rendered on the continue button in the result card. Defaults to "Weiter". */
  continueLabel?: string
  /** Invoked with the decoded value when the user confirms via the continue button */
  onContinue?: (value: string) => void
  /** Optional extra block rendered inside the success card (e.g. GPS readout). */
  extra?: React.ReactNode
}

const SCAN_VIBRATE_MS = 40

const QRScannerView = ({ continueLabel, onContinue, extra }: QRScannerViewProps = {}) => {
  const showToast = createToast()

  const { videoRef, status, scannedData, startScanning, resetScan } = useQRScanner({
    onScan: () => {
      if (typeof navigator !== 'undefined' && typeof navigator.vibrate === 'function') {
        navigator.vibrate(SCAN_VIBRATE_MS)
      }
      showToast('QR-Code erfolgreich gescannt', 'success')
    },
  })

  useEffect(() => {
    void startScanning()
  }, [startScanning])

  const handleScanAgain = () => {
    resetScan()
    void startScanning()
  }

  const handleRetry = () => {
    void startScanning()
  }

  const viewportState = STATUS_TO_VIEWPORT[status]

  let viewportOverlay: React.ReactNode = null
  if (status === 'requesting') {
    viewportOverlay = (
      <span className="flex flex-col items-center gap-3 text-white/80">
        <Loading size="lg" label="" />
        <span className="text-sm">Kamera wird gestartet …</span>
      </span>
    )
  } else if (status === 'denied') {
    viewportOverlay = <CameraOff aria-hidden="true" className="size-12 text-white/40" />
  } else if (status === 'unsupported') {
    viewportOverlay = <ShieldAlert aria-hidden="true" className="size-12 text-white/40" />
  } else if (status === 'error') {
    viewportOverlay = <CircleAlert aria-hidden="true" className="size-12 text-white/40" />
  }

  const showResult = status === 'scanned' && scannedData

  return (
    <div className="mx-auto max-w-md pb-[env(safe-area-inset-bottom)]">
      <PWAInstallHint />
      <div className={showResult ? 'hidden' : 'block'}>
        <CameraViewport
          videoRef={videoRef}
          state={viewportState}
          overlay={viewportOverlay}
          ariaLabel="Kamera-Vorschau für QR-Code-Scanner"
        />
      </div>
      {showResult && (
        <QRScanResult
          sensorId={scannedData}
          onScanAgain={handleScanAgain}
          continueLabel={continueLabel}
          onContinue={onContinue}
          extra={extra}
        />
      )}

      {STATUS_LABELS[status] && (
        <p
          role="status"
          aria-live="polite"
          className="mt-4 text-center text-xs uppercase tracking-[0.2em] text-muted-foreground"
        >
          · {STATUS_LABELS[status]} ·
        </p>
      )}

      <div className="mt-6 min-h-32">
        {status === 'scanning' && (
          <p className="text-sm text-muted-foreground text-center max-w-prose mx-auto">
            Sorge für ausreichend Licht und halte den Code ruhig vor die Kamera.
          </p>
        )}
        {(status === 'denied' || status === 'unsupported' || status === 'error') && (
          <CameraPermissionNotice
            status={status}
            onRetry={status === 'unsupported' ? undefined : handleRetry}
          />
        )}
      </div>
    </div>
  )
}

export default QRScannerView
