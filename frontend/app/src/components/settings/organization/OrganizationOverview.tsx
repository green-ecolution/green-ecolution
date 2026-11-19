import { useEffect, useRef, type ReactNode } from 'react'
import { Link } from '@tanstack/react-router'
import { ArrowRight, Lock, Trash2, UserPlus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Button,
  FormField,
  Separator,
} from '@green-ecolution/ui'
import type { OrganizationDetailResponse } from '@/api/backendApi'
import { initialsOf, initialsOfName } from '@/lib/initials'
import { CARD, CARD_TITLE } from '../cardStyles'
import OrganizationActionButtons from './OrganizationActionButtons'
import type { AddressFieldErrors, OrganizationDraft } from './useOrganizationDraft'

export interface OverviewLink {
  key: 'members' | 'structure' | 'areas'
  to:
    | '/settings/organization/$orgId/members'
    | '/settings/organization/$orgId/structure'
    | '/settings/organization/$orgId/areas'
  title: string
  summary: string
}

interface OrganizationOverviewProps {
  detail: OrganizationDetailResponse
  draft: OrganizationDraft
  dirty: boolean
  addressErrors: AddressFieldErrors
  /** Whether pressing save would send anything; decided on the page, which is
   *  the only place that sees both endpoints. */
  canSave: boolean
  /** True for the topmost organization of the instance, which nobody may edit. */
  readOnly: boolean
  canUpdate: boolean
  canDelete: boolean
  /** The map view card; null without `setting:read`. */
  mapViewSection: ReactNode
  links: OverviewLink[]
  saving: boolean
  nameError: string | null
  /** 422 from the backend when the person is not a member of this organization. */
  contactPersonError: string | null
  onNameChange: (value: string) => void
  onStreetChange: (value: string) => void
  onPostalCodeChange: (value: string) => void
  onCityChange: (value: string) => void
  onContactPersonRequest: () => void
  onSave: () => void
  onCancel: () => void
  onDelete: () => void
}

const TILE = 'flex shrink-0 items-center justify-center rounded-lg font-semibold'

const sinceLabel = (createdAt?: string | null): string | null => {
  if (createdAt == null) return null
  const date = new Date(createdAt)
  if (Number.isNaN(date.getTime())) return null
  return `${String(date.getMonth() + 1).padStart(2, '0')}/${date.getFullYear()}`
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

const OrganizationOverview = ({
  detail,
  draft,
  dirty,
  addressErrors,
  canSave,
  readOnly,
  canUpdate,
  canDelete,
  mapViewSection,
  links,
  saving,
  nameError,
  contactPersonError,
  onNameChange,
  onStreetChange,
  onPostalCodeChange,
  onCityChange,
  onContactPersonRequest,
  onSave,
  onCancel,
  onDelete,
}: OrganizationOverviewProps) => {
  const { t } = useTranslation(['settings', 'common'])
  const nameRef = useRef<HTMLInputElement>(null)
  const contactRef = useRef<HTMLButtonElement>(null)

  // On mobile the save button sits in the drawer footer, so a rejected save
  // would otherwise leave its only explanation scrolled out of view.
  useEffect(() => {
    if (nameError) nameRef.current?.focus()
    else if (contactPersonError) contactRef.current?.focus()
  }, [nameError, contactPersonError])

  const locked = readOnly || !canUpdate
  const nameEmpty = draft.name.trim().length === 0
  const since = sinceLabel(detail.createdAt)
  const kind = t(detail.parentId == null ? 'organization.topLevelKind' : 'organization.childKind')
  const contact = detail.contactPerson
  const notProvided = t('notProvided')

  return (
    <div className="@container flex flex-col gap-6">
      <header className="flex min-w-0 items-start gap-4">
        <span className={`${TILE} size-12 rounded-xl bg-green-dark text-base text-white`}>
          {initialsOfName(detail.name)}
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="font-lato text-2xl font-bold text-dark break-words">{detail.name}</h2>
          <p className="mt-0.5 text-sm text-dark-600">
            {since ? `${kind} · ${t('organization.sinceLabel', { since })}` : kind}
          </p>
        </div>
      </header>

      {readOnly && (
        <Alert variant="info" size="default" className="flex w-full gap-4">
          <AlertIcon variant="info" icon={Lock} />
          <AlertContent>
            <AlertDescription>{t('organization.readOnlyRootNotice')}</AlertDescription>
          </AlertContent>
        </Alert>
      )}

      <div className="grid gap-4 @min-[48rem]:grid-cols-2">
        <section className={CARD}>
          <h3 className={CARD_TITLE}>{t('organization.masterDataTitle')}</h3>
          <div className="mt-4">
            {locked ? (
              <StaticField
                label={t('organization.nameLabel')}
                value={draft.name}
                fallback={notProvided}
              />
            ) : (
              // FormField owns aria-invalid and aria-describedby; passing
              // aria-invalid in would set it without an associated reason.
              <FormField
                ref={nameRef}
                label={t('organization.nameLabel')}
                value={draft.name}
                onChange={(event) => onNameChange(event.target.value)}
                error={nameError ?? (nameEmpty ? t('organization.nameEmptyHint') : undefined)}
                placeholder={t('organization.namePlaceholder')}
              />
            )}
          </div>
        </section>

        <section className={CARD}>
          <h3 className={CARD_TITLE}>{t('organization.locationTitle')}</h3>
          <p className="mt-0.5 text-sm text-dark-600">{t('organization.locationHint')}</p>
          <div className="mt-4 flex flex-col gap-4">
            {locked ? (
              <>
                <StaticField
                  label={t('organization.streetLabel')}
                  value={draft.street}
                  fallback={notProvided}
                />
                <div className="grid gap-4 @min-[22rem]:grid-cols-[8rem_1fr]">
                  <StaticField
                    label={t('organization.postalCodeLabel')}
                    value={draft.postalCode}
                    fallback={notProvided}
                  />
                  <StaticField
                    label={t('organization.cityLabel')}
                    value={draft.city}
                    fallback={notProvided}
                  />
                </div>
              </>
            ) : (
              <>
                <FormField
                  label={t('organization.streetLabel')}
                  value={draft.street}
                  onChange={(event) => onStreetChange(event.target.value)}
                  error={addressErrors.street}
                  placeholder={t('organization.streetPlaceholder')}
                />
                <div className="grid gap-4 @min-[22rem]:grid-cols-[8rem_1fr]">
                  <FormField
                    label={t('organization.postalCodeLabel')}
                    value={draft.postalCode}
                    onChange={(event) => onPostalCodeChange(event.target.value)}
                    error={addressErrors.postalCode}
                    placeholder={t('organization.postalCodePlaceholder')}
                    inputMode="numeric"
                  />
                  <FormField
                    label={t('organization.cityLabel')}
                    value={draft.city}
                    onChange={(event) => onCityChange(event.target.value)}
                    error={addressErrors.city}
                    placeholder={t('organization.cityPlaceholder')}
                  />
                </div>
              </>
            )}
          </div>
        </section>

        <section className={`${CARD} @min-[48rem]:col-span-2`}>
          <h3 className={CARD_TITLE}>{t('organization.contactPersonTitle')}</h3>
          <div className="mt-4 flex flex-wrap items-center gap-4">
            {contact ? (
              <>
                <span
                  className={`${TILE} size-10 bg-green-light-100 text-sm text-green-dark`}
                  aria-hidden
                >
                  {initialsOf(contact.firstName, contact.lastName)}
                </span>
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-semibold text-dark">
                    {contact.firstName} {contact.lastName}
                  </p>
                  <p className="truncate text-sm text-dark-600">{contact.email}</p>
                </div>
                {!locked && (
                  <Button
                    ref={contactRef}
                    type="button"
                    variant="outline"
                    size="sm"
                    onClick={onContactPersonRequest}
                  >
                    {t('organization.changeContactPerson')}
                  </Button>
                )}
              </>
            ) : !locked ? (
              <Button
                ref={contactRef}
                type="button"
                variant="outline"
                size="sm"
                onClick={onContactPersonRequest}
              >
                <UserPlus className="size-4" aria-hidden />
                {t('organization.setContactPerson')}
              </Button>
            ) : (
              <p className="text-sm text-dark-600">{t('organization.noContactPerson')}</p>
            )}
          </div>
          {contactPersonError && (
            <p role="alert" aria-live="assertive" className="mt-2 text-sm text-destructive">
              {contactPersonError}
            </p>
          )}
        </section>
      </div>

      {mapViewSection}

      {links.length > 0 && (
        <ul className="grid list-none gap-4 @min-[48rem]:grid-cols-3">
          {links.map((link) => (
            <li key={link.key}>
              <Link
                to={link.to}
                params={{ orgId: detail.id }}
                className={`${CARD} flex h-full items-center gap-3 hover:border-green-dark focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring`}
              >
                <span className="min-w-0 flex-1">
                  <span className={`block ${CARD_TITLE}`}>{link.title}</span>
                  <span className="block text-sm text-dark-600">{link.summary}</span>
                </span>
                <ArrowRight className="size-4 shrink-0 text-dark-500" aria-hidden />
              </Link>
            </li>
          ))}
        </ul>
      )}

      {canDelete && !readOnly && (
        <div className="flex flex-col gap-3">
          <Separator />
          <div className="flex flex-wrap items-center justify-between gap-2">
            <p className="text-sm text-dark-600">{t('organization.deleteHint')}</p>
            <Button type="button" variant="ghost-destructive" size="sm" onClick={onDelete}>
              <Trash2 className="size-4" aria-hidden />
              {t('organization.deleteButton')}
            </Button>
          </div>
        </div>
      )}

      {/* `readOnly` does not hide the bar: the instance root refuses its master
          data but still keeps the defaults every organization below inherits. */}
      {dirty && (
        <div className="sticky bottom-0 -mt-6 flex flex-col-reverse gap-2 border-t border-dark-200 bg-dark-50 pb-3 pt-6 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
          <OrganizationActionButtons
            saving={saving}
            canSave={canSave}
            onSave={onSave}
            onCancel={onCancel}
          />
        </div>
      )}
    </div>
  )
}

export default OrganizationOverview
