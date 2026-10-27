import type { ComponentType, ReactNode } from 'react'
import type { QueryClient, QueryKey, UseQueryOptions } from '@tanstack/react-query'
import type { z } from 'zod'
import type {
  CapacityKindDto,
  PlanInListResponse,
  PlanResponse,
  SensorResponse,
  StopOutcomeDto,
  TargetRefDto,
} from '@green-ecolution/backend-client'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import type { PermissionRequirement, Resource } from '@/lib/auth/permissions'
import type { AreaGroup } from '@/lib/auth/permissionAreas'
import type { ParseKeys } from 'i18next'
import type { AllNamespaces, Language } from '@/lib/i18n/languages'

export type I18nKey = ParseKeys<AllNamespaces>

export interface ModuleI18n {
  namespaces: readonly string[]
  load: (lng: Language) => Promise<Record<string, object>>
}

export type NavSectionKey = 'greenspaces' | 'planning' | 'analysis'

export interface ModuleNavLink {
  key: string
  to: string
  labelKey: I18nKey
  icon: ComponentType<{ className?: string }>
}

export interface ModuleNav {
  links: readonly (ModuleNavLink & { section: NavSectionKey })[]
}

export interface ModuleResource {
  key: Resource
  area: AreaGroup
  /** Prefix of the resource's `label`, `pluralNoun`, `singular`, `description` and `hint.<action>` keys. */
  textsKey: string
}

export interface ModuleStatistic {
  icon: ComponentType<{ className?: string }>
  labelKey: I18nKey
  chartLabelKey?: I18nKey
  subtitleKey?: I18nKey
  to?: string
  /** Features the statistic as the large card; `companionKind` names the count shown below it. */
  hero?: { companionKind: string; companionKey: I18nKey; linkLabelKey: I18nKey }
}

export interface DashboardTileProps {
  variant: 'light' | 'dark'
}

export interface ModuleMap {
  permissions: PermissionRequirement
  /** Replaces the core description of the dashboard's map card while the module is active. */
  cardDescriptionKey?: I18nKey
  searchSchema: z.ZodObject
  interactiveLayerIds: readonly string[]
  prefetch?: (queryClient: QueryClient) => void
  Overview: ComponentType
  Toolbar?: ComponentType
  Overlay?: ComponentType
}

export interface OutcomeInputProps {
  plan: PlanInListResponse
  /** Outcomes of the stops the input can show; the core completes the rest. */
  onChange: (outcomes: StopOutcomeDto[], valid: boolean) => void
}

/** Where the plan form stands, so a picker that leaves the page can bring the user back. */
export interface PlanFormContext {
  type: 'create' | 'update'
  planId: string | undefined
  transporterId: string | undefined
  trailerId: string | undefined
  saveDraft: () => void
}

export interface StopPickerProps {
  /**
   * Every stop of the plan, including ones the picker cannot resolve. A picker
   * that shows fewer reports through `keepUnresolvedStops`, so those survive.
   */
  value: string[]
  onChange: (stops: string[]) => void
  organizationId: string | undefined
  error: string | undefined
  form: PlanFormContext
}

/** Plan page texts a module words for its own stops; a missing one keeps the core text. */
export interface ModulePlanTexts {
  listDescription?: I18nKey
  listDescriptionCanModify?: I18nKey
  createDescription?: I18nKey
  updateDescription?: I18nKey
  statusUpdateDescription?: I18nKey
  routeErrorAlert?: I18nKey
  routePreviewAriaLabel?: I18nKey
}

export interface ModulePlanning {
  planResource: Resource
  /** Replaces the core description of the dashboard's plans card while the module is active. */
  cardDescriptionKey?: I18nKey
  /** Names every plan on the system info page while it is the only active planning module. */
  plansLabelKey?: I18nKey
  texts?: ModulePlanTexts
  capacityKind: CapacityKindDto
  labelKey: I18nKey
  stopsLabelKey: I18nKey
  stopCountLabelKey: I18nKey
  stopCountKey: I18nKey
  stopIcon: ComponentType<{ className?: string }>
  invalidateOnStatusChange: readonly QueryKey[]
  /** Pages the `StopPicker` may send the user to; the dirty plan form lets them go there. */
  stopPickerPaths: readonly string[]
  /** Warms the board's module parts while its loader runs. */
  prefetch?: (queryClient: QueryClient) => void
  SuggestionsColumn: ComponentType
  StopList: ComponentType<{ plan: PlanResponse }>
  /** Renders `<li>` items into the status card grid of the plan's general tab. */
  PlanSummary: ComponentType<{ plan: PlanResponse }>
  StopPicker: ComponentType<StopPickerProps>
  RouteOverlay: ComponentType<{ plan: PlanResponse }>
  OutcomeInput: ComponentType<OutcomeInputProps>
}

export interface SelectedTarget {
  target: TargetRefDto
  label: string
  detail?: string
}

export interface TargetPickerProps {
  current: TargetRefDto | undefined
  onSelect: (selected: SelectedTarget) => void
}

/** The wizard freezes the position at the scan, so a picker can suggest targets near it. */
export interface TargetWizardStepProps extends TargetPickerProps {
  position: GeolocationFix | null
  geolocationStatus: GeolocationStatus
  geolocationError: string | null
  onRelocate: () => void
}

export type TargetAssignMode = 'activate' | 'reassign'

/** The dialog only picks; the core sends the request and reports back through `pending` and `error`. */
export interface TargetAssignDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  mode: TargetAssignMode
  sensor: SensorResponse
  pending: boolean
  error: string | null
  onConfirm: (selected: SelectedTarget) => void
}

/** Sensor texts a module words for its target kind; a missing one keeps the neutral core text. */
export interface ModuleTargetTexts {
  activateAction?: I18nKey
  reassignAction?: I18nKey
  removeLinkAction?: I18nKey
  removeConfirmTitle?: I18nKey
  /** Receives `{status}`, the state the sensor falls back to. */
  removeConfirmDescription?: I18nKey
  activateSuccess?: I18nKey
  reassignSuccess?: I18nKey
  activateConflict?: I18nKey
  reassignNotFound?: I18nKey
  reassignConflict?: I18nKey
  reassignFailed?: I18nKey
  scanDescription?: I18nKey
  alreadyLinkedDescription?: I18nKey
  listSearchPlaceholder?: I18nKey
}

export interface ModuleTarget {
  kind: string
  labelKey: I18nKey
  texts?: ModuleTargetTexts
  /** Key prefixes a link change touches besides the sensor's own. */
  invalidate: readonly QueryKey[]
  AssignDialog: ComponentType<TargetAssignDialogProps>
  WizardStep: ComponentType<TargetWizardStepProps>
  /** `children` is the core's coordinates panel; it belongs between the map and the rows. */
  Placement: ComponentType<{ sensor: SensorResponse; children?: ReactNode }>
  listFilter?: ModuleSensorListFilter
}

export interface SensorListFilterProps {
  value: string[]
  onChange: (value: string[]) => void
}

export interface ModuleSensorListFilter {
  /** Search param of the sensor list; shared links carry it, so it must stay stable. */
  param: string
  Control: ComponentType<SensorListFilterProps>
  /** Renders a `FilterChip` per selected value into the list's active-filter row. */
  Chips: ComponentType<SensorListFilterProps>
  /** The sensors the selection narrows the list to; a query so the route loader can prefetch it. */
  sensorIds: (value: string[]) => UseQueryOptions<string[], Error, string[], QueryKey>
}

export interface ModuleSettingsDraft {
  loading: boolean
  /** The page reports it like a failed load of the core settings. */
  loadError: unknown
  /** Only something valid to send counts; an invalid edit shows through `valid`. */
  dirty: boolean
  valid: boolean
  reset: () => void
  /** Rejects on failure; the page reports one outcome for everything it saved. */
  save: () => Promise<void>
}

export interface ModuleSettingsSectionProps<D extends ModuleSettingsDraft = ModuleSettingsDraft> {
  draft: D
  /** Nothing to edit: an ancestor froze the subtree or the viewer lacks `setting:update`. */
  locked: boolean
  /** An ancestor froze the subtree, so an own value lies dormant. */
  enforced: boolean
  /** Resolves source organizations from the list the page already loaded. */
  organizationName: (organizationId: string | null | undefined) => string | null
}

/** Build it with `defineSettingsSection`, which ties `Section` to the draft `useDraft` returns. */
export interface ModuleSettingsSection<D extends ModuleSettingsDraft = ModuleSettingsDraft> {
  /**
   * Called on every render of the organization page, also where the module is
   * off (`enabled: false`), so it must not fetch then.
   */
  useDraft: (organizationId: string, options: { enabled: boolean }) => D
  Section: ComponentType<ModuleSettingsSectionProps<D>>
}

export interface ModuleManifest {
  key: string
  labelKey: I18nKey
  i18n: ModuleI18n
  nav?: ModuleNav
  routePermissions?: Readonly<Record<string, PermissionRequirement>>
  resources?: readonly ModuleResource[]
  dashboardTiles?: readonly ComponentType<DashboardTileProps>[]
  statistics?: Readonly<Record<string, ModuleStatistic>>
  map?: ModuleMap
  planning?: ModulePlanning
  targets?: readonly ModuleTarget[]
  settingsSection?: ModuleSettingsSection
  /** Rendered on every sensor's detail page; the module decides itself whether it has something to show. */
  sensorInsights?: ComponentType<{ sensor: SensorResponse }>
}
