import { useCallback, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  irrigationValidateJustWateredTtlHours,
  irrigationValidateWaterDemand,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import {
  type IrrigationSettingsResponse,
  type IrrigationSettingsUpdateRequest,
} from '@green-ecolution/backend-client'
import type { ModuleSettingsDraft } from '@/modules/types'
import { irrigationSettingsQueries } from '../../api/settings'

// The manifest imports this file eagerly, next to the registry the i18n setup
// imports. So nothing here may pull in `@/lib/i18n` (the section translates the
// issues itself) or the API clients (loaded on demand, see `api/settings`).

export interface IrrigationDraft {
  /** raw input text, so a half-typed number does not get coerced */
  waterDemand: string
  justWateredTtlHours: string
}

export type NumericField = keyof IrrigationDraft

export type IrrigationIssues = Record<NumericField, ValidationIssue | null>

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary,
// same pattern as the draft resolvers in @green-ecolution/domain-wasm.
const waterDemandValidator = irrigationValidateWaterDemand as (
  liters: number,
) => ValidationIssue | null
const justWateredTtlValidator = irrigationValidateJustWateredTtlHours as (
  hours: number,
) => ValidationIssue | null

/** The API speaks seconds, the field asks for hours. */
export const hoursOf = (seconds: number): number => seconds / 3600
const secondsOf = (hours: string): number => Math.round(Number(hours) * 3600)

const draftOf = (response: IrrigationSettingsResponse): IrrigationDraft => ({
  waterDemand: String(response.waterDemand.value),
  justWateredTtlHours: String(hoursOf(response.justWateredTtlSecs.value)),
})

export const useIrrigationDraftState = () => {
  const [server, setServer] = useState<IrrigationSettingsResponse | null>(null)
  const [draft, setDraft] = useState<IrrigationDraft | null>(null)

  const load = useCallback((response: IrrigationSettingsResponse) => {
    setServer(response)
    setDraft(draftOf(response))
  }, [])

  const setText = (field: NumericField, text: string) => {
    setDraft((current) => (current ? { ...current, [field]: text } : current))
  }

  const issues: IrrigationIssues = {
    waterDemand: draft ? waterDemandValidator(Number(draft.waterDemand)) : null,
    justWateredTtlHours: draft ? justWateredTtlValidator(Number(draft.justWateredTtlHours)) : null,
  }
  const valid = issues.waterDemand === null && issues.justWateredTtlHours === null

  const toRequest = (): IrrigationSettingsUpdateRequest | null => {
    // An empty field is `Number('') === 0`, which the validators refuse, so no
    // half-typed value reaches the server.
    if (!draft || !server || !valid) return null
    const body: IrrigationSettingsUpdateRequest = {}
    const liters = Number(draft.waterDemand)
    if (liters !== server.waterDemand.value) body.waterDemand = liters
    const seconds = secondsOf(draft.justWateredTtlHours)
    if (seconds !== server.justWateredTtlSecs.value) body.justWateredTtlSecs = seconds
    return Object.keys(body).length > 0 ? body : null
  }

  const dirty = toRequest() !== null

  return { server, draft, dirty, issues, valid, load, setText, toRequest }
}

/** Everything the draft is built from; a refetch that changes none of it must
 *  leave an edit in progress alone. */
const fingerprint = (
  orgId: string,
  response: IrrigationSettingsResponse | undefined,
): string | null => (response ? JSON.stringify([orgId, response]) : null)

export interface IrrigationSettingsDraft extends ModuleSettingsDraft {
  server: IrrigationSettingsResponse | null
  draft: IrrigationDraft | null
  issues: IrrigationIssues
  setText: (field: NumericField, text: string) => void
}

export const useIrrigationSettingsDraft = (
  organizationId: string,
  { enabled }: { enabled: boolean },
): IrrigationSettingsDraft => {
  const queryClient = useQueryClient()
  const query = irrigationSettingsQueries.byOrganization(organizationId)
  const { data: response, error } = useQuery({ ...query, enabled })
  const state = useIrrigationDraftState()
  const { load } = state
  const [loadedKey, setLoadedKey] = useState<string | null>(null)
  const responseKey = fingerprint(organizationId, response)

  const update = useMutation({
    mutationFn: async (body: IrrigationSettingsUpdateRequest) => {
      const { irrigationApi } = await import('../../api/client')
      return irrigationApi.updateIrrigationSettings({
        orgId: organizationId,
        irrigationSettingsUpdateRequest: body,
      })
    },
    // Also after a failure: another tab may have moved the values meanwhile.
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: query.queryKey })
    },
  })

  // Keyed on the content: a refetch hands back a new object, and an identity
  // check would reset the draft while a save is still in flight.
  if (response && responseKey !== loadedKey) {
    setLoadedKey(responseKey)
    load(response)
  }

  const save = async () => {
    const body = state.toRequest()
    if (!body) return
    await update.mutateAsync(body)
  }

  return {
    loading: !enabled || !response || state.draft === null,
    loadError: error,
    dirty: state.dirty,
    valid: state.valid,
    reset: () => {
      if (response) load(response)
    },
    save,
    server: state.server,
    draft: state.draft,
    issues: state.issues,
    setText: state.setText,
  }
}
