import { Lock } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Alert, AlertContent, AlertDescription, AlertIcon, Combobox } from '@green-ecolution/ui'
import type { OrganizationResponse } from '@/api/backendApi'
import { CARD, CARD_TITLE } from './cardChrome'

interface MemberOrganizationCardProps {
  organization: OrganizationResponse | undefined
  organizations: OrganizationResponse[]
  editable: boolean
  lockedReason: string | null
  error: string | null
  onChange: (orgId: string) => void
}

const MemberOrganizationCard = ({
  organization,
  organizations,
  editable,
  lockedReason,
  error,
  onChange,
}: MemberOrganizationCardProps) => {
  const { t } = useTranslation('settings')

  return (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('members.organizationTitle')}</h3>
      <p className="mt-0.5 text-sm text-dark-600">{t('members.organizationHint')}</p>

      <p className="mt-4 text-sm font-semibold text-dark">
        {organization?.name ?? t('members.noOrganizationAssigned')}
      </p>

      {lockedReason && (
        <Alert variant="info" size="default" className="mt-4 flex w-full gap-4">
          <AlertIcon variant="info" icon={Lock} />
          <AlertContent>
            <AlertDescription>{lockedReason}</AlertDescription>
          </AlertContent>
        </Alert>
      )}

      {editable && (
        <div className="mt-4">
          <Combobox
            options={organizations
              .filter((org) => org.id !== organization?.id)
              .map((org) => ({ value: org.id, label: org.name }))}
            onChange={onChange}
            placeholder={t('members.changeOrganizationPlaceholder')}
            searchPlaceholder={t('members.changeOrganizationSearchPlaceholder')}
            emptyText={t('members.changeOrganizationEmptyText')}
            aria-label={t('members.changeOrganizationAriaLabel')}
            aria-describedby={error ? 'member-organization-error' : undefined}
            aria-invalid={!!error}
          />
        </div>
      )}

      {error && (
        <p
          id="member-organization-error"
          role="alert"
          aria-live="assertive"
          className="mt-2 text-sm text-red"
        >
          {error}
        </p>
      )}
    </section>
  )
}

export default MemberOrganizationCard
