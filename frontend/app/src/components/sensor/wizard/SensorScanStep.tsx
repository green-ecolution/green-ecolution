import QRScannerView from '@/components/scanner/QRScannerView'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { Button, CopyableText, Loading } from '@green-ecolution/ui'
import {
  AlertTriangle,
  Barcode,
  CheckCircle2,
  ChevronRight,
  RotateCw,
  ScanSearch,
  WifiOff,
} from 'lucide-react'

interface SensorScanStepProps {
  scannedSensorId: string | null
  isLookupLoading: boolean
  isLookupError: boolean
  lookupErrorStatus: number | null
  sensor: SensorResponse | null
  onScanned: (sensorId: string) => void
  onScanAgain: () => void
  onRetryLookup: () => void
  onContinue: () => void
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
  onContinue,
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
    const kind: 'notFound' | 'badRequest' | 'network' =
      lookupErrorStatus === 404
        ? 'notFound'
        : lookupErrorStatus === 400
          ? 'badRequest'
          : 'network'

    const content = {
      notFound: {
        icon: <ScanSearch className="size-8" />,
        title: 'Sensor nicht gefunden',
        description: 'Diese Sensor-ID ist im System nicht hinterlegt.',
        hint: 'Vergewissere dich, dass die Sensoreinheit über die Verwaltung im System registriert wurde, oder kontaktiere den Admin.',
      },
      badRequest: {
        icon: <Barcode className="size-8" />,
        title: 'Sensor-ID ungültig',
        description: 'Die gescannte ID entspricht nicht dem erwarteten Format.',
        hint: 'Bitte prüfe, ob du den korrekten QR-Code auf der Sensoreinheit gescannt hast.',
      },
      network: {
        icon: <WifiOff className="size-8" />,
        title: 'Abgleich fehlgeschlagen',
        description: 'Der Abgleich mit der Datenbank konnte nicht ausgeführt werden.',
        hint: 'Bitte prüfe deine Internetverbindung und versuche es erneut.',
      },
    }[kind]

    const showRetry = kind === 'network'

    return (
      <div className="mx-auto max-w-xl py-6 md:py-10">
        <div className="rounded-2xl border-2 border-red-200 bg-red-50/40 p-6 md:p-10 shadow-sm">
          <div className="flex flex-col items-center text-center space-y-5">
            <div
              className="flex size-16 items-center justify-center rounded-full bg-red text-white"
              aria-hidden
            >
              {content.icon}
            </div>

            <div className="space-y-2">
              <h2 className="font-lato font-bold text-2xl md:text-3xl text-foreground">
                {content.title}
              </h2>
              <p className="text-sm text-muted-foreground max-w-prose">{content.description}</p>
            </div>

            <div className="w-full max-w-sm rounded-xl border border-red-200/70 bg-background px-4 py-3 text-left">
              <p className="text-[10px] uppercase tracking-wider font-semibold text-red/80 mb-1">
                Gescannte ID
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {scannedSensorId}
              </p>
            </div>

            <p className="text-xs text-muted-foreground max-w-prose">{content.hint}</p>

            <div className="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-center">
              {showRetry && (
                <Button onClick={onRetryLookup} className="sm:min-w-[180px]">
                  <RotateCw className="size-4" />
                  Erneut prüfen
                </Button>
              )}
              <Button
                variant={showRetry ? 'outline' : 'default'}
                onClick={onScanAgain}
                className="sm:min-w-[200px]"
              >
                <RotateCw className="size-4" />
                Anderen Sensor scannen
              </Button>
            </div>
          </div>
        </div>
      </div>
    )
  }

  if (sensor && sensor.status !== 'prepared') {
    const isOnline = sensor.status === 'online'
    return (
      <div className="mx-auto max-w-xl py-6 md:py-10">
        <div className="rounded-2xl border-2 border-yellow-200 bg-yellow-50 p-6 md:p-10 shadow-sm">
          <div className="flex flex-col items-center text-center space-y-5">
            <div
              className="flex size-16 items-center justify-center rounded-full bg-yellow text-yellow-900"
              aria-hidden
            >
              <AlertTriangle className="size-8" />
            </div>

            <div className="space-y-2">
              <h2 className="font-lato font-bold text-2xl md:text-3xl text-foreground">
                Sensor nicht aktivierbar
              </h2>
              <p className="text-sm text-muted-foreground max-w-prose">
                {isOnline
                  ? 'Dieser Sensor ist bereits aktiviert und einem Baum zugeordnet.'
                  : 'Dieser Sensor wurde bereits aktiviert und ist derzeit offline.'}
              </p>
            </div>

            <div className="w-full max-w-sm rounded-xl border border-yellow-200/70 bg-background px-4 py-3 text-left space-y-1">
              <p className="text-[10px] uppercase tracking-wider font-semibold text-yellow-900/80">
                Status: {isOnline ? 'Online' : 'Offline'}
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {sensor.id}
              </p>
              {sensor.latestData?.createdAt && (
                <p className="text-xs text-muted-foreground pt-1">
                  Zuletzt gesehen: {formatLatestData(sensor.latestData.createdAt)}
                </p>
              )}
            </div>

            <Button onClick={onScanAgain} className="w-full sm:w-auto sm:min-w-[200px]">
              <RotateCw className="size-4" />
              Anderen Sensor scannen
            </Button>
          </div>
        </div>
      </div>
    )
  }

  if (sensor) {
    return (
      <div className="mx-auto max-w-xl py-6 md:py-10">
        <div className="rounded-2xl border-2 border-green-dark/30 bg-green-dark-50/40 p-6 md:p-10 shadow-sm">
          <div className="flex flex-col items-center text-center space-y-5">
            <div
              className="flex size-16 items-center justify-center rounded-full bg-green-dark text-white"
              aria-hidden
            >
              <CheckCircle2 className="size-8" />
            </div>

            <div className="space-y-2">
              <h2 className="font-lato font-bold text-2xl md:text-3xl text-foreground">
                Sensor erkannt
              </h2>
              <p className="text-sm text-muted-foreground max-w-prose">
                Im System bekannt und zur Aktivierung freigegeben.
              </p>
            </div>

            <div className="w-full max-w-sm rounded-xl border border-green-dark/30 bg-background px-4 py-3 text-left space-y-1">
              <p className="text-[10px] uppercase tracking-wider font-semibold text-green-dark/80">
                Sensor-ID · Status: Bereit
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {sensor.id}
              </p>
            </div>

            <div className="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-center">
              <Button variant="outline" onClick={onScanAgain} className="sm:min-w-[200px]">
                <RotateCw className="size-4" />
                Anderen Sensor scannen
              </Button>
              <Button onClick={onContinue} className="sm:min-w-[200px]">
                Weiter
                <ChevronRight className="size-4" />
              </Button>
            </div>
          </div>
        </div>
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
