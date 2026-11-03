import { z } from 'zod'
import type { TFunction } from 'i18next'
import type { DataExportTargetResponse } from '@/api/backendApi'
import type { DataExportFormValues } from '@/api/dataExport'

const URL_MAX_LENGTH = 2048
const PRINCIPAL_MAX_LENGTH = 256
const SECRET_MAX_LENGTH = 1024
const SCOPE_MAX_LENGTH = 256

const parseUrl = (value: string): URL | null => {
  try {
    return new URL(value)
  } catch {
    return null
  }
}

// Checked on the raw text: `new URL` drops an empty `?` or `#`, which the
// backend would still refuse.
export const isExportUrl = (value: string): boolean => {
  if (value.includes('?') || value.includes('#')) return false
  const url = parseUrl(value)
  return (
    url !== null &&
    (url.protocol === 'http:' || url.protocol === 'https:') &&
    url.hostname !== '' &&
    url.username === '' &&
    url.password === ''
  )
}

const originOf = (value: string): string | null => parseUrl(value.trim())?.origin ?? null

/**
 * Mirrors the backend: a stored secret survives only while the auth kind and
 * every receiving origin stay the same, otherwise it would reach a new server.
 */
export const keepsStoredSecret = (
  values: Pick<DataExportFormValues, 'authKind' | 'baseUrl' | 'tokenUrl'>,
  stored: DataExportTargetResponse | undefined,
): boolean => {
  if (!stored?.auth.hasSecret || stored.auth.kind !== values.authKind) return false
  if (originOf(values.baseUrl) !== originOf(stored.baseUrl)) return false
  return (
    values.authKind !== 'oauth2_client_credentials' ||
    originOf(values.tokenUrl) === originOf(stored.auth.tokenUrl ?? '')
  )
}

export const dataExportSchema = (
  t: TFunction<'settings'>,
  stored: DataExportTargetResponse | undefined,
) =>
  z
    .object({
      enabled: z.boolean(),
      baseUrl: z.string(),
      authKind: z.enum(['none', 'basic', 'oauth2_client_credentials']),
      username: z.string(),
      password: z.string(),
      tokenUrl: z.string(),
      clientId: z.string(),
      clientSecret: z.string(),
      scope: z.string(),
    })
    .superRefine((values, ctx) => {
      const fail = (path: keyof DataExportFormValues, message: string) =>
        ctx.addIssue({ code: 'custom', path: [path], message })

      const checkUrl = (path: 'baseUrl' | 'tokenUrl', missing: string) => {
        const value = values[path].trim()
        if (value.length === 0) fail(path, missing)
        else if (value.length > URL_MAX_LENGTH)
          fail(path, t('organization.dataExport.dialog.errors.tooLong'))
        else if (!isExportUrl(value))
          fail(path, t('organization.dataExport.dialog.errors.urlInvalid'))
      }

      const checkPrincipal = (path: 'username' | 'clientId') => {
        const value = values[path].trim()
        if (value.length === 0) fail(path, t('organization.dataExport.dialog.errors.required'))
        else if (value.length > PRINCIPAL_MAX_LENGTH)
          fail(path, t('organization.dataExport.dialog.errors.tooLong'))
      }

      const checkSecret = (path: 'password' | 'clientSecret') => {
        const value = values[path]
        if (value.length > SECRET_MAX_LENGTH)
          fail(path, t('organization.dataExport.dialog.errors.tooLong'))
        else if (value.trim().length === 0 && !keepsStoredSecret(values, stored))
          fail(path, t('organization.dataExport.dialog.errors.secretRequired'))
      }

      checkUrl('baseUrl', t('organization.dataExport.dialog.errors.urlRequired'))

      switch (values.authKind) {
        case 'none':
          return
        case 'basic':
          checkPrincipal('username')
          checkSecret('password')
          return
        case 'oauth2_client_credentials':
          checkUrl('tokenUrl', t('organization.dataExport.dialog.errors.required'))
          checkPrincipal('clientId')
          checkSecret('clientSecret')
          if (values.scope.trim().length > SCOPE_MAX_LENGTH)
            fail('scope', t('organization.dataExport.dialog.errors.tooLong'))
      }
    })
