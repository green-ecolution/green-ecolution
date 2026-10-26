import type { ComponentType } from 'react'
import type { QueryClient, QueryKey } from '@tanstack/react-query'
import type { z } from 'zod'
import type {
  CapacityKindDto,
  PlanResponse,
  SensorResponse,
  StopOutcomeDto,
  TargetRefDto,
} from '@green-ecolution/backend-client'
import type { PermissionRequirement } from '@/lib/auth/permissions'
import type { Language } from '@/lib/i18n/languages'

export type I18nKey = string

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
  key: string
  area: string
}

export interface ModuleStatistic {
  labelKey: I18nKey
  to?: string
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
  plan: PlanResponse
  onChange: (outcomes: StopOutcomeDto[], valid: boolean) => void
}

export interface StopPickerProps {
  value: string[]
  onChange: (stops: string[]) => void
  organizationId: string | undefined
  error: string | undefined
}

export interface ModulePlanning {
  capacityKind: CapacityKindDto
  labelKey: I18nKey
  stopCountKey: I18nKey
  invalidateOnStatusChange: readonly QueryKey[]
  SuggestionsColumn: ComponentType
  StopList: ComponentType<{ plan: PlanResponse }>
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
  dashboardTiles?: readonly ComponentType[]
  statistics?: Readonly<Record<string, ModuleStatistic>>
  map?: ModuleMap
  planning?: ModulePlanning
  targets?: readonly ModuleTarget[]
  settingsSection?: ModuleSettingsSection
}
