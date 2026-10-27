import type { ComponentType } from 'react'
import { moduleManifests } from '@/modules/registry'
import type { ModuleSettingsDraft, ModuleSettingsSectionProps } from '@/modules/types'

export interface ModuleSettingsEntry {
  key: string
  /** Active in this organization; only then does the section show and take part in saving. */
  active: boolean
  draft: ModuleSettingsDraft
  Section: ComponentType<ModuleSettingsSectionProps>
}

const registeredSections = moduleManifests.flatMap(({ key, settingsSection }) =>
  settingsSection ? [{ key, settingsSection }] : [],
)

// eslint-disable-next-line react-x/no-unnecessary-use-prefix -- it calls each section's `useDraft`, which the rule cannot see through the manifest
export const useModuleSettingsDrafts = (
  organizationId: string,
  isActive: (key: string) => boolean,
): ModuleSettingsEntry[] =>
  // The registered list is fixed for the build, never the active subset, so
  // every render calls the same hooks in the same order.
  registeredSections.map(({ key, settingsSection }) => {
    const active = isActive(key)
    return {
      key,
      active,
      draft: settingsSection.useDraft(organizationId, { enabled: active }),
      Section: settingsSection.Section,
    }
  })
