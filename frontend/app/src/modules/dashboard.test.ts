import { describe, expect, it } from 'vitest'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { probeManifest } from '@/test/planningManifest'
import { cardDescriptionKeys, coreCardsShown } from './dashboard'

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

describe('coreCardsShown', () => {
  it('drops the map and plan cards without a module behind them', () => {
    expect(coreCardsShown([])).toEqual({ map: false, plans: false })
  })

  it('shows the plan card for a planning module without a map', () => {
    expect(coreCardsShown([probeManifest('probe')])).toEqual({ map: false, plans: true })
  })

  it('shows both while irrigation is active', () => {
    expect(coreCardsShown([irrigationManifest])).toEqual({ map: true, plans: true })
  })
})
