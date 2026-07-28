import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from './permissions'
import {
  activeActionCount,
  applyLevel,
  clampToGrantable,
  isGrantable,
  levelOf,
  PERMISSION_AREAS,
  toggleAction,
  unknownPermissions,
} from './permissionAreas'

describe('PERMISSION_AREAS', () => {
  it('covers all nine resources with four actions each', () => {
    expect(PERMISSION_AREAS).toHaveLength(9)
    for (const area of PERMISSION_AREAS) {
      expect(area.actions).toHaveLength(4)
    }
  })

  it('labels the organization actions without the word Untereinheit', () => {
    const area = PERMISSION_AREAS.find((candidate) => candidate.resource === 'organization')
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
    const area = PERMISSION_AREAS.find((candidate) => candidate.resource === 'tree')
    expect(area!.actions.map((action) => action.permission)).toEqual([
      'tree:read',
      'tree:create',
      'tree:update',
      'tree:delete',
    ])
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
