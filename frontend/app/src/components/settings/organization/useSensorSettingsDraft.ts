import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import {
  validateDefectStreak,
  validateSensorOfflineAfterHours,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import type {
  OrganizationSettingsResponse,
  OrganizationSettingsUpdateRequest,
} from '@/api/backendApi'
import { settingsQueries } from '@/api/queries'
import { useSettingsMutations } from '@/hooks/useSettingsMutations'
import type { ModuleSettingsDraft } from '@/modules/types'

export type SensorField = 'offlineAfterHours' | 'defectStreak'
type Draft = Record<SensorField, string>

export interface SensorSettingsDraft extends ModuleSettingsDraft {
  server: OrganizationSettingsResponse | null
  draft: Draft | null
  issues: Record<SensorField, ValidationIssue | null>
  setText: (field: SensorField, text: string) => void
}

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary.
const offlineAfterValidator = validateSensorOfflineAfterHours as (
  hours: number,
) => ValidationIssue | null
const defectStreakValidator = validateDefectStreak as (count: number) => ValidationIssue | null

/** The API speaks seconds, the field asks for hours. */
export const hoursOf = (seconds: number): number => seconds / 3600
const secondsOf = (hours: string): number => Math.round(Number(hours) * 3600)

const draftOf = (response: OrganizationSettingsResponse): Draft => ({
  offlineAfterHours: String(hoursOf(response.sensorOfflineAfterSecs.value)),
  defectStreak: String(response.defectStreak.value),
})

const fingerprint = (orgId: string, response: OrganizationSettingsResponse | undefined) =>
  response
    ? JSON.stringify([orgId, response.sensorOfflineAfterSecs, response.defectStreak])
    : null

export const useSensorSettingsDraft = (
  organizationId: string,
  { enabled }: { enabled: boolean },
): SensorSettingsDraft => {
  const { data: response, error } = useQuery({
    ...settingsQueries.byOrganization(organizationId),
    enabled,
  })
  const { updateSettings } = useSettingsMutations()
  const [draft, setDraft] = useState<Draft | null>(null)
  const [loadedKey, setLoadedKey] = useState<string | null>(null)
  const responseKey = fingerprint(organizationId, response)

  if (response && responseKey !== loadedKey) {
    setLoadedKey(responseKey)
    setDraft(draftOf(response))
  }

  const server = response ?? null
  const issues = {
    offlineAfterHours: draft ? offlineAfterValidator(Number(draft.offlineAfterHours)) : null,
    defectStreak: draft ? defectStreakValidator(Number(draft.defectStreak)) : null,
  }
  const valid = issues.offlineAfterHours === null && issues.defectStreak === null

  const toRequest = (): OrganizationSettingsUpdateRequest | null => {
    if (!draft || !server || !valid) return null
    const body: OrganizationSettingsUpdateRequest = {}
    const seconds = secondsOf(draft.offlineAfterHours)
    if (seconds !== server.sensorOfflineAfterSecs.value) body.sensorOfflineAfterSecs = seconds
    const streak = Number(draft.defectStreak)
    if (streak !== server.defectStreak.value) body.defectStreak = streak
    return Object.keys(body).length > 0 ? body : null
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
      if (request) await updateSettings.mutateAsync({ orgId: organizationId, body: request })
    },
    server,
    draft,
    issues,
    setText: (field, text) =>
      setDraft((current) => (current ? { ...current, [field]: text } : current)),
  }
}
