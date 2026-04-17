import BackLink from '@/components/general/links/BackLink'
import InlineGPSReadout from '@/components/geolocation/InlineGPSReadout'
import QRScannerView from '@/components/scanner/QRScannerView'
import SensorGeolocationSummary from '@/components/sensor/SensorGeolocationSummary'
import useGeolocation, { type GeolocationFix } from '@/hooks/useGeolocation'
import { createFileRoute } from '@tanstack/react-router'
import { useCallback, useEffect, useRef, useState } from 'react'

export const Route = createFileRoute('/_protected/sensors/new/')({
  component: NewSensor,
})

function NewSensor() {
  // Start GPS in parallel with the QR-scan so a fix is usually ready by the
  // time the technician confirms the scanned sensor.
  const { status, position, errorMessage, stop, relocate } = useGeolocation({ autoStart: true })

  const [scannedSensorId, setScannedSensorId] = useState<string | null>(null)
  // Frozen snapshot shown in the summary. Decouples the displayed fix from
  // further watchPosition updates so the map / accuracy don't keep drifting
  // after the user has accepted the sensor.
  const [frozenFix, setFrozenFix] = useState<GeolocationFix | null>(null)
  const pendingStopRef = useRef(false)

  useEffect(() => {
    if (!pendingStopRef.current || !position) return
    pendingStopRef.current = false
    stop()
  }, [position, stop])

  const handleContinue = (sensorId: string) => {
    setScannedSensorId(sensorId)
    if (position) {
      setFrozenFix(position)
      stop()
    } else {
      pendingStopRef.current = true
    }
  }

  const handleScanAgain = () => {
    setScannedSensorId(null)
    setFrozenFix(null)
    pendingStopRef.current = false
    void relocate()
  }

  const handleRelocate = async () => {
    setFrozenFix(null)
    pendingStopRef.current = false
    const next = await relocate().catch(() => null)
    if (next) {
      setFrozenFix(next)
      stop()
    }
  }

  const handleConfirmTree = useCallback((_treeId: number) => {
    // Tree assignment stored for future backend linking
  }, [])

  const summaryFix = frozenFix ?? (scannedSensorId ? position : null)

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
          position={summaryFix}
          status={status}
          errorMessage={errorMessage}
          onScanAgain={handleScanAgain}
          onRelocate={handleRelocate}
          onConfirmTree={handleConfirmTree}
        />
      ) : (
        <QRScannerView
          continueLabel="Sensor übernehmen"
          onContinue={handleContinue}
          extra={<InlineGPSReadout position={position} status={status} />}
        />
      )}
    </div>
  )
}
