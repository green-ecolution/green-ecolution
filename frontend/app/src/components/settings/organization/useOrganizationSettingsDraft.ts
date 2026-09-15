import { useCallback, useState } from 'react'
import {
  translateIssue,
  validateJustWateredTtlHours,
  validateWaterDemand,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import {
  SettingOriginDto,
  type OrganizationSettingsResponse,
  type OrganizationSettingsUpdateRequest,
} from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'

export interface SettingsFieldDraft {
  /** false = inherit, true = this organization sets its own value */
  own: boolean
  /** raw input text, so a half-typed number does not get coerced */
  text: string
}

export interface SettingsDraft {
  waterDemand: SettingsFieldDraft
  justWateredTtlHours: SettingsFieldDraft
  descendantsMayOverride: boolean
}

type NumericField = 'waterDemand' | 'justWateredTtlHours'

// wasm-bindgen emits `(input: number) => any`; tighten the return type at the boundary,
// same pattern as the draft resolvers in @green-ecolution/domain-wasm.
const waterDemandValidator = validateWaterDemand as (liters: number) => ValidationIssue | null
const justWateredTtlValidator = validateJustWateredTtlHours as (
  hours: number,
) => ValidationIssue | null

const hoursOf = (seconds: number): string => String(seconds / 3600)
const secondsOf = (hours: string): number => Math.round(Number(hours) * 3600)

const draftOf = (response: OrganizationSettingsResponse): SettingsDraft => ({
  waterDemand: {
    own: response.waterDemand.origin === SettingOriginDto.Own,
    text: String(response.waterDemand.value),
  },
  justWateredTtlHours: {
    own: response.justWateredTtlSecs.origin === SettingOriginDto.Own,
    text: hoursOf(response.justWateredTtlSecs.value),
  },
  descendantsMayOverride: response.descendantsMayOverride,
})

export const useOrganizationSettingsDraft = () => {
  const translate = useIssueTranslator()
  const [server, setServer] = useState<OrganizationSettingsResponse | null>(null)
  const [draft, setDraft] = useState<SettingsDraft | null>(null)

  // Stable so a page may load it into an effect that follows the loaded settings.
  const load = useCallback((response: OrganizationSettingsResponse) => {
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
          : hoursOf(server.justWateredTtlSecs.value)
      return { ...current, [field]: { own: true, text } }
    })
  }

  const setText = (field: NumericField, text: string) => {
    setDraft((current) =>
      current ? { ...current, [field]: { ...current[field], text } } : current,
    )
  }

  const setDescendantsMayOverride = (value: boolean) => {
    setDraft((current) => (current ? { ...current, descendantsMayOverride: value } : current))
  }

  const waterDemandIssue = draft?.waterDemand.own
    ? waterDemandValidator(Number(draft.waterDemand.text))
    : null
  const justWateredTtlIssue = draft?.justWateredTtlHours.own
    ? justWateredTtlValidator(Number(draft.justWateredTtlHours.text))
    : null

  const errors = {
    waterDemand: waterDemandIssue ? translateIssue(waterDemandIssue, translate) : null,
    justWateredTtlHours: justWateredTtlIssue
      ? translateIssue(justWateredTtlIssue, translate)
      : null,
  }

  const valid = errors.waterDemand === null && errors.justWateredTtlHours === null

  const toRequest = (): OrganizationSettingsUpdateRequest | null => {
    if (!draft || !server) return null
    const body: OrganizationSettingsUpdateRequest = {}

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

    if (draft.descendantsMayOverride !== server.descendantsMayOverride) {
      body.descendantsMayOverride = draft.descendantsMayOverride
    }

    return Object.keys(body).length > 0 ? body : null
  }

  const dirty = toRequest() !== null

  return {
    draft,
    dirty,
    errors,
    valid,
    load,
    setOwn,
    setText,
    setDescendantsMayOverride,
    toRequest,
  }
}
