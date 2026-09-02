import { describe, it, expect, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { getI18n } from '@/lib/i18n'
import { ensureLanguage } from '@/lib/i18n/load'
import { UNRESTRICTED } from './permissions'
import {
  activeActionCount,
  applyLevel,
  applyLevelWithinGrantable,
  clampToGrantable,
  isGrantable,
  levelOf,
  permissionAreasFor,
  toggleAction,
  unknownPermissions,
} from './permissionAreas'

let t: TFunction<'settings'>
let tEn: TFunction<'settings'>
beforeAll(async () => {
  t = getI18n().getFixedT('de', 'settings')
  // The English bundle is loaded lazily on first switch; fixedT('en', …)
  // resolves to raw keys until it's registered.
  await ensureLanguage(getI18n(), 'en')
  tEn = getI18n().getFixedT('en', 'settings')
})

describe('permissionAreasFor', () => {
  it('covers all nine resources with four actions each', () => {
    const areas = permissionAreasFor(t)
    expect(areas).toHaveLength(9)
    for (const area of areas) {
      expect(area.actions).toHaveLength(4)
    }
  })

  it('labels the organization actions without the word Untereinheit', () => {
    const area = permissionAreasFor(t).find((candidate) => candidate.resource === 'organization')
    const text = area!.actions.map((action) => `${action.label} ${action.hint}`).join(' ')
    expect(text).not.toMatch(/Untereinheit/)
    expect(area!.actions.map((action) => action.label)).toEqual([
      'Organisation ansehen',
      'Organisation anlegen',
      'Organisation bearbeiten',
      'Organisation löschen',
    ])
  })

  it('builds the permission string from resource and action', () => {
    const area = permissionAreasFor(t).find((candidate) => candidate.resource === 'tree')
    expect(area!.actions.map((action) => action.permission)).toEqual([
      'tree:read',
      'tree:create',
      'tree:update',
      'tree:delete',
    ])
  })

  // The generic "{resource} {verb}" composition is lossless for 34 of the 36
  // cells; these two pin the label overrides that restore the distinction the
  // generic verb would otherwise flatten (invite vs. create a person, remove
  // access vs. delete a person). watering_plan:create is deliberately not
  // overridden — see the next test.
  it('overrides the two cells the generic verbs would flatten', () => {
    const labelOf = (t: TFunction<'settings'>, resource: string, action: string) =>
      permissionAreasFor(t)
        .find((area) => area.resource === resource)!
        .actions.find((candidate) => candidate.action === action)!.label

    expect(labelOf(t, 'user', 'create')).toBe('Mitarbeitende einladen')
    expect(labelOf(t, 'user', 'delete')).toBe('Mitarbeitende entfernen')

    expect(labelOf(tEn, 'user', 'create')).toBe('Invite members')
    expect(labelOf(tEn, 'user', 'delete')).toBe('Remove members')
  })

  // watering_plan:create composes like the other 33 cells: once the entity is
  // correctly named "Einsatzplan", the extra verb "planen" a prior fix added
  // here said nothing "anlegen" didn't already say.
  it('composes watering_plan:create like the other cells instead of overriding it', () => {
    const labelOf = (t: TFunction<'settings'>, resource: string, action: string) =>
      permissionAreasFor(t)
        .find((area) => area.resource === resource)!
        .actions.find((candidate) => candidate.action === action)!.label

    expect(labelOf(t, 'watering_plan', 'create')).toBe('Einsatzplan anlegen')
    expect(labelOf(tEn, 'watering_plan', 'create')).toBe('Create watering plan')
  })

  it('still composes the other user and watering_plan actions', () => {
    const labelOf = (resource: string, action: string) =>
      permissionAreasFor(t)
        .find((area) => area.resource === resource)!
        .actions.find((candidate) => candidate.action === action)!.label

    expect(labelOf('user', 'read')).toBe('Mitarbeitende ansehen')
    expect(labelOf('user', 'update')).toBe('Mitarbeitende bearbeiten')
    expect(labelOf('watering_plan', 'read')).toBe('Einsatzpläne ansehen')
    expect(labelOf('watering_plan', 'update')).toBe('Einsatzplan bearbeiten')
    expect(labelOf('watering_plan', 'delete')).toBe('Einsatzplan löschen')
  })
})

describe('levelOf', () => {
  it('maps the four presets', () => {
    expect(levelOf('tree', new Set())).toBe('none')
    expect(levelOf('tree', new Set(['tree:read']))).toBe('view')
    expect(levelOf('tree', new Set(['tree:read', 'tree:create', 'tree:update']))).toBe('edit')
    expect(
      levelOf('tree', new Set(['tree:read', 'tree:create', 'tree:update', 'tree:delete'])),
    ).toBe('manage')
  })

  it('reports custom when the set matches no preset', () => {
    expect(levelOf('tree', new Set(['tree:read', 'tree:delete']))).toBe('custom')
    expect(levelOf('tree', new Set(['tree:create']))).toBe('custom')
  })

  it('ignores permissions of other resources', () => {
    expect(levelOf('tree', new Set(['sensor:read', 'sensor:delete', 'tree:read']))).toBe('view')
  })
})

describe('applyLevel', () => {
  it('replaces only the actions of the given resource', () => {
    const before = new Set(['tree:read', 'tree:delete', 'sensor:read'])
    const after = applyLevel(before, 'tree', 'edit')
    expect([...after].sort()).toEqual(['sensor:read', 'tree:create', 'tree:read', 'tree:update'])
  })

  it('clears the resource on level none', () => {
    const after = applyLevel(new Set(['tree:read', 'sensor:read']), 'tree', 'none')
    expect([...after]).toEqual(['sensor:read'])
  })

  it('does not mutate the input', () => {
    const before = new Set(['tree:read'])
    applyLevel(before, 'tree', 'manage')
    expect([...before]).toEqual(['tree:read'])
  })
})

describe('applyLevelWithinGrantable', () => {
  it('does not add a non-grantable action from a preset', () => {
    const after = applyLevelWithinGrantable(new Set(), 'tree', 'manage', new Set(['tree:read']))
    expect([...after].sort()).toEqual(['tree:read'])
  })

  it('preserves a pre-existing non-grantable action across a preset click', () => {
    const before = new Set(['tree:delete'])
    const after = applyLevelWithinGrantable(before, 'tree', 'view', new Set(['tree:read']))
    expect([...after].sort()).toEqual(['tree:delete', 'tree:read'])
  })

  it('equals plain applyLevel when unrestricted', () => {
    const before = new Set(['tree:read', 'tree:delete', 'sensor:read'])
    expect([...applyLevelWithinGrantable(before, 'tree', 'edit', UNRESTRICTED)].sort()).toEqual(
      [...applyLevel(before, 'tree', 'edit')].sort(),
    )
  })

  it('moves grantable actions per preset', () => {
    const grantable = new Set(['tree:read', 'tree:create', 'tree:update'])
    const after = applyLevelWithinGrantable(new Set(['tree:read']), 'tree', 'edit', grantable)
    expect([...after].sort()).toEqual(['tree:create', 'tree:read', 'tree:update'])
    const back = applyLevelWithinGrantable(after, 'tree', 'view', grantable)
    expect([...back].sort()).toEqual(['tree:read'])
  })

  it('does not mutate the input', () => {
    const before = new Set(['tree:read'])
    applyLevelWithinGrantable(before, 'tree', 'manage', UNRESTRICTED)
    expect([...before]).toEqual(['tree:read'])
  })
})

describe('toggleAction', () => {
  it('adds a missing permission and removes a present one', () => {
    expect([...toggleAction(new Set(), 'tree:read')]).toEqual(['tree:read'])
    expect([...toggleAction(new Set(['tree:read']), 'tree:read')]).toEqual([])
  })
})

describe('unknownPermissions', () => {
  it('lists entries outside the known catalog', () => {
    expect(unknownPermissions(new Set(['tree:read', 'report:export', 'x:y']))).toEqual([
      'report:export',
      'x:y',
    ])
  })

  it('survives a round trip through applyLevel', () => {
    const before = new Set(['report:export', 'tree:read'])
    const after = applyLevel(before, 'tree', 'manage')
    expect(after.has('report:export')).toBe(true)
  })
})

describe('isGrantable', () => {
  it('allows everything when unrestricted', () => {
    expect(isGrantable('tree:delete', UNRESTRICTED)).toBe(true)
  })

  it('checks membership otherwise', () => {
    expect(isGrantable('tree:delete', new Set(['tree:read']))).toBe(false)
    expect(isGrantable('tree:read', new Set(['tree:read']))).toBe(true)
  })
})

describe('clampToGrantable', () => {
  it('drops permissions the caller does not hold and reports them', () => {
    const result = clampToGrantable(
      new Set(['tree:read', 'tree:delete', 'role:create']),
      new Set(['tree:read']),
    )
    expect([...result.permissions]).toEqual(['tree:read'])
    expect(result.removed).toEqual(['role:create', 'tree:delete'])
  })

  it('keeps everything when unrestricted', () => {
    const result = clampToGrantable(new Set(['tree:delete']), UNRESTRICTED)
    expect([...result.permissions]).toEqual(['tree:delete'])
    expect(result.removed).toEqual([])
  })

  it('counts the number of removed permissions for the notice', () => {
    const result = clampToGrantable(new Set(['a:b', 'c:d']), new Set())
    expect(result.removed).toHaveLength(2)
  })
})

describe('activeActionCount', () => {
  it('counts only the given resource', () => {
    expect(activeActionCount('tree', new Set(['tree:read', 'tree:update', 'sensor:read']))).toBe(2)
  })
})
