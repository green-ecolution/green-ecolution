import { useTranslation } from 'react-i18next'
import type { SubmissionState } from './state'
import { AccuracyBadge, Button, CopyableText, InlineAlert } from '@green-ecolution/ui'
import { Loader2, ShieldCheck, TreeDeciduous } from 'lucide-react'

interface SensorReviewStepProps {
  sensorId: string
  treeNumber: string
  treeSpecies: string
  position: {
    latitude: number
    longitude: number
    accuracy: number
    timestamp: number
  } | null
  status: SubmissionState
  errorMessage: string | null
  onActivate: () => void
}

const formatCoordinate = (n: number) => n.toFixed(5)

const SensorReviewStep = ({
  sensorId,
  treeNumber,
  treeSpecies,
  position,
  status,
  errorMessage,
  onActivate,
}: SensorReviewStepProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const isPending = status === 'pending'
  const isError = status === 'error'

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato font-bold text-3xl lg:text-4xl">
          {t('wizard.review.header.title')}
        </h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          {t('wizard.review.header.description')}
        </p>
      </header>

      <div className="grid gap-4 md:grid-cols-2">
        <CopyableText
          value={sensorId}
          label={t('common:scanner.result.idLabel')}
          className="md:col-span-2"
        />

        <div className="rounded-xl border border-dark-100 p-4">
          <p className="text-xs uppercase tracking-wide font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
            <TreeDeciduous className="size-3.5" aria-hidden />
            {t('wizard.review.assignedTreeLabel')}
          </p>
          <div className="space-y-1">
            <p className="font-semibold">{treeSpecies}</p>
            <p className="font-mono text-xs text-dark-600">{treeNumber}</p>
          </div>
        </div>

        {position && (
          <div className="rounded-xl border border-dark-100 p-4">
            <p className="text-xs uppercase tracking-wide font-semibold text-muted-foreground mb-2">
              {t('wizard.review.locationLabel')}
            </p>
            <p className="text-sm tabular-nums">
              {formatCoordinate(position.latitude)}, {formatCoordinate(position.longitude)}
            </p>
            <div className="mt-2">
              <AccuracyBadge accuracyMeters={position.accuracy} />
            </div>
          </div>
        )}
      </div>

      <div className="flex flex-col gap-3 sm:flex-row sm:items-center sm:gap-6">
        {isError && errorMessage && (
          <InlineAlert variant="destructive" description={errorMessage} />
        )}
        <Button onClick={onActivate} disabled={isPending} className="w-full sm:w-auto sm:ml-auto">
          {isPending ? (
            <>
              <Loader2 className="size-4 animate-spin" />
              {t('wizard.review.activatingLabel')}
            </>
          ) : (
            <>
              <ShieldCheck className="size-4" />
              {t('wizard.review.activateButton')}
            </>
          )}
        </Button>
      </div>
    </div>
  )
}

export default SensorReviewStep
