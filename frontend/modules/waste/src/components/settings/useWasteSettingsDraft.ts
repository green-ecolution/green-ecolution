import { useState } from 'react'
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { wasteValidateFullThreshold, type ValidationIssue } from '@green-ecolution/domain-wasm'
import {
  type WasteSettingsResponse,
  type WasteSettingsUpdateRequest,
} from '@green-ecolution/backend-client'
import type { ModuleSettingsDraft } from '@/modules/types'
import { wasteSettingsQueries } from '../../api/settings'

// The manifest imports this file eagerly, next to the registry the i18n setup
// imports. So nothing here may pull in `@/lib/i18n` (the section translates the
// issue itself) or the API clients (loaded on demand, see `api/settings`).

export interface FullThresholdDraft {
  /** raw input text, so a half-typed number does not get coerced */
  text: string
}

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary.
const thresholdValidator = wasteValidateFullThreshold as (percent: number) => ValidationIssue | null

const draftOf = (response: WasteSettingsResponse): FullThresholdDraft => ({
  text: String(response.fullThreshold.value),
})

const fingerprint = (orgId: string, response: WasteSettingsResponse | undefined): string | null =>
  response ? JSON.stringify([orgId, response]) : null

export interface WasteSettingsDraft extends ModuleSettingsDraft {
  server: WasteSettingsResponse | null
  draft: FullThresholdDraft | null
  issue: ValidationIssue | null
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

  // Keyed on the content: `lastChange.changedAt` is parsed into a Date, so a
  // refetch hands back a new object and an identity check would reset the draft
  // while a save is still in flight.
  if (response && responseKey !== loadedKey) {
    setLoadedKey(responseKey)
    setDraft(draftOf(response))
  }

  const server = response ?? null
  const issue = draft ? thresholdValidator(Number(draft.text)) : null
  const valid = issue === null

  const toRequest = (): WasteSettingsUpdateRequest | null => {
    if (!draft || !server || !valid) return null
    const parsed = Number(draft.text)
    return parsed !== server.fullThreshold.value ? { fullThreshold: parsed } : null
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
    setText: (text) => setDraft((current) => (current ? { ...current, text } : current)),
  }
}
