import QRScannerView from '@/components/scanner/QRScannerView'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { Button, CopyableText, Loading } from '@green-ecolution/ui'
import { AlertTriangle, CheckCircle2, RotateCw, ScanSearch, WifiOff } from 'lucide-react'

interface SensorScanStepProps {
  scannedSensorId: string | null
  isLookupLoading: boolean
  isLookupError: boolean
  lookupErrorStatus: number | null
  sensor: SensorResponse | null
  onScanned: (sensorId: string) => void
  onScanAgain: () => void
  onRetryLookup: () => void
}

const dateFormatter = new Intl.DateTimeFormat('de-DE', {
  dateStyle: 'medium',
  timeStyle: 'short',
})

const formatLatestData = (iso: string): string => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return iso
  return dateFormatter.format(date)
}

const ScannerHeader = () => (
  <header className="space-y-2">
    <h1 className="font-lato font-bold text-3xl lg:text-4xl">Sensor-QR scannen</h1>
    <p className="text-sm text-muted-foreground max-w-prose">
      Halte den QR-Code auf der Sensoreinheit ruhig in den Scan-Rahmen. Wir identifizieren damit den
      Sensor eindeutig, bevor du ihn im nächsten Schritt einem Baum zuordnest. Bei schlechter
      Beleuchtung hilft es, den Code etwas näher heranzuhalten.
    </p>
  </header>
)

const SensorScanStep = ({
  scannedSensorId,
  isLookupLoading,
  isLookupError,
  lookupErrorStatus,
  sensor,
  onScanned,
  onScanAgain,
  onRetryLookup,
}: SensorScanStepProps) => {
  if (!scannedSensorId) {
    return (
      <div className="space-y-6">
        <ScannerHeader />
        <QRScannerView continueLabel="Sensor übernehmen" onContinue={onScanned} />
      </div>
    )
  }

  if (isLookupLoading) {
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">Sensor wird geprüft</h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            Wir gleichen die gescannte ID mit der Datenbank ab. Das dauert nur einen Moment.
          </p>
        </header>

        <div className="rounded-2xl border border-dark-100 bg-dark-50/40 p-4 md:p-5 space-y-4">
          <CopyableText value={scannedSensorId} label="Sensor-ID" />
          <Loading size="default" label="Sensor wird im System abgeglichen …" />
        </div>
      </div>
    )
  }

  if (isLookupError) {
    const notFound = lookupErrorStatus === 404
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">
            {notFound ? 'Sensor nicht gefunden' : 'Abgleich fehlgeschlagen'}
          </h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            {notFound
              ? 'Die gescannte Sensor-ID ist im System nicht hinterlegt. Bitte den QR-Code prüfen oder einen anderen Sensor scannen.'
              : 'Der Abgleich mit der Datenbank ist fehlgeschlagen. Bitte prüfe deine Internetverbindung und versuche es erneut.'}
          </p>
        </header>

        <div className="overflow-hidden rounded-2xl border border-red-200 bg-red-50/50">
          <div className="flex items-center gap-4 p-5 md:p-6">
            <div
              className="flex size-12 shrink-0 items-center justify-center rounded-full bg-red-100 text-red"
              aria-hidden
            >
              {notFound ? <ScanSearch className="size-6" /> : <WifiOff className="size-6" />}
            </div>
            <div className="min-w-0 flex-1 space-y-1">
              <p className="text-[11px] uppercase tracking-wider font-semibold text-red/80">
                Gescannte Sensor-ID
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {scannedSensorId}
              </p>
            </div>
          </div>
          <div className="border-t border-red-200/70 bg-red-50/80 px-5 py-3 md:px-6">
            <p className="text-xs leading-relaxed text-red/90">
              {notFound
                ? 'Vergewissere dich, dass die Sensoreinheit über die Verwaltung im System registriert wurde, oder kontaktiere den Admin.'
                : 'Sobald die Verbindung wieder steht, kannst du die Prüfung erneut anstoßen.'}
            </p>
          </div>
        </div>

        <div className="flex flex-col gap-2 sm:flex-row">
          {!notFound && (
            <Button onClick={onRetryLookup} className="w-full sm:w-auto">
              <RotateCw className="size-4" />
              Erneut prüfen
            </Button>
          )}
          <Button
            variant={notFound ? 'default' : 'outline'}
            onClick={onScanAgain}
            className="w-full sm:w-auto"
          >
            <RotateCw className="size-4" />
            Anderen Sensor scannen
          </Button>
        </div>
      </div>
    )
  }

  if (sensor && sensor.status !== 'prepared') {
    const isOnline = sensor.status === 'online'
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">Sensor nicht aktivierbar</h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            {isOnline
              ? 'Dieser Sensor ist bereits aktiviert und einem Baum zugeordnet. Bitte einen anderen Sensor scannen.'
              : 'Dieser Sensor wurde bereits aktiviert und ist derzeit offline. Eine erneute Aktivierung ist nicht möglich.'}
          </p>
        </header>

        <div className="overflow-hidden rounded-2xl border border-yellow-200 bg-yellow-50">
          <div className="flex items-center gap-4 p-5 md:p-6">
            <div
              className="flex size-12 shrink-0 items-center justify-center rounded-full bg-yellow-100 text-yellow-900"
              aria-hidden
            >
              <AlertTriangle className="size-6" />
            </div>
            <div className="min-w-0 flex-1 space-y-1">
              <p className="text-[11px] uppercase tracking-wider font-semibold text-yellow-900/80">
                Status: {isOnline ? 'Online' : 'Offline'}
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {sensor.id}
              </p>
            </div>
          </div>
          {sensor.latestData?.createdAt && (
            <div className="border-t border-yellow-200/70 bg-yellow-100/40 px-5 py-3 md:px-6">
              <p className="text-xs text-yellow-900/90">
                Zuletzt gesehen: {formatLatestData(sensor.latestData.createdAt)}
              </p>
            </div>
          )}
        </div>

        <Button onClick={onScanAgain} className="w-full sm:w-auto">
          <RotateCw className="size-4" />
          Anderen Sensor scannen
        </Button>
      </div>
    )
  }

  if (sensor) {
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">Sensor erkannt</h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            Der Sensor ist im System bekannt und zur Aktivierung freigegeben. Tippe auf „Weiter", um
            mit dem GPS-Standort fortzufahren, oder scanne einen anderen Sensor.
          </p>
        </header>

        <div className="rounded-2xl border border-green-dark/30 bg-green-dark-50/30 p-4 md:p-5 space-y-3">
          <div className="flex items-center gap-2 text-sm font-medium text-green-dark">
            <CheckCircle2 className="size-4" aria-hidden />
            <span>Status: Bereit zur Aktivierung</span>
          </div>
          <CopyableText value={sensor.id} label="Sensor-ID" />
        </div>

        <Button variant="outline" onClick={onScanAgain} className="w-full sm:w-auto">
          <RotateCw className="size-4" />
          Anderen Sensor scannen
        </Button>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <ScannerHeader />
      <QRScannerView continueLabel="Sensor übernehmen" onContinue={onScanned} />
    </div>
  )
}

export default SensorScanStep
