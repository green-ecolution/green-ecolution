import { useCallback, useState } from 'react'
import type { OrganizationDetailResponse } from '@/api/backendApi'

export interface OrganizationDraft {
  name: string
  street: string
  postalCode: string
  city: string
  contactPersonId: string | null
}

export interface AddressFieldErrors {
  street?: string
  postalCode?: string
  city?: string
}

const MISSING: Required<AddressFieldErrors> = {
  street: 'Straße fehlt',
  postalCode: 'PLZ fehlt',
  city: 'Ort fehlt',
}

const draftOf = (org: OrganizationDetailResponse): OrganizationDraft => ({
  name: org.name,
  street: org.address?.street ?? '',
  postalCode: org.address?.postalCode ?? '',
  city: org.address?.city ?? '',
  // The raw id, never the resolved person: an unresolvable id still has to be
  // round-tripped, otherwise saving anything else would clear the reference.
  contactPersonId: org.contactPersonId ?? null,
})

const sameField = (a: string, b: string): boolean => a.trim() === b.trim()

export const useOrganizationDraft = () => {
  const [draft, setDraft] = useState<OrganizationDraft | null>(null)
  const [baseline, setBaseline] = useState<OrganizationDraft | null>(null)

  // Stable so a page may load it into an effect that follows the selected organization.
  const edit = useCallback((org: OrganizationDetailResponse) => {
    setDraft(draftOf(org))
    setBaseline(draftOf(org))
  }, [])

  const discard = useCallback(() => {
    setDraft(null)
    setBaseline(null)
  }, [])

  const patch = (change: Partial<OrganizationDraft>) => {
    setDraft((current) => (current ? { ...current, ...change } : current))
  }

  const filled = draft
    ? [draft.street, draft.postalCode, draft.city].filter((value) => value.trim().length > 0).length
    : 0

  // An organization may have no address at all; a partial one is what we reject.
  const addressComplete = filled === 0 || filled === 3

  const addressErrors: AddressFieldErrors = {}
  if (draft && !addressComplete) {
    if (draft.street.trim().length === 0) addressErrors.street = MISSING.street
    if (draft.postalCode.trim().length === 0) addressErrors.postalCode = MISSING.postalCode
    if (draft.city.trim().length === 0) addressErrors.city = MISSING.city
  }

  const dirty =
    draft !== null &&
    baseline !== null &&
    !(
      sameField(draft.name, baseline.name) &&
      sameField(draft.street, baseline.street) &&
      sameField(draft.postalCode, baseline.postalCode) &&
      sameField(draft.city, baseline.city) &&
      draft.contactPersonId === baseline.contactPersonId
    )

  return {
    draft,
    dirty,
    addressErrors,
    addressComplete,
    edit,
    setName: (value: string) => patch({ name: value }),
    setStreet: (value: string) => patch({ street: value }),
    setPostalCode: (value: string) => patch({ postalCode: value }),
    setCity: (value: string) => patch({ city: value }),
    setContactPersonId: (value: string | null) => patch({ contactPersonId: value }),
    discard,
  }
}
