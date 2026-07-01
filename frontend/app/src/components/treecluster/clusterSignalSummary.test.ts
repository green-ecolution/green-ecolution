import { describe, expect, it } from 'vitest'
import type { Tree } from '@/api/backendApi'
import { clusterSignalSummary } from './clusterSignalSummary'

const treeWith = (num: string, rssiDbm: number | null): Tree =>
  ({
    id: num,
    number: num,
    sensor:
      rssiDbm === null ? null : { latestData: { signal: { rssiDbm, snrDb: 0, gatewayCount: 1 } } },
  }) as unknown as Tree

describe('clusterSignalSummary', () => {
  it('counts levels and finds the weakest sensor', () => {
    const s = clusterSignalSummary([
      treeWith('1', -100),
      treeWith('2', -110),
      treeWith('3', -118),
      treeWith('4', null),
    ])
    expect(s.total).toBe(3)
    expect(s.good).toBe(1)
    expect(s.fair).toBe(1)
    expect(s.weak).toBe(1)
    expect(s.weakest).toEqual({ treeNumber: '3', rssiDbm: -118 })
  })

  it('handles no trees', () => {
    expect(clusterSignalSummary(undefined).total).toBe(0)
  })
})
