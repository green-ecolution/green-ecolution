import { useState, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { CircleCheck, ChevronRight } from 'lucide-react'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
  cn,
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
import {
  useDataQualityDetails,
  useQualityReasonLabel,
} from '@/hooks/details/useDetailsForDataHealth'
import { useDateLocale } from '@/lib/i18n/useFormatters'

interface SensorDataQualitySectionProps {
  sensorId: string
}

const COLLAPSED_ISSUE_COUNT = 5

const IssueList = ({ issues }: { issues: SensorQualityIssueResponse[] }) => {
  const { t } = useTranslation('sensor')
  const getQualityReasonLabel = useQualityReasonLabel()
  const dateLocale = useDateLocale()
  const [expanded, setExpanded] = useState(false)
  const isCollapsible = issues.length > COLLAPSED_ISSUE_COUNT
  const visibleIssues = expanded ? issues : issues.slice(0, COLLAPSED_ISSUE_COUNT)
  return (
    <div className="flex flex-col gap-1">
      <ul className="divide-y divide-dark-100 rounded-lg border border-dark-100 bg-white text-sm">
        {visibleIssues.map((issue) => (
          <li
            key={`${issue.recordedAt}-${issue.ability}-${issue.depthCm}`}
            className="flex flex-wrap items-baseline justify-between gap-x-4 gap-y-0.5 px-3 py-2"
          >
            <span className="font-semibold tabular-nums">
              {format(new Date(issue.recordedAt), 'dd.MM.yyyy HH:mm', { locale: dateLocale })} ·{' '}
              {t('dataQuality.issueDepth', { depth: issue.depthCm })}
            </span>
            <span className="text-dark-800">
              {t('dataQuality.issueReading', { value: issue.value })} ·{' '}
              {getQualityReasonLabel(issue.reason)}
            </span>
          </li>
        ))}
      </ul>
      {isCollapsible && (
        <Button
          variant="ghost"
          size="sm"
          className="w-fit"
          aria-expanded={expanded}
          onClick={() => setExpanded((value) => !value)}
        >
          {expanded
            ? t('dataQuality.showFewerIssues')
            : t('dataQuality.showAllIssues', { count: issues.length })}
        </Button>
      )}
    </div>
  )
}

interface IssueDisclosureProps {
  summary: string
  className?: string
  children: ReactNode
}

const IssueDisclosure = ({ summary, className, children }: IssueDisclosureProps) => (
  <details className={cn('group mt-3', className)}>
    <summary className="inline-flex min-h-11 cursor-pointer list-none items-center gap-1 text-sm font-medium text-foreground [&::-webkit-details-marker]:hidden">
      <ChevronRight
        className="size-4 transition-transform duration-base group-open:rotate-90 motion-reduce:transition-none"
        aria-hidden
      />
      {summary}
    </summary>
    <div className="mt-1">{children}</div>
  </details>
)

const SensorDataQualitySection = ({ sensorId }: SensorDataQualitySectionProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const dateLocale = useDateLocale()
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

  const acknowledgedLine =
    data.acknowledged &&
    `${t('dataQuality.acknowledgedBy', {
      name: data.acknowledged.byName ?? t('dataQuality.unknownReviewer'),
      date: format(new Date(data.acknowledged.at), 'dd.MM.yyyy HH:mm', { locale: dateLocale }),
    })}${data.acknowledged.note ? `: ${data.acknowledged.note}` : ''}`

  // Nothing left to review: only the history remains, which should not compete with the KPIs.
  if (pending.length === 0 && quality.alert === 'success') {
    return (
      <section
        aria-label={t('dataQuality.title')}
        className="flex flex-wrap items-center gap-x-3 rounded-xl border border-dark-100 bg-white px-4 py-1 text-sm"
      >
        <span className="inline-flex items-center gap-1.5 font-semibold">
          <CircleCheck className="size-4 text-green-dark" aria-hidden />
          {quality.label}
        </span>
        {acknowledgedLine && <span className="text-muted-foreground">{acknowledgedLine}</span>}
        <IssueDisclosure
          summary={t('dataQuality.pastIssues', { count: reviewed.length })}
          className="mt-0 open:basis-full sm:ml-auto sm:open:ml-0"
        >
          <div className="pb-2">
            <IssueList issues={reviewed} />
          </div>
        </IssueDisclosure>
      </section>
    )
  }

  return (
    <section aria-label={t('dataQuality.title')}>
      <Alert variant={quality.alert} className="flex w-full items-start gap-3">
        <AlertIcon variant={quality.alert} />
        <AlertContent className="min-w-0 flex-1">
          <AlertTitle>{quality.label}</AlertTitle>
          <AlertDescription>
            {quality.description}{' '}
            {data.implausibleRecent > 0 &&
              t('dataQuality.discardedSummary', { count: data.implausibleRecent })}
          </AlertDescription>
          {acknowledgedLine && (
            <AlertDescription className="mt-1 italic">{acknowledgedLine}</AlertDescription>
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
          {pending.length > 0 && (
            <IssueDisclosure summary={t('dataQuality.pendingIssues', { count: pending.length })}>
              <IssueList issues={pending} />
            </IssueDisclosure>
          )}
          {reviewed.length > 0 && (
            <IssueDisclosure summary={t('dataQuality.pastIssues', { count: reviewed.length })}>
              <IssueList issues={reviewed} />
            </IssueDisclosure>
          )}
        </AlertContent>
      </Alert>

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
    </section>
  )
}

export default SensorDataQualitySection
