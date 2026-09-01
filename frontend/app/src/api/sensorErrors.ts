import { getI18n } from '@/lib/i18n'

// These flows map by status alone because they run outside a form and cannot
// await the response body. Activate and re-link answer 422 only for a sensor
// and tree in different organizations, so the status is unambiguous there.
// Read lazily: the i18n instance isn't ready yet at module-import time.
const orgMismatch = (): string => getI18n().t('errors:code.organization_mismatch.sensor_vs_tree')

export const resolveResponseStatus = (err: unknown): number | null => {
  if (err instanceof Response) return err.status
  if (
    err != null &&
    typeof err === 'object' &&
    'response' in err &&
    err.response instanceof Response
  )
    return err.response.status
  return null
}

export const mapActivateError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return getI18n().t('errors:sensorFlow.activate.notFound')
  if (status === 409) return getI18n().t('errors:sensorFlow.activate.conflict')
  if (status === 422) return orgMismatch()
  return getI18n().t('errors:sensorFlow.activate.failed')
}

export const mapReassignError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return getI18n().t('errors:sensorFlow.reassign.notFound')
  if (status === 409) return getI18n().t('errors:sensorFlow.reassign.conflict')
  if (status === 422) return orgMismatch()
  return getI18n().t('errors:sensorFlow.reassign.failed')
}

export const mapDeactivateError = (err: unknown): string => {
  const status = resolveResponseStatus(err)
  if (status === 404) return getI18n().t('errors:sensorFlow.deactivate.notFound')
  return getI18n().t('errors:sensorFlow.deactivate.failed')
}
