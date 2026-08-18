import { FormField, Label, MultiSelect, SelectField } from '@green-ecolution/ui'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import {
  DrivingLicenseOptions,
  getDrivingLicenseDetails,
} from '@/hooks/details/useDetailsForDrivingLicense'
import { getUserStatusDetails, UserStatusOptions } from '@/hooks/details/useDetailsForUserStatus'
import { CARD, CARD_TITLE } from './cardChrome'
import { phoneNumberError } from './memberList'
import type { MemberProfileDraft } from './useMemberProfileDraft'

interface MemberProfileCardProps {
  draft: MemberProfileDraft
  editable: boolean
  onStatusChange: (status: UserStatus) => void
  onDrivingLicensesChange: (licenses: DrivingLicense[]) => void
  onPhoneNumberChange: (value: string) => void
  onEmployeeIdChange: (value: string) => void
}

const StaticField = ({ label, value }: { label: string; value: string }) => (
  <div className="flex flex-col gap-y-1">
    <p className="text-sm font-medium text-dark">{label}</p>
    <p className="text-sm text-dark-600">{value.trim().length > 0 ? value : 'Nicht hinterlegt'}</p>
  </div>
)

const MemberProfileCard = ({
  draft,
  editable,
  onStatusChange,
  onDrivingLicensesChange,
  onPhoneNumberChange,
  onEmployeeIdChange,
}: MemberProfileCardProps) => {
  const licenseLabels = draft.drivingLicenses
    .map((license) => getDrivingLicenseDetails(license).label)
    .join(', ')

  return (
    <section className={`${CARD} @min-[48rem]:col-span-2`}>
      <h3 className={CARD_TITLE}>Profil</h3>
      <p className="mt-0.5 text-sm text-dark-600">
        Diese Angaben zählen für die Einteilung in Einsatzpläne.
      </p>

      <div className="mt-4 grid gap-4 @min-[36rem]:grid-cols-2">
        {editable ? (
          <>
            <SelectField
              label="Verfügbarkeit"
              value={draft.status}
              onValueChange={(value) => onStatusChange(value as UserStatus)}
              options={UserStatusOptions.filter((option) => option.value !== 'unknown').map(
                (option) => ({ value: option.value, label: option.label }),
              )}
            />
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="member-driving-licenses">Führerscheinklassen</Label>
              <MultiSelect
                id="member-driving-licenses"
                value={draft.drivingLicenses}
                onChange={(value) => onDrivingLicensesChange(value as DrivingLicense[])}
                options={DrivingLicenseOptions.map((option) => ({
                  value: option.value,
                  label: option.label,
                }))}
              />
            </div>
            <FormField
              label="Telefonnummer"
              value={draft.phoneNumber}
              onChange={(event) => onPhoneNumberChange(event.target.value)}
              error={phoneNumberError(draft.phoneNumber) ?? undefined}
              placeholder="z. B. +49 461 123456"
              inputMode="tel"
            />
            <FormField
              label="Personalnummer"
              value={draft.employeeId}
              onChange={(event) => onEmployeeIdChange(event.target.value)}
              placeholder="z. B. EMP-042"
            />
          </>
        ) : (
          <>
            <StaticField label="Verfügbarkeit" value={getUserStatusDetails(draft.status).label} />
            <StaticField label="Führerscheinklassen" value={licenseLabels} />
            <StaticField label="Telefonnummer" value={draft.phoneNumber} />
            <StaticField label="Personalnummer" value={draft.employeeId} />
          </>
        )}
      </div>
    </section>
  )
}

export default MemberProfileCard
