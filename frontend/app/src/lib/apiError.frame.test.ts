import { ResponseError } from '@green-ecolution/backend-client'
import { beforeEach, expect, it } from 'vitest'
import { createI18n, getI18n } from './i18n'
import { resolveApiError } from './apiError'

beforeEach(async () => {
  localStorage.clear()
  await createI18n()
})

function errorWith(status: number, body: unknown): ResponseError {
  const response = new Response(JSON.stringify(body), {
    status,
    headers: { 'content-type': 'application/json' },
  })
  return new ResponseError(response, 'Response returned an error code')
}

it('prefers the named code over the status', async () => {
  const info = await resolveApiError(
    errorWith(409, { error: 'conflict', code: 'conflict.tree_in_cluster' }),
  )
  expect(info.message).toBe(
    'Der Baum gehört zu einer Bewässerungsgruppe und muss erst daraus entfernt werden.',
  )
  expect(info.messageKey).toBe('code.conflict.tree_in_cluster')
})

it('points messageKey at the validation catalog for field errors', async () => {
  const info = await resolveApiError(
    errorWith(422, {
      error: 'validation failed',
      code: 'request.validation_failed',
      validation: { path: 'species', field: 'species', key: 'tree.species.empty', params: {} },
    }),
  )
  expect(info.messageKey).toBe('validation:tree.species.empty')
  expect(info.message).toBe('Art ist erforderlich.')
})

it('renders a frame sentence with the resolved reason', async () => {
  const info = await resolveApiError(errorWith(403, { error: 'forbidden' }))
  expect(getI18n().t('errors:frame.wateringPlanStartFailed', { reason: info.message })).toBe(
    'Der Einsatzplan konnte nicht gestartet werden: Für diese Aktion fehlt dir die Berechtigung.',
  )
})
