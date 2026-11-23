import { Suspense } from 'react'
import { useBlocker } from '@tanstack/react-router'
import { Blocks } from 'lucide-react'
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
  Button,
  Loading,
} from '@green-ecolution/ui'
import { useSettingsSave } from '@/hooks/useSettingsMutations'
import { statusOf } from '@/lib/httpError'
import { moduleManifests } from '@/modules/registry'
import ModuleAreaCard from './ModuleAreaCard'
import SensorsCard from './SensorsCard'
import { useModuleActivationDraft } from './useModuleActivationDraft'
import { useModuleSettingsDrafts } from './useModuleSettingsDrafts'
import { useSensorSettingsDraft } from './useSensorSettingsDraft'
import { useSettingsOrganization } from './useSettingsOrganization'

const OrganizationAreasPage = () => {
  const { t, i18n } = useTranslation(['settings', 'common'])
  const { orgId, organizations, can } = useSettingsOrganization()
  const canUpdate = can(['setting:update'])
  const name = organizations.find((org) => org.id === orgId)?.name ?? ''

  const settingsSave = useSettingsSave()
  const moduleDraft = useModuleActivationDraft(orgId, { enabled: true })
  const sensors = useSensorSettingsDraft(orgId, { enabled: true })
  // A card's fields follow the drafted switch: a module switched on here is
  // written first, so its values may be edited in the same save.
  const sections = useModuleSettingsDrafts(orgId, moduleDraft.isEnabled)
  const shown = sections.filter((section) => section.active)

  // An invalid draft builds no request, so its `dirty` is false. Typed input
  // would then be dropped without a word on the way out, hence the `!valid` arm.
  const dirty = moduleDraft.dirty || sensors.dirty || shown.some((section) => section.draft.dirty)
  const valid = sensors.valid && shown.every((section) => section.draft.valid)
  const unsaved = dirty || !valid

  const blocker = useBlocker({
    shouldBlockFn: () => unsaved,
    enableBeforeUnload: () => unsaved,
    withResolver: true,
  })

  const save = () => {
    // Module values are refused while the module is off, so switching on comes
    // before the values and switching off after them.
    void settingsSave.save([
      moduleDraft.enabling ? [moduleDraft.saveEnabling] : [],
      [
        ...(sensors.dirty ? [sensors.save] : []),
        ...shown.filter(({ draft }) => draft.dirty).map(({ draft }) => draft.save),
      ],
      moduleDraft.disabling ? [moduleDraft.saveDisabling] : [],
    ])
  }

  const loadError =
    moduleDraft.loadError ??
    sensors.loadError ??
    shown.map(({ draft }) => draft.loadError).find((error) => error != null)
  const ready = !moduleDraft.loading && !sensors.loading

  return (
    <div className="flex flex-col gap-6">
      <header>
        <h2 className="font-lato text-2xl font-bold text-dark">{t('organization.areas.title')}</h2>
        <p className="mt-1 max-w-[62ch] text-sm text-dark-600">
          {t('organization.areas.intro', { name })}
        </p>
        {!canUpdate && (
          <p className="mt-2 text-sm text-dark-600">{t('organization.areas.readOnlyNotice')}</p>
        )}
      </header>

      {loadError ? (
        <p role="alert" className="text-sm text-dark-600">
          {t(
            statusOf(loadError) === 403
              ? 'organization.areas.loadErrorForbidden'
              : 'organization.areas.loadErrorGeneric',
          )}
        </p>
      ) : ready ? (
        <>
          {moduleDraft.rows.map((row) => {
            const manifest = moduleManifests.find((candidate) => candidate.key === row.key)
            const section = shown.find((candidate) => candidate.key === row.key)
            return (
              <ModuleAreaCard
                key={row.key}
                icon={manifest?.icon ?? Blocks}
                title={manifest ? i18n.t(manifest.labelKey) : row.key}
                description={manifest ? i18n.t(manifest.descriptionKey) : null}
                toggle={{
                  enabled: row.enabled,
                  note: row.switchable ? null : t('organization.areas.followsTenant'),
                  control: row.switchable
                    ? {
                        disabled: !canUpdate,
                        onChange: (enabled) => moduleDraft.setEnabled(row.key, enabled),
                      }
                    : undefined,
                }}
              >
                {section && !section.draft.loading && (
                  <Suspense fallback={null}>
                    <section.Section draft={section.draft} locked={!canUpdate} />
                  </Suspense>
                )}
              </ModuleAreaCard>
            )
          })}
          <SensorsCard draft={sensors} locked={!canUpdate} />
        </>
      ) : (
        <Loading className="mt-10 justify-center" label={t('organization.areas.loading')} />
      )}

      {canUpdate && ready && !loadError && (
        <div className="flex flex-wrap items-center justify-end gap-3">
          <p className="flex-[1_1_15rem] text-sm text-dark-600">
            {t('organization.areas.saveHint')}
          </p>
          <Button type="button" disabled={!dirty || !valid || settingsSave.pending} onClick={save}>
            {t('organization.areas.save')}
          </Button>
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
