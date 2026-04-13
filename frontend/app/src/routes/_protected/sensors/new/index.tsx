import createToast from '@/hooks/createToast'
import BackLink from '@/components/general/links/BackLink'
import QRScannerView from '@/components/scanner/QRScannerView'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/new/')({
  component: NewSensor,
})

function NewSensor() {
  const showToast = createToast()

  const handleContinue = (sensorId: string) => {
    showToast(`Sensor-Verknüpfung ist noch nicht implementiert (erkannt: ${sensorId})`, 'error')
  }

  return (
    <div className="container mt-6">
      <BackLink label="Zurück zur Sensorübersicht" link={{ to: '/sensors' }} />
      <article className="2xl:w-4/5 mb-8 md:mb-10">
        <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
          Sensor hinzufügen
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Scanne den QR-Code auf der Sensoreinheit, um den Sensor zu identifizieren.
        </p>
      </article>
      <QRScannerView continueLabel="Sensor übernehmen" onContinue={handleContinue} />
    </div>
  )
}
