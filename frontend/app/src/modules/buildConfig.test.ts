import { describe, expect, it } from 'vitest'
import { moduleRouteNodes } from './buildConfig'

describe('moduleRouteNodes', () => {
  it('mounts each module routes directory relative to the app routes directory', () => {
    const nodes = moduleRouteNodes('/repo/frontend/app/src/routes', [
      {
        root: '/repo/frontend/modules/irrigation',
        config: { key: 'irrigation', routesDirectory: 'src/routes' },
      },
    ])
    expect(nodes).toEqual([
      { type: 'physical', pathPrefix: '', directory: '.' },
      { type: 'physical', pathPrefix: '', directory: '../../../modules/irrigation/src/routes' },
    ])
  })

  it('keeps only the app routes when no module is registered', () => {
    expect(moduleRouteNodes('/repo/frontend/app/src/routes', [])).toEqual([
      { type: 'physical', pathPrefix: '', directory: '.' },
    ])
  })
})
