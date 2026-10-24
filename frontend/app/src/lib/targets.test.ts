import { describe, expect, it } from 'vitest'
import { isTreeTarget, treeTarget } from './targets'

describe('targets', () => {
  it('builds an irrigation tree target', () => {
    expect(treeTarget('t1')).toEqual({ module: 'irrigation', kind: 'tree', id: 't1' })
  })

  it('recognises only irrigation trees', () => {
    expect(isTreeTarget(treeTarget('t1'))).toBe(true)
    expect(isTreeTarget({ module: 'waste', kind: 'bin', id: 'b' })).toBe(false)
    expect(isTreeTarget(null)).toBe(false)
  })
})
