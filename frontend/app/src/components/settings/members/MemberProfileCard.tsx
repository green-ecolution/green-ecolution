import { useTranslation } from 'react-i18next'
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

const StaticField = ({
  label,
  value,
  fallback,
}: {
  label: string
  value: string
  fallback: string
}) => (
  <div className="flex flex-col gap-y-1">
    <p className="text-sm font-medium text-dark">{label}</p>
    <p className="text-sm text-dark-600">{value.trim().length > 0 ? value : fallback}</p>
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
  const { t } = useTranslation(['settings', 'common'])
  const translate = useIssueTranslator()
  const getDrivingLicenseDetails = useDrivingLicenseDetails()
  const getUserStatusDetails = useUserStatusDetails()
  const drivingLicenseOptions = useDrivingLicenseOptions()
  const userStatusOptions = useUserStatusOptions()
  const licenseLabels = draft.drivingLicenses
    .map((license) => getDrivingLicenseDetails(license).label)
    .join(', ')
  const phoneIssue = phoneNumberIssue(draft.phoneNumber)
  const notProvided = t('notProvided')

  return (
    <section className={`${CARD} @min-[48rem]:col-span-2`}>
      <h3 className={CARD_TITLE}>{t('members.profileTitle')}</h3>
      <p className="mt-0.5 text-sm text-dark-600">{t('members.profileHint')}</p>

      <div className="mt-4 grid gap-4 @min-[36rem]:grid-cols-2">
        {editable ? (
          <>
            <SelectField
              label={t('members.availabilityLabel')}
              value={draft.status}
              onValueChange={(value) => onStatusChange(value as UserStatus)}
              options={userStatusOptions
                .filter((option) => option.value !== 'unknown')
                .map((option) => ({ value: option.value, label: option.label }))}
            />
            <div className="flex flex-col gap-y-2">
              <Label htmlFor="member-driving-licenses">{t('members.drivingLicensesLabel')}</Label>
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
              label={t('members.phoneNumberLabel')}
              value={draft.phoneNumber}
              onChange={(event) => onPhoneNumberChange(event.target.value)}
              error={phoneIssue ? translateIssue(phoneIssue, translate) : undefined}
              placeholder={t('members.phoneNumberPlaceholder')}
              inputMode="tel"
            />
            <FormField
              label={t('members.employeeIdLabel')}
              value={draft.employeeId}
              onChange={(event) => onEmployeeIdChange(event.target.value)}
              placeholder={t('members.employeeIdPlaceholder')}
            />
            <div className="flex flex-col gap-y-2 @min-[36rem]:col-span-2">
              <div className="flex items-center gap-x-3">
                <Switch
                  id="member-watering-plan-selectable"
                  checked={draft.wateringPlanSelectable}
                  onCheckedChange={onWateringPlanSelectableChange}
                />
                <Label htmlFor="member-watering-plan-selectable">
                  {t('members.wateringPlanSelectableLabel')}
                </Label>
              </div>
              <p className="text-sm text-dark-600">{t('members.wateringPlanSelectableHint')}</p>
            </div>
          </>
        ) : (
          <>
            <StaticField
              label={t('members.availabilityLabel')}
              value={getUserStatusDetails(draft.status).label}
              fallback={notProvided}
            />
            <StaticField
              label={t('members.drivingLicensesLabel')}
              value={licenseLabels}
              fallback={notProvided}
            />
            <StaticField
              label={t('members.phoneNumberLabel')}
              value={draft.phoneNumber}
              fallback={notProvided}
            />
            <StaticField
              label={t('members.employeeIdLabel')}
              value={draft.employeeId}
              fallback={notProvided}
            />
            <StaticField
              label={t('members.wateringPlanSelectableLabel')}
              value={draft.wateringPlanSelectable ? t('members.yes') : t('members.no')}
              fallback={notProvided}
            />
          </>
        )}
      </div>
    </section>
  )
}

export default MemberProfileCard
