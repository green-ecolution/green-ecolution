import { useId, type ReactNode } from 'react'
import { Lock, LockOpen } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Label,
  Separator,
  Switch,
} from '@green-ecolution/ui'
import type { MapViewDto, OrganizationSettingsResponse } from '@/api/backendApi'
import { CARD, CARD_TITLE } from './OrganizationDetail'
import MapViewSettingRow from './MapViewSettingRow'
import type { SettingsDraft } from './useOrganizationSettingsDraft'

export interface SettingsFieldErrors {
  mapView: string | null
}

export type SettingsSourceNames = Record<'mapView', string | null>

interface OrganizationSettingsSectionProps {
  settings: OrganizationSettingsResponse
  draft: SettingsDraft
  errors: SettingsFieldErrors
  canUpdate: boolean
  /** Name of the organization that froze this subtree, if any. */
  enforcedByName: string | null
  sourceNames: SettingsSourceNames
  /** The active modules' own rows, ahead of the core ones. */
  moduleSections?: ReactNode
  onMapViewOwnChange: (own: boolean) => void
  onMapViewChange: (view: MapViewDto) => void
  onMapViewRestrictedChange: (restricted: boolean) => void
  onMapViewZoomChange: (which: 'minZoom' | 'maxZoom', level: number) => void
  onDescendantsMayOverrideChange: (value: boolean) => void
}

const OrganizationSettingsSection = ({
  settings,
  draft,
  errors,
  canUpdate,
  enforcedByName,
  sourceNames,
  moduleSections,
  onMapViewOwnChange,
  onMapViewChange,
  onMapViewRestrictedChange,
  onMapViewZoomChange,
  onDescendantsMayOverrideChange,
}: OrganizationSettingsSectionProps) => {
  const { t } = useTranslation('settings')
  const switchId = useId()
  // Read off the response, not off the name: an organization above the visible
  // list locks just the same, it just cannot be named.
  const enforced = settings.enforcedBy != null
  // The lock covers the switch itself — the backend refuses the whole write,
  // so a sub-unit cannot unlock itself.
  const locked = enforced || !canUpdate

  const descendantsMayOverride = draft.descendantsMayOverride

  return (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('organization.settings.title')}</h3>
      <p className="mt-1 max-w-[70ch] text-sm text-dark-600">{t('organization.settings.hint')}</p>

      {enforced ? (
        <Alert variant="info" size="default" className="mt-4 flex w-full gap-4" role="alert">
          <AlertIcon variant="info" icon={Lock} />
          <AlertContent>
            <AlertDescription>
              {enforcedByName
                ? t('organization.settings.enforcedNotice', { name: enforcedByName })
                : t('organization.settings.enforcedNoticeUnknown')}
            </AlertDescription>
          </AlertContent>
        </Alert>
      ) : (
        // Without the permission the take-over actions are gone; say why rather
        // than leave an unexplained gap where they were.
        !canUpdate && (
          <p className="mt-2 text-sm text-dark-600">{t('organization.settings.readOnlyNotice')}</p>
        )
      )}

      {moduleSections}

      {/* A full-width row: a map needs the width to be worth looking at. */}
      <div className="mt-6">
        <MapViewSettingRow
          label={t('organization.settings.mapView.label')}
          description={t('organization.settings.mapView.hint')}
          effective={settings.mapView.value}
          draft={draft.mapView.value}
          origin={settings.mapView.origin}
          sourceName={sourceNames.mapView}
          dormant={enforced ? (settings.mapView.ownValue ?? null) : null}
          lastChange={settings.mapView.lastChange}
          own={draft.mapView.own}
          locked={locked}
          error={errors.mapView}
          onOwn={onMapViewOwnChange}
          onChange={onMapViewChange}
          onRestrictedChange={onMapViewRestrictedChange}
          onZoomChange={onMapViewZoomChange}
        />
      </div>

      <Separator className="my-5" />

      <div className="flex flex-wrap items-start justify-between gap-x-6 gap-y-3">
        <div className="flex min-w-0 flex-1 gap-3">
          {descendantsMayOverride ? (
            <LockOpen className="mt-0.5 size-4 shrink-0 text-dark-500" aria-hidden />
          ) : (
            <Lock className="mt-0.5 size-4 shrink-0 text-dark" aria-hidden />
          )}
          <div className="min-w-0">
            <Label htmlFor={switchId}>{t('organization.settings.descendantsLabel')}</Label>
            {/* The consequence of switching it off reaches the whole subtree, so
                spell out the state the person is in rather than both at once. */}
            <p className="mt-1 max-w-[62ch] text-sm text-dark-600">
              {descendantsMayOverride
                ? t('organization.settings.descendantsHintOn')
                : t('organization.settings.descendantsHintOff')}
            </p>
          </div>
        </div>
        <Switch
          id={switchId}
          checked={descendantsMayOverride}
          disabled={locked}
          onCheckedChange={onDescendantsMayOverrideChange}
        />
      </div>
    </section>
  )
}

export default OrganizationSettingsSection
