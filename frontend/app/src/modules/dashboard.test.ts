import { describe, expect, it } from 'vitest'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { probeManifest } from '@/test/planningManifest'
import { cardDescriptionKeys } from './dashboard'

describe('cardDescriptionKeys', () => {
  it('keeps the core texts without a module phrasing them', () => {
    expect(cardDescriptionKeys([probeManifest('probe')])).toEqual({
      map: undefined,
      plans: undefined,
    })
  })

  it('takes the irrigation wording while irrigation is active', () => {
    expect(cardDescriptionKeys([irrigationManifest])).toEqual({
      map: 'irrigation:dashboard.mapCardDescription',
      plans: 'irrigation:dashboard.plansCardDescription',
    })
  })
})
