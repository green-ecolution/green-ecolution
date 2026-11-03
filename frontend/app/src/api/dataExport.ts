import { queryOptions } from '@tanstack/react-query'
import {
  dataExportApi,
  ResponseError,
  type DataExportAuthRequest,
  type DataExportTargetRequest,
  type DataExportTargetResponse,
} from '@/api/backendApi'

export type DataExportState =
  { kind: 'disabled' } | { kind: 'none' } | { kind: 'configured'; target: DataExportTargetResponse }

export const dataExportQueries = {
  byOrganization: (orgId: string) =>
    queryOptions<DataExportState>({
      queryKey: ['organizations', orgId, 'data-export'],
      queryFn: async () => {
        try {
          const target = await dataExportApi.getDataExportTarget({ orgId })
          return { kind: 'configured', target }
        } catch (error) {
          if (error instanceof ResponseError) {
            if (error.response.status === 503) return { kind: 'disabled' }
            if (error.response.status === 404) return { kind: 'none' }
          }
          throw error
        }
      },
      retry: false,
    }),
}

export interface DataExportFormValues {
  enabled: boolean
  baseUrl: string
  authKind: 'none' | 'basic' | 'oauth2_client_credentials'
  username: string
  password: string
  tokenUrl: string
  clientId: string
  clientSecret: string
  scope: string
}

const authFrom = (values: DataExportFormValues): DataExportAuthRequest => {
  const password = values.password.trim()
  const clientSecret = values.clientSecret.trim()
  const scope = values.scope.trim()
  switch (values.authKind) {
    case 'none':
      return { kind: 'none' }
    case 'basic':
      return { kind: 'basic', username: values.username.trim(), ...(password && { password }) }
    case 'oauth2_client_credentials':
      return {
        kind: 'oauth2_client_credentials',
        tokenUrl: values.tokenUrl.trim(),
        clientId: values.clientId.trim(),
        ...(clientSecret && { clientSecret }),
        ...(scope && { scope }),
      }
  }
}

export function toDataExportRequest(values: DataExportFormValues): DataExportTargetRequest {
  return { enabled: values.enabled, baseUrl: values.baseUrl.trim(), auth: authFrom(values) }
}

export function formValuesFrom(target?: DataExportTargetResponse): DataExportFormValues {
  return {
    enabled: target?.enabled ?? true,
    baseUrl: target?.baseUrl ?? '',
    authKind: target?.auth.kind ?? 'none',
    username: target?.auth.username ?? '',
    password: '',
    tokenUrl: target?.auth.tokenUrl ?? '',
    clientId: target?.auth.clientId ?? '',
    clientSecret: '',
    scope: target?.auth.scope ?? '',
  }
}
