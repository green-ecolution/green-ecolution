import { useId, useMemo, useState } from 'react'
import { Controller, useForm, useWatch } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  FormField,
  Label,
  SegmentedControl,
  Switch,
} from '@green-ecolution/ui'
import type { DataExportTargetResponse } from '@/api/backendApi'
import { formValuesFrom, type DataExportFormValues } from '@/api/dataExport'
import { resolveApiError } from '@/lib/apiError'
import { dataExportSchema, keepsStoredSecret } from './dataExportSchema'

interface DataExportDialogProps {
  open: boolean
  /** The stored target when editing; absent while setting one up. */
  target?: DataExportTargetResponse
  onOpenChange: (open: boolean) => void
  /** Rejects with the request error, which the dialog shows itself. */
  onSubmit: (values: DataExportFormValues) => Promise<unknown>
}

type FormProps = Omit<DataExportDialogProps, 'open' | 'onOpenChange'> & {
  onDone: () => void
  onCancel: () => void
}

const DataExportForm = ({ target, onSubmit, onDone, onCancel }: FormProps) => {
  const { t } = useTranslation('settings')
  const ids = useId()
  const [serverError, setServerError] = useState<string | null>(null)

  const schema = useMemo(() => dataExportSchema(t, target), [t, target])
  const { register, control, handleSubmit, setError, formState } = useForm<DataExportFormValues>({
    resolver: zodResolver(schema),
    defaultValues: formValuesFrom(target),
  })
  const { errors, isSubmitting } = formState

  const [authKind, baseUrl, tokenUrl] = useWatch({
    control,
    name: ['authKind', 'baseUrl', 'tokenUrl'],
  })
  const secretKept = keepsStoredSecret({ authKind, baseUrl, tokenUrl }, target)
  const secretHint = secretKept ? t('organization.dataExport.dialog.secretKept') : undefined

  const submit = handleSubmit(async (values) => {
    setServerError(null)
    try {
      await onSubmit(values)
      onDone()
    } catch (error) {
      const info = await resolveApiError(error)
      if (info.code === 'data_export.url_not_allowed') {
        setError('baseUrl', { type: 'server', message: info.message })
      } else if (info.code === 'data_export.auth_incomplete') {
        setError(values.authKind === 'basic' ? 'password' : 'clientSecret', {
          type: 'server',
          message: info.message,
        })
      } else {
        setServerError(info.message)
      }
    }
  })

  const authOptions = [
    { value: 'none' as const, label: t('organization.dataExport.auth.none') },
    { value: 'basic' as const, label: t('organization.dataExport.auth.basic') },
    {
      value: 'oauth2_client_credentials' as const,
      label: t('organization.dataExport.auth.oauth2'),
    },
  ]

  return (
    <form onSubmit={submit} className="flex min-h-0 flex-col gap-4" noValidate>
      <Controller
        control={control}
        name="enabled"
        render={({ field }) => (
          <div className="flex items-center justify-between gap-4">
            <Label htmlFor={`${ids}-enabled`}>{t('organization.dataExport.dialog.enabled')}</Label>
            <Switch id={`${ids}-enabled`} checked={field.value} onCheckedChange={field.onChange} />
          </div>
        )}
      />

      <FormField
        id={`${ids}-base-url`}
        type="url"
        inputMode="url"
        autoComplete="off"
        spellCheck={false}
        label={t('organization.dataExport.dialog.baseUrl')}
        description={t('organization.dataExport.dialog.baseUrlHint')}
        error={errors.baseUrl?.message}
        {...register('baseUrl')}
      />

      <Controller
        control={control}
        name="authKind"
        render={({ field }) => (
          <div className="flex flex-col gap-2">
            <span className="text-sm font-medium">
              {t('organization.dataExport.dialog.authKind')}
            </span>
            <SegmentedControl
              options={authOptions}
              value={field.value}
              onChange={field.onChange}
              ariaLabel={t('organization.dataExport.dialog.authKind')}
              size="sm"
              className="flex-wrap self-start"
            />
          </div>
        )}
      />

      {authKind === 'basic' && (
        <>
          <FormField
            id={`${ids}-username`}
            autoComplete="off"
            label={t('organization.dataExport.dialog.username')}
            error={errors.username?.message}
            {...register('username')}
          />
          <FormField
            id={`${ids}-password`}
            type="password"
            autoComplete="new-password"
            label={t('organization.dataExport.dialog.password')}
            description={secretHint}
            error={errors.password?.message}
            {...register('password')}
          />
        </>
      )}

      {authKind === 'oauth2_client_credentials' && (
        <>
          <FormField
            id={`${ids}-token-url`}
            type="url"
            inputMode="url"
            autoComplete="off"
            spellCheck={false}
            label={t('organization.dataExport.dialog.tokenUrl')}
            error={errors.tokenUrl?.message}
            {...register('tokenUrl')}
          />
          <FormField
            id={`${ids}-client-id`}
            autoComplete="off"
            label={t('organization.dataExport.dialog.clientId')}
            error={errors.clientId?.message}
            {...register('clientId')}
          />
          <FormField
            id={`${ids}-client-secret`}
            type="password"
            autoComplete="new-password"
            label={t('organization.dataExport.dialog.clientSecret')}
            description={secretHint}
            error={errors.clientSecret?.message}
            {...register('clientSecret')}
          />
          <FormField
            id={`${ids}-scope`}
            autoComplete="off"
            label={t('organization.dataExport.dialog.scope')}
            error={errors.scope?.message}
            {...register('scope')}
          />
        </>
      )}

      {serverError && (
        <Alert variant="destructive" className="flex w-full gap-3">
          <AlertIcon variant="destructive" />
          <AlertContent>
            <AlertDescription>{serverError}</AlertDescription>
          </AlertContent>
        </Alert>
      )}

      <DialogFooter>
        <Button type="button" variant="outline" onClick={onCancel}>
          {t('organization.dataExport.dialog.cancel')}
        </Button>
        <Button type="submit" disabled={isSubmitting}>
          {t('organization.dataExport.dialog.save')}
        </Button>
      </DialogFooter>
    </form>
  )
}

const DataExportDialog = ({ open, onOpenChange, ...formProps }: DataExportDialogProps) => {
  const { t } = useTranslation('settings')

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[calc(100dvh-3rem)] max-w-xl overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {formProps.target
              ? t('organization.dataExport.dialog.editTitle')
              : t('organization.dataExport.dialog.createTitle')}
          </DialogTitle>
          <DialogDescription>{t('organization.dataExport.hint')}</DialogDescription>
        </DialogHeader>
        <DataExportForm
          {...formProps}
          onDone={() => onOpenChange(false)}
          onCancel={() => onOpenChange(false)}
        />
      </DialogContent>
    </Dialog>
  )
}

export default DataExportDialog
