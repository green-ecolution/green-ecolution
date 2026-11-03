import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertIcon,
  Badge,
  Button,
  Loading,
  type BadgeProps,
} from '@green-ecolution/ui'
import type {
  ConnectionTestResponse,
  DataExportAuthResponse,
  DataExportTargetResponse,
} from '@/api/backendApi'
import { dataExportQueries, type DataExportFormValues } from '@/api/dataExport'
import { useDataExportMutations } from '@/hooks/useDataExportMutations'
import { resolveApiError } from '@/lib/apiError'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useDateLocale, useNumberFormatter } from '@/lib/i18n/useFormatters'
import { CARD, CARD_TITLE } from '../OrganizationDetail'
import DataExportDialog from './DataExportDialog'

type ExportStatus = 'active' | 'paused' | 'failing'

const STATUS_BADGE: Record<ExportStatus, BadgeProps['variant']> = {
  active: 'success',
  paused: 'muted',
  failing: 'error',
}

const statusOf = (target: DataExportTargetResponse): ExportStatus => {
  if (!target.enabled) return 'paused'
  return target.status.lastError ? 'failing' : 'active'
}

const authSummary = (auth: DataExportAuthResponse, t: TFunction<'settings'>): string => {
  switch (auth.kind) {
    case 'none':
      return t('organization.dataExport.auth.none')
    case 'basic':
      return [
        t('organization.dataExport.auth.basic'),
        auth.username && t('organization.dataExport.auth.user', { name: auth.username }),
      ]
        .filter(Boolean)
        .join(' · ')
    case 'oauth2_client_credentials':
      return [
        t('organization.dataExport.auth.oauth2'),
        auth.clientId && t('organization.dataExport.auth.client', { name: auth.clientId }),
      ]
        .filter(Boolean)
        .join(' · ')
  }
}

type TestResult =
  { kind: 'outcome'; response: ConnectionTestResponse } | { kind: 'failed'; message: string }

const testResultView = (
  result: TestResult,
  t: TFunction<'settings'>,
): { variant: 'success' | 'warning' | 'destructive'; text: string } => {
  if (result.kind === 'failed') return { variant: 'destructive', text: result.message }
  const { outcome, dataArraySupported } = result.response
  if (outcome !== 'ok') {
    return { variant: 'destructive', text: t(`organization.dataExport.outcome.${outcome}`) }
  }
  return dataArraySupported
    ? { variant: 'success', text: t('organization.dataExport.outcome.ok') }
    : { variant: 'warning', text: t('organization.dataExport.outcome.okWithoutDataArray') }
}

interface DataExportCardProps {
  organizationId: string
}

const DataExportCard = ({ organizationId }: DataExportCardProps) => {
  const { t } = useTranslation('settings')
  const locale = useDateLocale()
  const numbers = useNumberFormatter()
  const canUpdate = useHasPermission(['setting:update'])
  const {
    data: state,
    isLoading,
    error,
  } = useQuery(dataExportQueries.byOrganization(organizationId))
  const { save, remove, test } = useDataExportMutations(organizationId)

  const [dialogOpen, setDialogOpen] = useState(false)
  const [confirmRemove, setConfirmRemove] = useState(false)
  const [testResult, setTestResult] = useState<TestResult | null>(null)

  if (isLoading) {
    return <Loading size="sm" role="status" aria-label={t('organization.dataExport.loading')} />
  }

  if (error) {
    return (
      <p role="alert" className="text-sm text-dark-600">
        {t('organization.dataExport.loadError')}
      </p>
    )
  }

  if (!state || state.kind === 'disabled') return null

  const target = state.kind === 'configured' ? state.target : undefined

  const runTest = () => {
    setTestResult(null)
    test.mutate(undefined, {
      onSuccess: (response) => setTestResult({ kind: 'outcome', response }),
      onError: (testError) => {
        void resolveApiError(testError).then((info) =>
          setTestResult({ kind: 'failed', message: info.message }),
        )
      },
    })
  }

  const submit = async (values: DataExportFormValues) => {
    await save.mutateAsync(values)
    setTestResult(null)
  }

  const runLine = (): string => {
    const { lastRunAt, lastRunStats } = target?.status ?? {}
    if (!lastRunAt || Number.isNaN(lastRunAt.getTime())) {
      return t('organization.dataExport.neverRun')
    }
    const parts = [
      t('organization.dataExport.lastRun', {
        time: format(lastRunAt, 'dd.MM.yyyy HH:mm', { locale }),
      }),
    ]
    if (lastRunStats) {
      parts.push(
        t('organization.dataExport.stats', {
          things: numbers.format(lastRunStats.thingsWritten),
          observations: numbers.format(lastRunStats.observationsWritten),
        }),
      )
      if (lastRunStats.skipped > 0) {
        parts.push(
          t('organization.dataExport.skipped', { count: numbers.format(lastRunStats.skipped) }),
        )
      }
    }
    return parts.join(' · ')
  }

  const result = testResult && testResultView(testResult, t)
  const status = target && statusOf(target)

  return (
    <section className={CARD}>
      <div className="flex items-start justify-between gap-3">
        <h3 className={CARD_TITLE}>{t('organization.dataExport.title')}</h3>
        {status && (
          <Badge variant={STATUS_BADGE[status]}>
            {t(`organization.dataExport.status.${status}`)}
          </Badge>
        )}
      </div>
      <p className="mt-2 text-sm text-dark-600">{t('organization.dataExport.hint')}</p>

      {target ? (
        <div className="mt-4 flex flex-col gap-1 text-sm">
          <p className="font-mono break-all text-dark">{target.baseUrl}</p>
          <p className="text-dark-600">
            {t('organization.dataExport.auth.label', { summary: authSummary(target.auth, t) })}
          </p>
          <p className="text-dark-600">{runLine()}</p>
          {target.status.lastError && (
            <Alert variant="destructive" size="compact" className="mt-2 flex w-full gap-2">
              <AlertIcon variant="destructive" />
              <AlertContent>
                <AlertDescription className="break-words">
                  {t('organization.dataExport.lastError', { error: target.status.lastError })}
                </AlertDescription>
              </AlertContent>
            </Alert>
          )}
        </div>
      ) : (
        !canUpdate && (
          <p className="mt-4 text-sm text-dark-600">{t('organization.dataExport.empty')}</p>
        )
      )}

      {canUpdate && (
        <div className="mt-4 flex flex-wrap gap-2">
          {target ? (
            <>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={test.isPending}
                onClick={runTest}
              >
                {t('organization.dataExport.test')}
              </Button>
              <Button type="button" variant="outline" size="sm" onClick={() => setDialogOpen(true)}>
                {t('organization.dataExport.edit')}
              </Button>
              <Button
                type="button"
                variant="outline"
                size="sm"
                disabled={remove.isPending}
                onClick={() => setConfirmRemove(true)}
              >
                {t('organization.dataExport.remove')}
              </Button>
            </>
          ) : (
            <Button type="button" size="sm" onClick={() => setDialogOpen(true)}>
              {t('organization.dataExport.setUp')}
            </Button>
          )}
        </div>
      )}

      {result && (
        <Alert variant={result.variant} size="compact" className="mt-3 flex w-full gap-2">
          <AlertIcon variant={result.variant} />
          <AlertContent>
            <AlertDescription>{result.text}</AlertDescription>
          </AlertContent>
        </Alert>
      )}

      <DataExportDialog
        open={dialogOpen}
        target={target}
        onOpenChange={setDialogOpen}
        onSubmit={submit}
      />

      <AlertDialog open={confirmRemove} onOpenChange={setConfirmRemove}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('organization.dataExport.removeDialog.title')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('organization.dataExport.removeDialog.description')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t('organization.dataExport.removeDialog.cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                setTestResult(null)
                remove.mutate()
              }}
            >
              {t('organization.dataExport.removeDialog.confirm')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}

export default DataExportCard
