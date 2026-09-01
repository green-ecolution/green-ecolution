import createToast from '@/hooks/createToast'
import {
  Button,
  Card,
  CardContent,
  CardFooter,
  CardHeader,
  CardTitle,
  CopyableText,
} from '@green-ecolution/ui'
import { ArrowRight, CheckCircle2, RotateCcw } from 'lucide-react'
import { useTranslation } from 'react-i18next'

interface QRScanResultProps {
  sensorId: string
  onScanAgain: () => void
  /** Label for the primary continue button. Defaults to `common:actions.next`. */
  continueLabel?: string
  /** If provided, called instead of the default placeholder toast */
  onContinue?: (sensorId: string) => void
  /** Optional extra block rendered below the sensor-ID (e.g. GPS readout). */
  extra?: React.ReactNode
}

const QRScanResult = ({
  sensorId,
  onScanAgain,
  continueLabel,
  onContinue,
  extra,
}: QRScanResultProps) => {
  const { t } = useTranslation('common')
  const showToast = createToast()
  const resolvedContinueLabel = continueLabel ?? t('actions.next')

  const handleContinue = () => {
    if (onContinue) {
      onContinue(sensorId)
    } else {
      showToast(t('scanner.result.toastNotImplemented'), 'success')
    }
  }

  return (
    <Card variant="outlined" className="animate-in fade-in-0 slide-in-from-bottom-2 duration-300">
      <CardHeader>
        <div className="flex items-center gap-2">
          <CheckCircle2 aria-hidden="true" className="size-5 text-green-dark" />
          <CardTitle className="text-xl">{t('scanner.result.title')}</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <CopyableText
          value={sensorId}
          label={t('scanner.result.idLabel')}
          onCopy={() => showToast(t('scanner.result.idCopied'), 'success')}
          onCopyError={() => showToast(t('scanner.result.idCopyFailed'), 'error')}
        />
        {extra}
      </CardContent>
      <CardFooter className="flex flex-col-reverse sm:flex-row sm:justify-end gap-2">
        <Button variant="outline" size="sm" onClick={onScanAgain} className="w-full sm:w-auto">
          <RotateCcw />
          {t('scanner.result.scanAgain')}
        </Button>
        <Button
          variant="default"
          size="sm"
          onClick={handleContinue}
          autoFocus
          className="w-full sm:w-auto"
        >
          {resolvedContinueLabel}
          <ArrowRight />
        </Button>
      </CardFooter>
    </Card>
  )
}

export default QRScanResult
