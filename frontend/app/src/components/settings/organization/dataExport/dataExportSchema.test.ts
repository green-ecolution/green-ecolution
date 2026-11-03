import { describe, it, expect } from 'vitest'
import type { TFunction } from 'i18next'
import type { DataExportTargetResponse } from '@/api/backendApi'
import { formValuesFrom, type DataExportFormValues } from '@/api/dataExport'
import { dataExportSchema } from './dataExportSchema'

const t = ((key: string) => key) as unknown as TFunction<'settings'>

const ERRORS = 'organization.dataExport.dialog.errors'

const basicTarget: DataExportTargetResponse = {
  activatedAt: new Date('2026-10-01T08:00:00Z'),
  auth: { kind: 'basic', hasSecret: true, username: 'alice' },
  baseUrl: 'https://frost.stadt.de/FROST-Server/v1.1',
  enabled: true,
  kind: 'sensorthings',
  organizationId: 'org-1',
  status: {},
}

const basic = (overrides: Partial<DataExportFormValues>): DataExportFormValues => ({
  ...formValuesFrom(),
  baseUrl: 'https://frost.stadt.de/v1.1',
  authKind: 'basic',
  username: 'alice',
  password: 'pw',
  ...overrides,
})

const issuesOf = (values: DataExportFormValues, stored?: DataExportTargetResponse) => {
  const result = dataExportSchema(t, stored).safeParse(values)
  return Object.fromEntries(
    (result.error?.issues ?? []).map((issue) => [issue.path.join('.'), issue.message]),
  )
}

describe('dataExportSchema', () => {
  it('accepts a secret of exactly 1024 UTF-8 bytes', () => {
    expect(issuesOf(basic({ password: 'ä'.repeat(512) }))).toEqual({})
  })

  it('counts a secret in UTF-8 bytes like the backend', () => {
    expect(issuesOf(basic({ password: 'ä'.repeat(513) }))).toEqual({
      password: `${ERRORS}.tooLong`,
    })
  })

  it('counts principal and scope in UTF-8 bytes', () => {
    expect(issuesOf(basic({ username: 'ä'.repeat(129) }))).toEqual({
      username: `${ERRORS}.tooLong`,
    })
    expect(
      issuesOf({
        ...basic({}),
        authKind: 'oauth2_client_credentials',
        tokenUrl: 'https://auth.stadt.de/token',
        clientId: 'client',
        clientSecret: 's3',
        scope: 'ä'.repeat(129),
      }),
    ).toEqual({ scope: `${ERRORS}.tooLong` })
  })

  it('counts the address in UTF-8 bytes', () => {
    const prefix = 'https://frost.stadt.de/'
    const fits = `${prefix}${'ä'.repeat(1012)}a`
    expect(new TextEncoder().encode(fits).length).toBe(2048)
    expect(issuesOf(basic({ baseUrl: fits }))).toEqual({})
    expect(issuesOf(basic({ baseUrl: `${fits}ä` }))).toEqual({ baseUrl: `${ERRORS}.tooLong` })
  })

  it('asks for no secret while the address itself is invalid', () => {
    expect(issuesOf(basic({ baseUrl: 'frost.stadt.de', password: '' }))).toEqual({
      baseUrl: `${ERRORS}.urlInvalid`,
    })
    expect(issuesOf(basic({ baseUrl: '', password: '' }), basicTarget)).toEqual({
      baseUrl: `${ERRORS}.urlRequired`,
    })
  })

  it('asks for no client secret while the token address is invalid', () => {
    expect(
      issuesOf({
        ...basic({}),
        authKind: 'oauth2_client_credentials',
        tokenUrl: 'not a url',
        clientId: 'client',
      }),
    ).toEqual({ tokenUrl: `${ERRORS}.urlInvalid` })
  })

  it('still asks for the secret once the addresses are valid', () => {
    expect(issuesOf(basic({ password: '' }))).toEqual({ password: `${ERRORS}.secretRequired` })
  })
})
