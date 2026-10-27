import { useCallback, useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import {
  irrigationValidateJustWateredTtlHours,
  irrigationValidateWaterDemand,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import {
  SettingOriginDto,
  type IrrigationSettingsResponse,
  type IrrigationSettingsUpdateRequest,
} from '@green-ecolution/backend-client'
import type { ModuleSettingsDraft } from '@/modules/types'
import { irrigationSettingsQueries } from '../../api/settings'

// The manifest imports this file eagerly, next to the registry the i18n setup
// imports. So nothing here may pull in `@/lib/i18n` (the section translates the
// issues itself) or the API clients (loaded on demand, see `api/settings`).

export interface SettingsFieldDraft {
  /** false = inherit, true = this organization sets its own value */
  own: boolean
  /** raw input text, so a half-typed number does not get coerced */
  text: string
}

export interface IrrigationDraft {
  waterDemand: SettingsFieldDraft
  justWateredTtlHours: SettingsFieldDraft
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
  waterDemand: {
    own: response.waterDemand.origin === SettingOriginDto.Own,
    text: String(response.waterDemand.value),
  },
  justWateredTtlHours: {
    own: response.justWateredTtlSecs.origin === SettingOriginDto.Own,
    text: String(hoursOf(response.justWateredTtlSecs.value)),
  },
})

export const useIrrigationDraftState = () => {
  const [server, setServer] = useState<IrrigationSettingsResponse | null>(null)
  const [draft, setDraft] = useState<IrrigationDraft | null>(null)

  // Stable so it may be loaded from an effect that follows the fetched settings.
  const load = useCallback((response: IrrigationSettingsResponse) => {
    setServer(response)
    setDraft(draftOf(response))
  }, [])

  const setOwn = (field: NumericField, own: boolean) => {
    setDraft((current) => {
      if (!current || !server) return current
      if (!own) return { ...current, [field]: { ...current[field], own: false } }
      // The starting point is the currently effective (inherited) value, not
      // whatever text was left over from an earlier, possibly discarded, own edit.
      const text =
        field === 'waterDemand'
          ? String(server.waterDemand.value)
          : String(hoursOf(server.justWateredTtlSecs.value))
      return { ...current, [field]: { own: true, text } }
    })
  }

  const setText = (field: NumericField, text: string) => {
    setDraft((current) =>
      current ? { ...current, [field]: { ...current[field], text } } : current,
    )
  }

  const issues: IrrigationIssues = {
    waterDemand: draft?.waterDemand.own
      ? waterDemandValidator(Number(draft.waterDemand.text))
      : null,
    justWateredTtlHours: draft?.justWateredTtlHours.own
      ? justWateredTtlValidator(Number(draft.justWateredTtlHours.text))
      : null,
  }

  const valid = issues.waterDemand === null && issues.justWateredTtlHours === null

  const toRequest = (): IrrigationSettingsUpdateRequest | null => {
    // A non-numeric own value would otherwise serialize to `null` over JSON
    // and silently read as "give up the own value" on the server. Refuse to
    // build a body at all while invalid; the caller already sees the error
    // via `issues`, so nothing is lost by withholding the request.
    if (!draft || !server || !valid) return null
    const body: IrrigationSettingsUpdateRequest = {}

    const waterDemandWasOwn = server.waterDemand.origin === SettingOriginDto.Own
    if (!draft.waterDemand.own) {
      if (waterDemandWasOwn) body.waterDemand = null
    } else {
      const parsed = Number(draft.waterDemand.text)
      if (!waterDemandWasOwn || parsed !== server.waterDemand.value) body.waterDemand = parsed
    }

    const ttlWasOwn = server.justWateredTtlSecs.origin === SettingOriginDto.Own
    if (!draft.justWateredTtlHours.own) {
      if (ttlWasOwn) body.justWateredTtlSecs = null
    } else {
      const parsed = secondsOf(draft.justWateredTtlHours.text)
      if (!ttlWasOwn || parsed !== server.justWateredTtlSecs.value) {
        body.justWateredTtlSecs = parsed
      }
    }

    return Object.keys(body).length > 0 ? body : null
  }

  const dirty = toRequest() !== null

  return { server, draft, dirty, issues, valid, load, setOwn, setText, toRequest }
}

/** Everything the draft is built from; a refetch that changes none of it must
 *  leave an edit in progress alone. */
const fingerprint = (
  orgId: string,
  response: IrrigationSettingsResponse | undefined,
): string | null =>
  response
    ? JSON.stringify([
        orgId,
        response.waterDemand.value,
        response.waterDemand.origin,
        response.waterDemand.ownValue ?? null,
        response.justWateredTtlSecs.value,
        response.justWateredTtlSecs.origin,
        response.justWateredTtlSecs.ownValue ?? null,
      ])
    : null

export interface IrrigationSettingsDraft extends ModuleSettingsDraft {
  server: IrrigationSettingsResponse | null
  draft: IrrigationDraft | null
  issues: IrrigationIssues
  setOwn: (field: NumericField, own: boolean) => void
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

  useEffect(() => {
    // Keyed on the content, not on the object: `lastChange.changedAt` is parsed
    // into a Date, which structural sharing cannot keep referentially stable, so
    // every refetch hands back a new object. An identity check would reset the
    // draft while a save is still in flight and lose the typed value if that
    // save then fails.
    if (!response || responseKey === loadedKey) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- loads the freshly fetched settings into the draft
    setLoadedKey(responseKey)
    load(response)
  }, [response, responseKey, loadedKey, load])

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
    setOwn: state.setOwn,
    setText: state.setText,
  }
}
