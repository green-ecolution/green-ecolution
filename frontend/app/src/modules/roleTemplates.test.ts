import { describe, expect, it } from 'vitest'
import { roleTemplateOf } from './roleTemplates'
import type { ModuleManifest } from './types'

const Icon = () => null

const withTemplate = (module: string, key: string) =>
  ({
    key: module,
    order: 0,
    roleTemplates: [
      { key, nameKey: `${module}:roles.${key}` as ModuleManifest['labelKey'], icon: Icon },
    ],
  }) as unknown as ModuleManifest

describe('role templates', () => {
  it('finds a core template', () => {
    expect(roleTemplateOf('observer', [])?.nameKey).toBe('settings:roles.templateName.observer')
  })

  it('finds a template a module declares', () => {
    const found = roleTemplateOf('probe_role', [withTemplate('probe', 'probe_role')])

    expect(found?.nameKey).toBe('probe:roles.probe_role')
    expect(found?.icon).toBe(Icon)
  })

  it('knows nothing about a key nobody declares', () => {
    expect(roleTemplateOf('gone_role', [withTemplate('probe', 'probe_role')])).toBeUndefined()
    expect(roleTemplateOf(null, [])).toBeUndefined()
    expect(roleTemplateOf(undefined, [])).toBeUndefined()
  })
})
