import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useBlocker, useNavigate } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import type { TFunction } from 'i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@green-ecolution/ui'
import type {
  AddressDto,
  OrganizationDetailResponse,
  OrganizationSettingsResponse,
} from '@/api/backendApi'
import { modulesQuery, organizationQueries, settingsQueries, userQueries } from '@/api/queries'
import { useOrganizationMutations } from '@/hooks/useOrganizationMutations'
import { useSettingsMutations, useSettingsSave } from '@/hooks/useSettingsMutations'
import { statusOf } from '@/lib/httpError'
import { moduleManifests } from '@/modules/registry'
import { CARD, CARD_TITLE } from '../cardStyles'
import ContactPersonPicker from './ContactPersonPicker'
import MapViewSettingRow from './MapViewSettingRow'
import OrganizationOverview, { type OverviewLink } from './OrganizationOverview'
import { directChildrenLine, memberSummary } from './organizationLabels'
import { buildTree } from './organizationTree'
import { useOrganizationDraft } from './useOrganizationDraft'
import { useOrganizationSettingsDraft } from './useOrganizationSettingsDraft'
import { useSettingsOrganization } from './useSettingsOrganization'

const MEMBERS_PER_PAGE = 100

const loadErrorMessage = (error: unknown, t: TFunction<'settings'>): string =>
  t(statusOf(error) === 403 ? 'organization.loadErrorForbidden' : 'organization.loadErrorGeneric')

const nameConflictMessage = (error: unknown, t: TFunction<'settings'>): string | null =>
  statusOf(error) === 409 ? t('organization.nameConflict') : null

const contactPersonMessage = (error: unknown, t: TFunction<'settings'>): string | null =>
  statusOf(error) === 422 ? t('organization.contactPersonNotAssigned') : null

/** A refetch that changes none of this must leave an edit in progress alone. */
const mapViewFingerprint = (
  orgId: string,
  settings: OrganizationSettingsResponse | undefined,
): string | null =>
  settings
    ? JSON.stringify([
        orgId,
        settings.mapView.value,
        settings.mapView.origin,
        settings.mapView.ownValue ?? null,
        settings.enforcedBy?.id ?? null,
      ])
    : null

const OrganizationOverviewPage = () => {
  const { t, i18n } = useTranslation(['settings', 'common'])
  const navigate = useNavigate()
  const { orgId, organizations, readableIds, can } = useSettingsOrganization()
  const canUpdate = can(['organization:update'])
  const canDelete = can(['organization:delete'])
  const canReadUsers = can(['user:read'])
  const canReadSettings = can(['setting:read'])
  const canUpdateSettings = can(['setting:update'])

  const { updateOrganization, deleteOrganization } = useOrganizationMutations()
  const { updateSettings } = useSettingsMutations()
  const settingsSave = useSettingsSave()
  const draftState = useOrganizationDraft()
  const { draft, dirty, addressErrors, addressComplete, edit } = draftState
  const settingsDraft = useOrganizationSettingsDraft()

  const [loadedDetail, setLoadedDetail] = useState<OrganizationDetailResponse | null>(null)
  const [loadedSettingsKey, setLoadedSettingsKey] = useState<string | null>(null)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [pickerOpen, setPickerOpen] = useState(false)

  const { data: detail, error: detailError } = useQuery(organizationQueries.byId(orgId))
  const { data: memberPage } = useQuery({
    ...userQueries.list({ page: 1, perPage: MEMBERS_PER_PAGE, organizationId: orgId }),
    enabled: canReadUsers,
  })
  const { data: settings, error: settingsError } = useQuery({
    ...settingsQueries.byOrganization(orgId),
    enabled: canReadSettings,
  })
  const { data: moduleList } = useQuery({ ...modulesQuery(), enabled: canReadSettings })

  // Adjusted while rendering instead of in an effect: after a save the query
  // hands back a new object and the draft has to follow the server.
  if (detail && detail !== loadedDetail) {
    setLoadedDetail(detail)
    edit(detail)
  }
  const settingsKey = mapViewFingerprint(orgId, settings)
  if (settings && settingsKey !== loadedSettingsKey) {
    setLoadedSettingsKey(settingsKey)
    settingsDraft.load(settings)
  }

  const unsaved = dirty || settingsDraft.dirty || !settingsDraft.valid
  const masterDataValid = draft !== null && draft.name.trim().length > 0 && addressComplete
  const canSave = (dirty && masterDataValid) || settingsDraft.dirty

  const blocker = useBlocker({
    shouldBlockFn: () => unsaved,
    enableBeforeUnload: () => unsaved,
    withResolver: true,
  })

  if (detailError) {
    return (
      <p role="alert" className="text-sm text-dark-600">
        {loadErrorMessage(detailError, t)}
      </p>
    )
  }
  if (!detail || !draft) return null

  const readOnly = detail.parentId == null
  const node = buildTree(organizations, orgId)
  const nameOfOrg = (id: string | null | undefined): string | null =>
    id ? (organizations.find((org) => org.id === id)?.name ?? null) : null

  const resetDraft = () => {
    edit(detail)
    if (settings) settingsDraft.load(settings)
  }

  const save = () => {
    const street = draft.street.trim()
    const postalCode = draft.postalCode.trim()
    const city = draft.city.trim()
    const filled = [street, postalCode, city].filter((value) => value.length > 0)
    const address: AddressDto | null = filled.length === 3 ? { street, postalCode, city } : null

    // Two endpoints behind one button: each half goes out only if it changed
    // and is valid on its own, so neither holds the other back.
    if (dirty && masterDataValid) {
      updateOrganization.mutate({
        orgId,
        name: draft.name.trim(),
        address,
        contactPersonId: draft.contactPersonId,
      })
    }
    const settingsBody = settingsDraft.toRequest()
    void settingsSave.save([
      settingsBody ? [() => updateSettings.mutateAsync({ orgId, body: settingsBody })] : [],
    ])
  }

  const remove = () => {
    deleteOrganization.mutate(
      { orgId },
      {
        onSuccess: () => {
          setConfirmDelete(false)
          // The parent's overview needs organization:read there; an organization the
          // user only reaches would end on Forbidden, so the entry picks a page instead.
          const parentReadable = !!detail.parentId && readableIds.has(detail.parentId)
          void (parentReadable && detail.parentId
            ? navigate({
                to: '/settings/organization/$orgId',
                params: { orgId: detail.parentId },
                ignoreBlocker: true,
              })
            : navigate({ to: '/settings/organization', ignoreBlocker: true }))
        },
        // A 409 explains itself in a toast, which the open dialog would cover.
        onError: () => setConfirmDelete(false),
      },
    )
  }

  const activeKeys =
    moduleList?.organizations.find((entry) => entry.organizationId === orgId)?.active ?? []
  const activeLabels = moduleManifests
    .filter((manifest) => activeKeys.includes(manifest.key))
    .map((manifest) => i18n.t(manifest.labelKey))

  const links: OverviewLink[] = [
    ...(canReadUsers && node
      ? [
          {
            key: 'members' as const,
            to: '/settings/organization/$orgId/members' as const,
            title: t('organization.overview.membersLink'),
            summary: memberSummary(node, t),
          },
        ]
      : []),
    {
      key: 'structure' as const,
      to: '/settings/organization/$orgId/structure' as const,
      title: t('organization.overview.structureLink'),
      summary:
        node && node.children.length > 0
          ? directChildrenLine(node.children.length, t)
          : t('organization.overview.structureNone'),
    },
    ...(canReadSettings
      ? [
          {
            key: 'areas' as const,
            to: '/settings/organization/$orgId/areas' as const,
            title: t('organization.overview.areasLink'),
            summary:
              activeLabels.length > 0
                ? t('organization.overview.areasActive', {
                    count: activeLabels.length,
                    modules: activeLabels,
                  })
                : t('organization.overview.areasNone'),
          },
        ]
      : []),
  ]

  const enforced = settings?.enforcedBy != null
  const mapViewSection = !canReadSettings ? null : settingsError ? (
    <p role="alert" className="text-sm text-dark-600">
      {t(
        statusOf(settingsError) === 403
          ? 'organization.settings.loadErrorForbidden'
          : 'organization.settings.loadErrorGeneric',
      )}
    </p>
  ) : settings && settingsDraft.draft ? (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('organization.overview.mapViewTitle')}</h3>
      <div className="mt-4">
        <MapViewSettingRow
          label={t('organization.settings.mapView.label')}
          description={t('organization.settings.mapView.hint')}
          effective={settings.mapView.value}
          draft={settingsDraft.draft.mapView.value}
          origin={settings.mapView.origin}
          sourceName={nameOfOrg(settings.mapView.source?.id)}
          dormant={enforced ? (settings.mapView.ownValue ?? null) : null}
          lastChange={settings.mapView.lastChange}
          own={settingsDraft.draft.mapView.own}
          locked={enforced || !canUpdateSettings}
          error={settingsDraft.errors.mapView}
          onOwn={settingsDraft.setMapViewOwn}
          onChange={settingsDraft.setMapView}
          onRestrictedChange={settingsDraft.setMapViewRestricted}
          onZoomChange={settingsDraft.setMapViewZoom}
        />
      </div>
    </section>
  ) : null

  return (
    <>
      <OrganizationOverview
        detail={detail}
        draft={draft}
        dirty={unsaved}
        addressErrors={addressErrors}
        canSave={canSave}
        readOnly={readOnly}
        canUpdate={canUpdate}
        canDelete={canDelete}
        mapViewSection={mapViewSection}
        links={links}
        saving={updateOrganization.isPending || settingsSave.pending}
        nameError={nameConflictMessage(updateOrganization.error, t)}
        contactPersonError={contactPersonMessage(updateOrganization.error, t)}
        onNameChange={draftState.setName}
        onStreetChange={draftState.setStreet}
        onPostalCodeChange={draftState.setPostalCode}
        onCityChange={draftState.setCity}
        onContactPersonRequest={() => setPickerOpen(true)}
        onSave={save}
        onCancel={resetDraft}
        onDelete={() => setConfirmDelete(true)}
      />

      <ContactPersonPicker
        open={pickerOpen}
        members={memberPage?.data ?? []}
        selectedId={draft.contactPersonId}
        onOpenChange={setPickerOpen}
        onSelect={(userId) => {
          draftState.setContactPersonId(userId)
          setPickerOpen(false)
        }}
      />

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('organization.deleteDialog.title')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('organization.deleteDialog.description', { name: detail.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common:actions.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={remove}>{t('common:actions.delete')}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {blocker.status === 'blocked' && (
        <AlertDialog open onOpenChange={() => blocker.reset?.()}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>{t('common:dialog.unsavedChanges.title')}</AlertDialogTitle>
              <AlertDialogDescription>{t('organization.leaveDescription')}</AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel onClick={() => blocker.reset?.()}>
                {t('dialog.discardChanges.continueEditing')}
              </AlertDialogCancel>
              <AlertDialogAction onClick={() => blocker.proceed?.()}>
                {t('common:dialog.unsavedChanges.confirm')}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      )}
    </>
  )
}

export default OrganizationOverviewPage
