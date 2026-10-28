import { useEffect, useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { wasteValidateFullThreshold, type ValidationIssue } from '@green-ecolution/domain-wasm'
import {
  SettingOriginDto,
  type WasteSettingsResponse,
  type WasteSettingsUpdateRequest,
} from '@green-ecolution/backend-client'
import type { ModuleSettingsDraft } from '@/modules/types'
import { wasteSettingsQueries } from '../../api/settings'

// The manifest imports this file eagerly, next to the registry the i18n setup
// imports. So nothing here may pull in `@/lib/i18n` (the section translates the
// issue itself) or the API clients (loaded on demand, see `api/settings`).

export interface FullThresholdDraft {
  /** false = inherit, true = this organization sets its own value */
  own: boolean
  /** raw input text, so a half-typed number does not get coerced */
  text: string
}

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary.
const thresholdValidator = wasteValidateFullThreshold as (percent: number) => ValidationIssue | null

const draftOf = (response: WasteSettingsResponse): FullThresholdDraft => ({
  own: response.fullThreshold.origin === SettingOriginDto.Own,
  text: String(response.fullThreshold.value),
})

/** Everything the draft is built from; a refetch that changes none of it must
 *  leave an edit in progress alone. */
const fingerprint = (orgId: string, response: WasteSettingsResponse | undefined): string | null =>
  response
    ? JSON.stringify([
        orgId,
        response.fullThreshold.value,
        response.fullThreshold.origin,
        response.fullThreshold.ownValue ?? null,
      ])
    : null

export interface WasteSettingsDraft extends ModuleSettingsDraft {
  server: WasteSettingsResponse | null
  draft: FullThresholdDraft | null
  issue: ValidationIssue | null
  setOwn: (own: boolean) => void
  setText: (text: string) => void
}

export const useWasteSettingsDraft = (
  organizationId: string,
  { enabled }: { enabled: boolean },
): WasteSettingsDraft => {
  const queryClient = useQueryClient()
  const query = wasteSettingsQueries.byOrganization(organizationId)
  const { data: response, error } = useQuery({ ...query, enabled })
  const [draft, setDraft] = useState<FullThresholdDraft | null>(null)
  const [loadedKey, setLoadedKey] = useState<string | null>(null)
  const responseKey = fingerprint(organizationId, response)

  const update = useMutation({
    mutationFn: async (body: WasteSettingsUpdateRequest) => {
      const { wasteBinApi } = await import('../../api/client')
      return wasteBinApi.updateWasteSettings({
        orgId: organizationId,
        wasteSettingsUpdateRequest: body,
      })
    },
    // Also after a failure: another tab may have moved the value meanwhile.
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: query.queryKey })
    },
  })

  useEffect(() => {
    // Keyed on the content, not on the object: `lastChange.changedAt` is parsed
    // into a Date, so every refetch hands back a new object and an identity
    // check would reset the draft while a save is still in flight.
    if (!response || responseKey === loadedKey) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- loads the freshly fetched settings into the draft
    setLoadedKey(responseKey)
    // eslint-disable-next-line react-x/set-state-in-effect -- same load
    setDraft(draftOf(response))
  }, [response, responseKey, loadedKey])

  const server = response ?? null
  const issue = draft?.own ? thresholdValidator(Number(draft.text)) : null
  const valid = issue === null

  const toRequest = (): WasteSettingsUpdateRequest | null => {
    // A non-numeric own value would serialize to `null` and read as "give up
    // the own value" on the server, so no body is built while invalid.
    if (!draft || !server || !valid) return null
    const wasOwn = server.fullThreshold.origin === SettingOriginDto.Own
    if (!draft.own) return wasOwn ? { fullThreshold: null } : null
    const parsed = Number(draft.text)
    return !wasOwn || parsed !== server.fullThreshold.value ? { fullThreshold: parsed } : null
  }

  const request = toRequest()

  return {
    loading: !enabled || !response || draft === null,
    loadError: error,
    dirty: request !== null,
    valid,
    reset: () => {
      if (response) setDraft(draftOf(response))
    },
    save: async () => {
      if (request) await update.mutateAsync(request)
    },
    server,
    draft,
    issue,
    setOwn: (own) =>
      setDraft((current) => {
        if (!current || !server) return current
        if (!own) return { ...current, own: false }
        // Start from the effective (inherited) value, not leftover text of a discarded edit.
        return { own: true, text: String(server.fullThreshold.value) }
      }),
    setText: (text) => setDraft((current) => (current ? { ...current, text } : current)),
  }
}
