import InlineGPSReadout from '@/components/geolocation/InlineGPSReadout'
import QRScannerView from '@/components/scanner/QRScannerView'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'

interface SensorScanStepProps {
  gpsPosition: GeolocationFix | null
  gpsStatus: GeolocationStatus
  onScanned: (sensorId: string) => void
}

const SensorScanStep = ({ gpsPosition, gpsStatus, onScanned }: SensorScanStepProps) => {
  return (
    <div className="space-y-4">
      <header className="space-y-1">
        <h2 className="text-xl font-semibold">Sensor-QR scannen</h2>
        <p className="text-sm text-muted-foreground">
          Halte den QR-Code auf der Sensoreinheit in den Scan-Rahmen. Dein Standort wird im
          Hintergrund bereits erfasst.
        </p>
      </header>
      <QRScannerView
        continueLabel="Sensor übernehmen"
        onContinue={onScanned}
        extra={<InlineGPSReadout position={gpsPosition} status={gpsStatus} />}
      />
    </div>
  )
}

export default SensorScanStep
