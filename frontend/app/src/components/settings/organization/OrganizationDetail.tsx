import { Fragment, useEffect, useRef } from 'react'
import { Link } from '@tanstack/react-router'
import { ArrowRight, ChevronRight, Lock, Plus, Trash2, UserPlus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AvatarStack,
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Button,
  FormField,
  ListCard,
  Separator,
} from '@green-ecolution/ui'
import type { OrganizationDetailResponse, OrganizationResponse } from '@/api/backendApi'
import { initialsOf, initialsOfName } from '@/lib/initials'
import OrganizationActionButtons from './OrganizationActionButtons'
import { directChildrenLine, memberSubtitle, memberSummary } from './organizationLabels'
import type { OrgNode } from './organizationTree'
import type { AddressFieldErrors, OrganizationDraft } from './useOrganizationDraft'

interface OrganizationDetailProps {
  node: OrgNode
  /** Ancestors down to and including this organization, for the breadcrumb. */
  path: OrganizationResponse[]
  detail: OrganizationDetailResponse
  draft: OrganizationDraft
  dirty: boolean
  addressErrors: AddressFieldErrors
  addressComplete: boolean
  /** True for the topmost organization of the instance, which nobody may edit. */
  readOnly: boolean
  canUpdate: boolean
  canCreate: boolean
  canDelete: boolean
  canReadUsers: boolean
  memberInitials: string[]
  saving: boolean
  nameError: string | null
  /** 422 from the backend when the person is not a member of this organization. */
  contactPersonError: string | null
  onNameChange: (value: string) => void
  onStreetChange: (value: string) => void
  onPostalCodeChange: (value: string) => void
  onCityChange: (value: string) => void
  onContactPersonRequest: () => void
  onSubOrganizationCreate: () => void
  onSelectChild: (org: OrganizationResponse) => void
  onSave: () => void
  onCancel: () => void
  onDelete: () => void
  renderActionBar: boolean
}

const CARD = '@container rounded-xl border border-dark-50 bg-white p-5 shadow-cards'
const CARD_TITLE = 'font-lato text-base font-semibold text-dark'
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

const OrganizationDetail = ({
  node,
  path,
  detail,
  draft,
  dirty,
  addressErrors,
  addressComplete,
  readOnly,
  canUpdate,
  canCreate,
  canDelete,
  canReadUsers,
  memberInitials,
  saving,
  nameError,
  contactPersonError,
  onNameChange,
  onStreetChange,
  onPostalCodeChange,
  onCityChange,
  onContactPersonRequest,
  onSubOrganizationCreate,
  onSelectChild,
  onSave,
  onCancel,
  onDelete,
  renderActionBar,
}: OrganizationDetailProps) => {
  const { t } = useTranslation(['settings', 'common'])
  const nameRef = useRef<HTMLInputElement>(null)
  const contactRef = useRef<HTMLButtonElement>(null)

  // On mobile the save button sits in the drawer footer, so a rejected save
  // would otherwise leave its only explanation scrolled out of view.
  useEffect(() => {
    if (nameError) nameRef.current?.focus()
    else if (contactPersonError) contactRef.current?.focus()
  }, [nameError, contactPersonError])

  const trail = path.slice(0, -1)
  const locked = readOnly || !canUpdate
  const nameEmpty = draft.name.trim().length === 0
  const since = sinceLabel(detail.createdAt)
  const kind = t(detail.parentId == null ? 'organization.topLevelKind' : 'organization.childKind')
  const contact = detail.contactPerson
  const children = node.children
  const notProvided = t('notProvided')

  return (
    <div className="flex flex-col gap-6">
      {trail.length > 0 && (
        <Breadcrumb>
          <BreadcrumbList>
            {trail.map((org) => (
              <Fragment key={org.id}>
                <BreadcrumbItem>
                  <BreadcrumbLink asChild>
                    <button
                      type="button"
                      onClick={() => onSelectChild(org)}
                      className="rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                    >
                      {org.name}
                    </button>
                  </BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator />
              </Fragment>
            ))}
            <BreadcrumbItem>
              <BreadcrumbPage>{detail.name}</BreadcrumbPage>
            </BreadcrumbItem>
          </BreadcrumbList>
        </Breadcrumb>
      )}

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

      <div className="@container">
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

          <section className={`${CARD} ${canReadUsers ? '' : '@min-[48rem]:col-span-2'}`}>
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

          {canReadUsers && (
            <section className={CARD}>
              <h3 className={CARD_TITLE}>{t('organization.assignedMembersTitle')}</h3>
              <div className="mt-4 flex flex-wrap items-center gap-4">
                <AvatarStack items={memberInitials} />
                <p className="min-w-0 flex-1 text-sm text-dark-600">{memberSummary(node, t)}</p>
              </div>
              <Link
                to="/settings/team/members"
                className="mt-4 inline-flex items-center gap-1 text-sm font-semibold text-green-dark hover:underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                {t('organization.manageMembers')}
                <ArrowRight className="size-4" aria-hidden />
              </Link>
            </section>
          )}

          <section className={`${CARD} @min-[48rem]:col-span-2`}>
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="min-w-0">
                <h3 className={CARD_TITLE}>{t('organization.subOrganizationsTitle')}</h3>
                <p className="mt-0.5 text-sm text-dark-600">
                  {children.length === 0
                    ? t('organization.noSubOrganizations')
                    : directChildrenLine(children.length, t)}
                </p>
              </div>
              {canCreate && (
                <Button type="button" variant="outline" size="sm" onClick={onSubOrganizationCreate}>
                  <Plus className="size-4" aria-hidden />
                  {t('organization.createSubOrganization')}
                </Button>
              )}
            </div>

            {children.length > 0 && (
              <ul className="mt-4 flex list-none flex-col gap-2">
                {children.map((child) => {
                  const subtitle = memberSubtitle(child, t)
                  return (
                    <li key={child.org.id}>
                      <ListCard size="compact" hoverable asChild>
                        <button
                          type="button"
                          onClick={() => onSelectChild(child.org)}
                          className="w-full text-left"
                        >
                          <span
                            className={`${TILE} size-8 bg-green-light-100 text-sm text-green-dark`}
                            aria-hidden
                          >
                            {initialsOfName(child.org.name)}
                          </span>
                          <span className="min-w-0 flex-1">
                            <span className="block truncate font-lato text-sm font-semibold text-dark">
                              {child.org.name}
                            </span>
                            {subtitle && (
                              <span className="block truncate text-xs text-dark-600">
                                {subtitle}
                              </span>
                            )}
                          </span>
                          <ChevronRight className="size-4 shrink-0 text-dark-500" aria-hidden />
                        </button>
                      </ListCard>
                    </li>
                  )
                })}
              </ul>
            )}
          </section>
        </div>
      </div>

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

      {/* --org-panel-bg lets the sticky action bar blend into its surface; the page sets it. */}
      {renderActionBar && !readOnly && dirty && (
        <div className="sticky bottom-0 -mt-6 flex flex-col-reverse gap-2 border-t border-dark-200 bg-[var(--org-panel-bg,var(--color-dark-50))] pb-3 pt-6 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
          <OrganizationActionButtons
            saving={saving}
            nameEmpty={nameEmpty}
            addressComplete={addressComplete}
            onSave={onSave}
            onCancel={onCancel}
          />
        </div>
      )}
    </div>
  )
}

export default OrganizationDetail
