import { useTranslation } from 'react-i18next'
import QRScannerView from '@/components/scanner/QRScannerView'
import type { SensorResponse } from '@green-ecolution/backend-client'
import { Button, CopyableText, Loading } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { intlLocale } from '@/lib/i18n/format'
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

const formatLatestData = (iso: string, locale: string): string => {
  const date = new Date(iso)
  if (Number.isNaN(date.getTime())) return iso
  return new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(date)
}

const ScannerHeader = () => {
  const { t } = useTranslation('sensor')
  return (
    <header className="space-y-2">
      <h1 className="font-lato font-bold text-3xl lg:text-4xl">{t('wizard.scan.header.title')}</h1>
      <p className="text-sm text-muted-foreground max-w-prose">
        {t('wizard.scan.header.description')}
      </p>
    </header>
  )
}

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
  const { t, i18n } = useTranslation(['sensor', 'common'])
  const getSensorStatusDetails = useSensorStatusDetails()

  if (!scannedSensorId) {
    return (
      <div className="space-y-6">
        <ScannerHeader />
        <QRScannerView continueLabel={t('wizard.scan.continueLabel')} onContinue={onScanned} />
      </div>
    )
  }

  if (isLookupLoading) {
    return (
      <div className="space-y-6">
        <header className="space-y-2">
          <h1 className="font-lato font-bold text-3xl lg:text-4xl">
            {t('wizard.scan.checking.title')}
          </h1>
          <p className="text-sm text-muted-foreground max-w-prose">
            {t('wizard.scan.checking.description')}
          </p>
        </header>

        <div className="rounded-2xl border border-dark-100 bg-dark-50/40 p-4 md:p-5 space-y-4">
          <CopyableText value={scannedSensorId} label={t('common:scanner.result.idLabel')} />
          <Loading size="default" label={t('wizard.scan.checking.loadingLabel')} />
        </div>
      </div>
    )
  }

  if (isLookupError) {
    const kind: 'notFound' | 'badRequest' | 'network' =
      lookupErrorStatus === 404 ? 'notFound' : lookupErrorStatus === 400 ? 'badRequest' : 'network'

    const content = {
      notFound: {
        icon: <ScanSearch className="size-8" />,
        title: t('wizard.scan.error.notFound.title'),
        description: t('wizard.scan.error.notFound.description'),
        hint: t('wizard.scan.error.notFound.hint'),
      },
      badRequest: {
        icon: <Barcode className="size-8" />,
        title: t('wizard.scan.error.badRequest.title'),
        description: t('wizard.scan.error.badRequest.description'),
        hint: t('wizard.scan.error.badRequest.hint'),
      },
      network: {
        icon: <WifiOff className="size-8" />,
        title: t('wizard.scan.error.network.title'),
        description: t('wizard.scan.error.network.description'),
        hint: t('wizard.scan.error.network.hint'),
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
                {t('wizard.scan.error.scannedIdLabel')}
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
                  {t('wizard.scan.retryButton')}
                </Button>
              )}
              <Button
                variant={showRetry ? 'outline' : 'default'}
                onClick={onScanAgain}
                className="sm:min-w-[200px]"
              >
                <RotateCw className="size-4" />
                {t('wizard.scan.rescanButton')}
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
                {t('wizard.scan.notActivatable.title')}
              </h2>
              <p className="text-sm text-muted-foreground max-w-prose">
                {isOnline
                  ? t('wizard.scan.notActivatable.onlineDescription')
                  : t('wizard.scan.notActivatable.offlineDescription')}
              </p>
            </div>

            <div className="w-full max-w-sm rounded-xl border border-yellow-200/70 bg-background px-4 py-3 text-left space-y-1">
              <p className="text-[10px] uppercase tracking-wider font-semibold text-yellow-900/80">
                {t('wizard.scan.notActivatable.statusLabel', {
                  status: getSensorStatusDetails(sensor.status).label,
                })}
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {sensor.id}
              </p>
              {sensor.latestData?.createdAt && (
                <p className="text-xs text-muted-foreground pt-1">
                  {t('wizard.scan.notActivatable.lastSeenLabel', {
                    date: formatLatestData(sensor.latestData.createdAt, intlLocale(i18n.language)),
                  })}
                </p>
              )}
            </div>

            <Button onClick={onScanAgain} className="w-full sm:w-auto sm:min-w-[200px]">
              <RotateCw className="size-4" />
              {t('wizard.scan.rescanButton')}
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
                {t('wizard.scan.recognized.title')}
              </h2>
              <p className="text-sm text-muted-foreground max-w-prose">
                {t('wizard.scan.recognized.description')}
              </p>
            </div>

            <div className="w-full max-w-sm rounded-xl border border-green-dark/30 bg-background px-4 py-3 text-left space-y-1">
              <p className="text-[10px] uppercase tracking-wider font-semibold text-green-dark/80">
                {t('wizard.scan.recognized.statusLabel')}
              </p>
              <p className="font-mono text-sm md:text-base font-semibold text-foreground break-all">
                {sensor.id}
              </p>
            </div>

            <div className="flex w-full flex-col-reverse gap-2 sm:flex-row sm:justify-center">
              <Button variant="outline" onClick={onScanAgain} className="sm:min-w-[200px]">
                <RotateCw className="size-4" />
                {t('wizard.scan.rescanButton')}
              </Button>
              <Button onClick={onContinue} className="sm:min-w-[200px]">
                {t('common:actions.next')}
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
      <QRScannerView continueLabel={t('wizard.scan.continueLabel')} onContinue={onScanned} />
    </div>
  )
}

export default SensorScanStep
