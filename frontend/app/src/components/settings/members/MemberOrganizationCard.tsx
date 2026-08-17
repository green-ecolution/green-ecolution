import { Lock } from 'lucide-react'
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
}: MemberOrganizationCardProps) => (
  <section className={CARD}>
    <h3 className={CARD_TITLE}>Organisation</h3>
    <p className="mt-0.5 text-sm text-dark-600">
      Eine Person gehört zu höchstens einer Organisation. Ein Wechsel verschiebt sie sofort.
    </p>

    <p className="mt-4 text-sm font-semibold text-dark">
      {organization?.name ?? 'Keine Organisation zugeordnet'}
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
          placeholder="Organisation wechseln"
          searchPlaceholder="Organisation suchen"
          emptyText="Keine passende Organisation"
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

export default MemberOrganizationCard
