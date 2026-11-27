import { describe, expect, it } from 'vitest'
import { manifestOf } from '@/test/moduleFixtures'
import { probeManifest } from '@/test/planningManifest'
import { cardDescriptionKeys, coreCardsShown } from './dashboard'
import type { I18nKey, ModuleManifest } from './types'

const irrigationManifest = manifestOf('irrigation')

const withoutMap = (key: string): ModuleManifest => ({ ...probeManifest(key), planning: undefined })

const withMap = (key: string, cardDescriptionKey?: string) =>
  ({ ...withoutMap(key), map: { cardDescriptionKey } }) as unknown as ModuleManifest

const withPlanText = (key: string) =>
  probeManifest(key, { cardDescriptionKey: `${key}:plansText` as I18nKey })

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

  it('uses the text of the only module that draws on the map', () => {
    expect(cardDescriptionKeys([withMap('a', 'a:mapText'), withoutMap('b')]).map).toBe('a:mapText')
  })

  it('keeps the core text when two modules draw on the map', () => {
    expect(
      cardDescriptionKeys([withMap('a', 'a:mapText'), withMap('b', 'b:mapText')]).map,
    ).toBeUndefined()
  })

  it('keeps the core text when only one of two map modules words the card', () => {
    expect(cardDescriptionKeys([withMap('a', 'a:mapText'), withMap('b')]).map).toBeUndefined()
  })

  it('uses the text of the only module that plans', () => {
    expect(cardDescriptionKeys([withPlanText('a'), withoutMap('b')]).plans).toBe('a:plansText')
  })

  it('keeps the core text when two modules plan', () => {
    expect(cardDescriptionKeys([withPlanText('a'), withPlanText('b')]).plans).toBeUndefined()
  })

  it('keeps the core text when only one of two planning modules words the card', () => {
    expect(cardDescriptionKeys([withPlanText('a'), probeManifest('b')]).plans).toBeUndefined()
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
