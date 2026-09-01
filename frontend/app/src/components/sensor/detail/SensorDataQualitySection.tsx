import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Textarea,
} from '@green-ecolution/ui'
import type { SensorQualityIssueResponse } from '@/api/backendApi'
import { sensorQueries } from '@/api/queries'
import { Can } from '@/lib/auth/Can'
import { useSensorQualityMutations } from '@/hooks/useSensorQualityMutations'
import { useDataQualityDetails, useQualityReasonLabel } from '@/hooks/details/useDetailsForDataHealth'

interface SensorDataQualitySectionProps {
  sensorId: string
}

const IssueList = ({ issues }: { issues: SensorQualityIssueResponse[] }) => {
  const { t } = useTranslation('sensor')
  const getQualityReasonLabel = useQualityReasonLabel()
  return (
    <ul className="flex flex-col gap-2">
      {issues.map((issue) => (
        <li
          key={`${issue.recordedAt}-${issue.ability}-${issue.depthCm}`}
          className="rounded-lg border border-dark-50 bg-white p-3 text-sm"
        >
          <p className="font-bold">
            {format(new Date(issue.recordedAt), 'dd.MM.yyyy HH:mm')} ·{' '}
            {t('dataQuality.issueDepth', { depth: issue.depthCm })}
          </p>
          <p className="text-dark-800">
            {t('dataQuality.issueReading', { value: issue.value })} ·{' '}
            {getQualityReasonLabel(issue.reason)}
          </p>
        </li>
      ))}
    </ul>
  )
}

const SensorDataQualitySection = ({ sensorId }: SensorDataQualitySectionProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const [dialogOpen, setDialogOpen] = useState(false)
  const [note, setNote] = useState('')
  const { data } = useQuery(sensorQueries.dataQuality(sensorId))
  const { acknowledge } = useSensorQualityMutations()

  const getDataQualityDetails = useDataQualityDetails()

  if (!data || data.issues.length === 0) return null

  const quality = getDataQualityDetails({
    dataHealth: data.health,
    implausibleRecent: data.implausibleRecent,
  })
  const acknowledgedAt = data.acknowledged ? new Date(data.acknowledged.at).getTime() : null
  const isReviewed = (issue: SensorQualityIssueResponse) =>
    acknowledgedAt !== null && new Date(issue.recordedAt).getTime() <= acknowledgedAt
  const pending = data.issues.filter((issue) => !isReviewed(issue))
  const reviewed = data.issues.filter(isReviewed)

  const submit = () => {
    acknowledge.mutate(
      { sensorId, note },
      {
        onSuccess: () => {
          setDialogOpen(false)
          setNote('')
        },
      },
    )
  }

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('dataQuality.title')}</CardTitle>
      </CardHeader>
      <CardContent>
        <Alert variant={quality.alert} className="mb-4 flex w-full items-start gap-3">
          <AlertIcon variant={quality.alert} />
          <AlertContent>
            <AlertTitle>{quality.label}</AlertTitle>
            <AlertDescription>
              {quality.description}{' '}
              {data.implausibleRecent > 0 &&
                t('dataQuality.discardedSummary', { count: data.implausibleRecent })}
            </AlertDescription>
            {data.acknowledged && (
              <AlertDescription className="mt-1 italic">
                {t('dataQuality.acknowledgedBy', {
                  name: data.acknowledged.byName ?? t('dataQuality.unknownReviewer'),
                  date: format(new Date(data.acknowledged.at), 'dd.MM.yyyy HH:mm'),
                })}
                {data.acknowledged.note ? `: ${data.acknowledged.note}` : ''}
              </AlertDescription>
            )}
            {pending.length > 0 && (
              <Can permission={['sensor:update']}>
                <Button
                  variant="outline"
                  size="sm"
                  className="mt-3 w-fit"
                  onClick={() => setDialogOpen(true)}
                >
                  {t('dataQuality.acknowledgeButton')}
                </Button>
              </Can>
            )}
          </AlertContent>
        </Alert>

        {pending.length > 0 && <IssueList issues={pending} />}

        {reviewed.length > 0 && (
          <details className={pending.length > 0 ? 'mt-4' : undefined}>
            <summary className="cursor-pointer text-sm text-muted-foreground">
              {t('dataQuality.pastIssues', { count: reviewed.length })}
            </summary>
            <div className="mt-2">
              <IssueList issues={reviewed} />
            </div>
          </details>
        )}
      </CardContent>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('dataQuality.acknowledgeDialogTitle')}</DialogTitle>
            <DialogDescription>{t('dataQuality.acknowledgeDialogDescription')}</DialogDescription>
          </DialogHeader>
          <Textarea
            value={note}
            onChange={(event) => setNote(event.target.value)}
            placeholder={t('dataQuality.acknowledgeNotePlaceholder')}
            maxLength={500}
          />
          <DialogFooter>
            <Button variant="outline" onClick={() => setDialogOpen(false)}>
              {t('common:actions.cancel')}
            </Button>
            <Button onClick={submit} disabled={acknowledge.isPending}>
              {t('dataQuality.acknowledgeSubmit')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </Card>
  )
}

export default SensorDataQualitySection
