import { QueryClient } from '@tanstack/react-query'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { queryRoots } from '@/api/queries'
import { irrigationRoots } from './api/roots'
import { irrigationManifest } from './manifest'
import { treeSensorTarget } from './sensorTargets'

const { listClusterSensorIds } = vi.hoisted(() => ({ listClusterSensorIds: vi.fn() }))

vi.mock('./api/client', async (importOriginal) => ({
  ...(await importOriginal<typeof import('./api/client')>()),
  clusterApi: { listClusterSensorIds },
}))

describe('treeSensorTarget', () => {
  beforeEach(() => {
    listClusterSensorIds.mockReset()
  })

  it('is the irrigation module’s only sensor target', () => {
    expect(irrigationManifest.targets).toEqual([treeSensorTarget])
    expect(treeSensorTarget.kind).toBe('tree')
    expect(treeSensorTarget.invalidate).toEqual(irrigationRoots.tree)
  })

  it('keeps the cluster filter on the search param existing links use', () => {
    expect(treeSensorTarget.listFilter?.param).toBe('clusterIds')
  })

  it('resolves the cluster filter to the union of the clusters’ sensors', async () => {
    listClusterSensorIds.mockImplementation(({ clusterId }: { clusterId: string }) =>
      Promise.resolve(clusterId === 'c1' ? ['s1', 's2'] : ['s2', 's3']),
    )
    const options = treeSensorTarget.listFilter!.sensorIds(['c2', 'c1'])

    const ids = await new QueryClient().fetchQuery(options)

    expect(ids.sort()).toEqual(['s1', 's2', 's3'])
  })

  it('sits under the sensor root, so a sensor change refreshes the filter', () => {
    const { queryKey } = treeSensorTarget.listFilter!.sensorIds(['c1'])
    expect(queryRoots.sensor.some((root) => root[0] === queryKey[0])).toBe(true)
  })
})
