import type { ComponentType } from 'react'
import type { QueryClient, QueryKey } from '@tanstack/react-query'
import type { z } from 'zod'
import type {
  CapacityKindDto,
  PlanInListResponse,
  PlanResponse,
  SensorResponse,
  StopOutcomeDto,
  TargetRefDto,
} from '@green-ecolution/backend-client'
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

export interface ModulePlanning {
  planResource: Resource
  capacityKind: CapacityKindDto
  labelKey: I18nKey
  stopsLabelKey: I18nKey
  stopCountLabelKey: I18nKey
  stopCountKey: I18nKey
  stopIcon: ComponentType<{ className?: string }>
  invalidateOnStatusChange: readonly QueryKey[]
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

export interface ModuleTarget {
  kind: string
  labelKey: I18nKey
  invalidate: readonly QueryKey[]
  AssignDialog: ComponentType<
    TargetPickerProps & { open: boolean; onOpenChange: (open: boolean) => void }
  >
  WizardStep: ComponentType<TargetPickerProps>
  Placement: ComponentType<{ sensor: SensorResponse }>
  SensorInsights?: ComponentType<{ sensor: SensorResponse }>
  listFilter?: ModuleSensorListFilter
}

export interface ModuleSensorListFilter {
  param: string
  Control: ComponentType<{ value: string[]; onChange: (value: string[]) => void }>
  useSensorIds: (value: string[]) => string[] | undefined
}

export interface ModuleSettingsDraft {
  loading: boolean
  dirty: boolean
  valid: boolean
  reset: () => void
  save: () => Promise<void>
}

export interface ModuleSettingsSection {
  useDraft: (organizationId: string) => ModuleSettingsDraft
  Section: ComponentType<{ organizationId: string; locked: boolean; editing: boolean }>
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
}
