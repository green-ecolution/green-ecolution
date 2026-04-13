import BackLink from '@/components/general/links/BackLink'
import InlineGPSReadout from '@/components/geolocation/InlineGPSReadout'
import QRScannerView from '@/components/scanner/QRScannerView'
import SensorGeolocationSummary from '@/components/sensor/SensorGeolocationSummary'
import useGeolocation from '@/hooks/useGeolocation'
import { createFileRoute } from '@tanstack/react-router'
import { useState } from 'react'

export const Route = createFileRoute('/_protected/sensors/new/')({
  component: NewSensor,
})

function NewSensor() {
  // Start GPS in parallel with the QR-scan so a fix is usually ready by the
  // time the technician confirms the scanned sensor.
  const { status, position, errorMessage, start, reset } = useGeolocation({ autoStart: true })

  const [scannedSensorId, setScannedSensorId] = useState<string | null>(null)

  const handleContinue = (sensorId: string) => {
    setScannedSensorId(sensorId)
  }

  const handleScanAgain = () => {
    setScannedSensorId(null)
  }

  const handleRelocate = () => {
    reset()
    void start()
  }

  return (
    <div className="container mt-6">
      <BackLink label="Zurück zur Sensorübersicht" link={{ to: '/sensors' }} />
      <article className="2xl:w-4/5 mb-8 md:mb-10">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
          Sensor hinzufügen
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          {scannedSensorId
            ? 'Überprüfe Sensor-ID und Standort. Anhand des erfassten GPS-Standorts wird automatisch der nächstgelegene Baum gesucht und mit dem Sensor verknüpft.'
            : 'Scanne den QR-Code auf der Sensoreinheit, um den Sensor zu identifizieren. Parallel wird dein GPS-Standort ermittelt, um den nächstgelegenen Baum automatisch zu finden und mit dem Sensor zu verknüpfen.'}
        </p>
      </article>

      {scannedSensorId ? (
        <SensorGeolocationSummary
          sensorId={scannedSensorId}
          position={position}
          status={status}
          errorMessage={errorMessage}
          onScanAgain={handleScanAgain}
          onRelocate={handleRelocate}
        />
      ) : (
        <QRScannerView
          continueLabel="Sensor übernehmen"
          onContinue={handleContinue}
          resultExtra={<InlineGPSReadout position={position} status={status} />}
        />
      )}
    </div>
  )
}
