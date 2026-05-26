import InlineGPSReadout from '@/components/geolocation/InlineGPSReadout'
import QRScannerView from '@/components/scanner/QRScannerView'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import { Button, CopyableText } from '@green-ecolution/ui'
import { CheckCircle2, RotateCw } from 'lucide-react'

interface SensorScanStepProps {
  gpsPosition: GeolocationFix | null
  gpsStatus: GeolocationStatus
  scannedSensorId: string | null
  onScanned: (sensorId: string) => void
  onScanAgain: () => void
}

const SensorScanStep = ({
  gpsPosition,
  gpsStatus,
  scannedSensorId,
  onScanned,
  onScanAgain,
}: SensorScanStepProps) => {
  if (scannedSensorId) {
    return (
      <div className="space-y-4">
        <header className="space-y-1">
          <h2 className="text-xl font-semibold">Sensor erkannt</h2>
          <p className="text-sm text-muted-foreground">
            Tippe auf „Weiter", um mit dem GPS-Standort fortzufahren, oder scanne einen anderen
            Sensor.
          </p>
        </header>

        <div className="rounded-2xl border border-green-dark/30 bg-green-dark-50/30 p-4 md:p-5 space-y-3">
          <div className="flex items-center gap-2 text-sm font-medium text-green-dark">
            <CheckCircle2 className="size-4" aria-hidden />
            <span>QR-Code übernommen</span>
          </div>
          <CopyableText value={scannedSensorId} label="Sensor-ID" />
        </div>

        <Button variant="outline" onClick={onScanAgain} className="w-full sm:w-auto">
          <RotateCw className="size-4" />
          Anderen Sensor scannen
        </Button>
      </div>
    )
  }

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
