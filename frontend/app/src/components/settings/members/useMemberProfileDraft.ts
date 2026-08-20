import { useCallback, useState } from 'react'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import type { UserResponse } from '@/api/backendApi'

export interface MemberProfileDraft {
  userId: string
  status: UserStatus
  drivingLicenses: DrivingLicense[]
  phoneNumber: string
  employeeId: string
  wateringPlanSelectable: boolean
  // The endpoint replaces the whole profile; an omitted avatarUrl clears the stored value.
  // This field is carried passively to avoid data loss on save.
  avatarUrl: string | null
}

const canonical = (draft: MemberProfileDraft): string =>
  [
    draft.status,
    [...draft.drivingLicenses].sort().join(','),
    draft.phoneNumber.trim(),
    draft.employeeId.trim(),
    String(draft.wateringPlanSelectable),
  ].join('|')

const draftOf = (user: UserResponse): MemberProfileDraft => ({
  userId: user.id,
  status: user.status,
  drivingLicenses: [...user.drivingLicenses],
  phoneNumber: user.phoneNumber ?? '',
  employeeId: user.employeeId ?? '',
  wateringPlanSelectable: user.wateringPlanSelectable,
  avatarUrl: user.avatarUrl ?? null,
})

export const useMemberProfileDraft = () => {
  const [draft, setDraft] = useState<MemberProfileDraft | null>(null)
  const [baseline, setBaseline] = useState<string | null>(null)

  const edit = useCallback((user: UserResponse) => {
    const next = draftOf(user)
    setDraft(next)
    setBaseline(canonical(next))
  }, [])

  const discard = useCallback(() => {
    setDraft(null)
    setBaseline(null)
  }, [])

  const setStatus = useCallback((status: UserStatus) => {
    setDraft((current) => (current ? { ...current, status } : current))
  }, [])

  const setDrivingLicenses = useCallback((drivingLicenses: DrivingLicense[]) => {
    setDraft((current) => (current ? { ...current, drivingLicenses } : current))
  }, [])

  const setPhoneNumber = useCallback((phoneNumber: string) => {
    setDraft((current) => (current ? { ...current, phoneNumber } : current))
  }, [])

  const setEmployeeId = useCallback((employeeId: string) => {
    setDraft((current) => (current ? { ...current, employeeId } : current))
  }, [])

  const setWateringPlanSelectable = useCallback((wateringPlanSelectable: boolean) => {
    setDraft((current) => (current ? { ...current, wateringPlanSelectable } : current))
  }, [])

  const dirty = draft !== null && baseline !== null && canonical(draft) !== baseline

  return {
    draft,
    dirty,
    edit,
    discard,
    setStatus,
    setDrivingLicenses,
    setPhoneNumber,
    setEmployeeId,
    setWateringPlanSelectable,
  }
}
