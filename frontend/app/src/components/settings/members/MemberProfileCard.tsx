import { FormField, Label, MultiSelect, SelectField, Switch } from '@green-ecolution/ui'
import { translateIssue } from '@green-ecolution/domain-wasm'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import { useIssueTranslator } from '@/lib/i18n/validation'
import {
  useDrivingLicenseOptions,
  useDrivingLicenseDetails,
} from '@/hooks/details/useDetailsForDrivingLicense'
import { useUserStatusDetails, useUserStatusOptions } from '@/hooks/details/useDetailsForUserStatus'
import { CARD, CARD_TITLE } from './cardChrome'
import { phoneNumberIssue } from './memberList'
import type { MemberProfileDraft } from './useMemberProfileDraft'

interface MemberProfileCardProps {
  draft: MemberProfileDraft
  editable: boolean
  onStatusChange: (status: UserStatus) => void
  onDrivingLicensesChange: (licenses: DrivingLicense[]) => void
  onPhoneNumberChange: (value: string) => void
  onEmployeeIdChange: (value: string) => void
  onWateringPlanSelectableChange: (value: boolean) => void
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
  onWateringPlanSelectableChange,
}: MemberProfileCardProps) => {
  const translate = useIssueTranslator()
  const getDrivingLicenseDetails = useDrivingLicenseDetails()
  const getUserStatusDetails = useUserStatusDetails()
  const drivingLicenseOptions = useDrivingLicenseOptions()
  const userStatusOptions = useUserStatusOptions()
  const licenseLabels = draft.drivingLicenses
    .map((license) => getDrivingLicenseDetails(license).label)
    .join(', ')
  const phoneIssue = phoneNumberIssue(draft.phoneNumber)

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
              options={userStatusOptions
                .filter((option) => option.value !== 'unknown')
                .map((option) => ({ value: option.value, label: option.label }))}
            />
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="member-driving-licenses">Führerscheinklassen</Label>
              <MultiSelect
                id="member-driving-licenses"
                value={draft.drivingLicenses}
                onChange={(value) => onDrivingLicensesChange(value as DrivingLicense[])}
                options={drivingLicenseOptions.map((option) => ({
                  value: option.value,
                  label: option.label,
                }))}
              />
            </div>
            <FormField
              label="Telefonnummer"
              value={draft.phoneNumber}
              onChange={(event) => onPhoneNumberChange(event.target.value)}
              error={phoneIssue ? translateIssue(phoneIssue, translate) : undefined}
              placeholder="z. B. +49 461 123456"
              inputMode="tel"
            />
            <FormField
              label="Personalnummer"
              value={draft.employeeId}
              onChange={(event) => onEmployeeIdChange(event.target.value)}
              placeholder="z. B. EMP-042"
            />
            <div className="flex flex-col gap-y-2 @min-[36rem]:col-span-2">
              <div className="flex items-center gap-x-3">
                <Switch
                  id="member-watering-plan-selectable"
                  checked={draft.wateringPlanSelectable}
                  onCheckedChange={onWateringPlanSelectableChange}
                />
                <Label htmlFor="member-watering-plan-selectable">
                  Für Einsatzplanung auswählbar
                </Label>
              </div>
              <p className="text-sm text-dark-600">
                Nur gekennzeichnete Personen erscheinen in der Auswahl der verknüpften
                Mitarbeitenden eines Einsatzplans.
              </p>
            </div>
          </>
        ) : (
          <>
            <StaticField label="Verfügbarkeit" value={getUserStatusDetails(draft.status).label} />
            <StaticField label="Führerscheinklassen" value={licenseLabels} />
            <StaticField label="Telefonnummer" value={draft.phoneNumber} />
            <StaticField label="Personalnummer" value={draft.employeeId} />
            <StaticField
              label="Für Einsatzplanung auswählbar"
              value={draft.wateringPlanSelectable ? 'Ja' : 'Nein'}
            />
          </>
        )}
      </div>
    </section>
  )
}

export default MemberProfileCard
