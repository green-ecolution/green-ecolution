import { describe, expect, it } from 'vitest'
import { z } from 'zod'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { interactiveLayerIdsFor, mapSearchSchemaFor } from './map'
import type { ModuleManifest, ModuleMap } from './types'

const probe = (key: string, map: Partial<ModuleMap> = {}): ModuleManifest => ({
  key,
  labelKey: 'common:modules.irrigation',
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
  map: {
    permissions: [],
    searchSchema: z.object({}),
    interactiveLayerIds: [],
    Overview: () => null,
    ...map,
  },
})

describe('mapSearchSchemaFor', () => {
  const withProbe = mapSearchSchemaFor([
    probe('probe', { searchSchema: z.object({ probeId: z.string().optional() }) }),
  ])

  it('accepts the search parameters a manifest contributes', () => {
    expect(withProbe.parse({ probeId: 'p-1' })).toMatchObject({ probeId: 'p-1' })
  })

  it('keeps the core parameters next to the module ones', () => {
    expect(
      withProbe.parse({ probeId: 'p-1', lat: 54.78, lng: 9.43, zoom: 15, sensorId: 's' }),
    ).toEqual({ probeId: 'p-1', lat: 54.78, lng: 9.43, zoom: 15, sensorId: 's' })
  })

  it('does not reject a parameter no schema knows', () => {
    expect(() => withProbe.parse({ probeId: 'p-1', unknown: 'x' })).not.toThrow()
  })

  it('ignores the parameters of a manifest it was not given', () => {
    expect(mapSearchSchemaFor([]).parse({ probeId: 'p-1' })).not.toHaveProperty('probeId')
  })

  it('keeps existing irrigation links with ?cluster= valid', () => {
    const search = mapSearchSchemaFor([irrigationManifest]).parse({
      cluster: 'c-1',
      tree: 't-1',
      wateringStatuses: ['bad'],
      q: 'Nord',
    })
    expect(search).toEqual({ cluster: 'c-1', tree: 't-1', wateringStatuses: ['bad'], q: 'Nord' })
  })
})

describe('interactiveLayerIdsFor', () => {
  it('collects the interactive layers of every given manifest', () => {
    expect(
      interactiveLayerIdsFor([
        probe('alpha', { interactiveLayerIds: ['alpha-points'] }),
        { ...probe('no-map'), map: undefined },
        probe('beta', { interactiveLayerIds: ['beta-fill', 'beta-line'] }),
      ]),
    ).toEqual(['alpha-points', 'beta-fill', 'beta-line'])
  })

  it('names the irrigation tree, cluster and boundary layers', () => {
    expect(interactiveLayerIdsFor([irrigationManifest])).toEqual([
      'gec-tree-points',
      'gec-cluster-markers',
      'gec-boundary-fill',
    ])
  })
})
