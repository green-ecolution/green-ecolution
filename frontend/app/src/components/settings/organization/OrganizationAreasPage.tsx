import { Suspense, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useBlocker } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Loading,
} from '@green-ecolution/ui'
import type { OrganizationSettingsResponse } from '@/api/backendApi'
import { settingsQueries } from '@/api/queries'
import { useSettingsMutations, useSettingsSave } from '@/hooks/useSettingsMutations'
import { statusOf } from '@/lib/httpError'
import OrganizationActionButtons from './OrganizationActionButtons'
import OrganizationModulesSection from './OrganizationModulesSection'
import OrganizationSettingsSection from './OrganizationSettingsSection'
import { useModuleActivationDraft } from './useModuleActivationDraft'
import { useModuleSettingsDrafts } from './useModuleSettingsDrafts'
import { useOrganizationSettingsDraft } from './useOrganizationSettingsDraft'
import { useSettingsOrganization } from './useSettingsOrganization'

const lockFingerprint = (
  orgId: string,
  settings: OrganizationSettingsResponse | undefined,
): string | null =>
  settings
    ? JSON.stringify([orgId, settings.descendantsMayOverride, settings.enforcedBy?.id ?? null])
    : null

const OrganizationAreasPage = () => {
  const { t } = useTranslation(['settings', 'common'])
  const { orgId, organizations, can } = useSettingsOrganization()
  const canUpdate = can(['setting:update'])
  const name = organizations.find((org) => org.id === orgId)?.name ?? ''

  const { updateSettings } = useSettingsMutations()
  const settingsSave = useSettingsSave()
  const settingsDraft = useOrganizationSettingsDraft()
  const [loadedKey, setLoadedKey] = useState<string | null>(null)

  const { data: settings, error: settingsError } = useQuery(settingsQueries.byOrganization(orgId))
  const moduleDraft = useModuleActivationDraft(orgId, { enabled: true })
  const moduleSections = useModuleSettingsDrafts(orgId, moduleDraft.isActive)
  const activeSections = moduleSections.filter((section) => section.active)

  const key = lockFingerprint(orgId, settings)
  if (settings && key !== loadedKey) {
    setLoadedKey(key)
    settingsDraft.load(settings)
  }

  // An invalid draft builds no request, so its `dirty` is false. Typed input
  // would then be dropped without a word on the way out, hence the `!valid` arm.
  const dirty =
    settingsDraft.dirty ||
    moduleDraft.dirty ||
    activeSections.some((section) => section.draft.dirty)
  const valid =
    settingsDraft.valid &&
    moduleDraft.valid &&
    activeSections.every((section) => section.draft.valid)
  const unsaved = dirty || !valid

  const blocker = useBlocker({
    shouldBlockFn: () => unsaved,
    enableBeforeUnload: () => unsaved,
    withResolver: true,
  })

  const nameOfOrg = (id: string | null | undefined): string | null =>
    id ? (organizations.find((org) => org.id === id)?.name ?? null) : null

  const reset = () => {
    if (settings) settingsDraft.load(settings)
    moduleDraft.reset()
    for (const section of moduleSections) section.draft.reset()
  }

  const save = () => {
    const body = settingsDraft.toRequest()
    void settingsSave.save([
      [
        ...(body ? [() => updateSettings.mutateAsync({ orgId, body })] : []),
        ...activeSections.filter(({ draft }) => draft.dirty).map(({ draft }) => draft.save),
      ],
      // After the sections: a module switched off in the same save refuses
      // writes to its settings once the switch is through.
      moduleDraft.dirty ? [moduleDraft.save] : [],
    ])
  }

  const loadError =
    settingsError ??
    moduleDraft.loadError ??
    activeSections.map(({ draft }) => draft.loadError).find((error) => error != null)
  const ready =
    settings !== undefined &&
    settingsDraft.draft !== null &&
    !moduleDraft.loading &&
    activeSections.every(({ draft }) => !draft.loading)
  const enforced = settings?.enforcedBy != null

  return (
    <div className="flex flex-col gap-6">
      <header>
        <h2 className="font-lato text-2xl font-bold text-dark">{t('organization.areas.title')}</h2>
        <p className="mt-1 max-w-[62ch] text-sm text-dark-600">
          {t('organization.areas.intro', { name })}
        </p>
      </header>

      {loadError ? (
        <p role="alert" className="text-sm text-dark-600">
          {t(
            statusOf(loadError) === 403
              ? 'organization.settings.loadErrorForbidden'
              : 'organization.settings.loadErrorGeneric',
          )}
        </p>
      ) : settings && settingsDraft.draft && ready ? (
        <>
          <OrganizationSettingsSection
            settings={settings}
            draft={settingsDraft.draft}
            canUpdate={canUpdate}
            enforcedByName={nameOfOrg(settings.enforcedBy?.id)}
            moduleSections={activeSections.map(({ key: sectionKey, draft, Section }) => (
              <Suspense key={sectionKey} fallback={null}>
                <Section
                  draft={draft}
                  locked={enforced || !canUpdate}
                  enforced={enforced}
                  organizationName={nameOfOrg}
                />
              </Suspense>
            ))}
            onDescendantsMayOverrideChange={settingsDraft.setDescendantsMayOverride}
          />
          <OrganizationModulesSection
            draft={moduleDraft}
            canUpdate={canUpdate}
            organizationName={nameOfOrg}
          />
        </>
      ) : (
        <Loading className="mt-10 justify-center" label={t('organization.areas.loading')} />
      )}

      {unsaved && (
        <div className="sticky bottom-0 flex flex-col-reverse gap-2 border-t border-dark-200 bg-dark-50 pb-3 pt-6 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
          <OrganizationActionButtons
            saving={settingsSave.pending}
            canSave={dirty}
            onSave={save}
            onCancel={reset}
          />
        </div>
      )}

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
    </div>
  )
}

export default OrganizationAreasPage
