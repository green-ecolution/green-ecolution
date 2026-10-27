import { describe, expect, it } from 'vitest'
import {
  completeOutcomes,
  draftForModule,
  keepUnresolvedStops,
  planModule,
  planningFor,
  planTextKey,
} from './planning'
import { probeManifest, probePlanning } from '@/test/planningManifest'

const irrigation = probeManifest('irrigation')
const waste = probeManifest('waste')
const withoutPlanning = { ...probeManifest('sensors-only'), planning: undefined }

describe('planModule', () => {
  it('takes the module named in the search', () => {
    expect(planModule({ module: 'waste' }, [irrigation, waste])?.key).toBe('waste')
  })

  it('falls back to the only active planning module', () => {
    expect(planModule({}, [withoutPlanning, irrigation])?.key).toBe('irrigation')
  })

  it('ignores a search naming an inactive module', () => {
    expect(planModule({ module: 'waste' }, [irrigation])?.key).toBe('irrigation')
  })

  it('starts with the first planning module when several are active', () => {
    expect(planModule({}, [irrigation, waste])?.key).toBe('irrigation')
  })

  it('finds nothing without an active planning module', () => {
    expect(planModule({}, [withoutPlanning])).toBeUndefined()
  })
})

describe('planningFor', () => {
  it('returns the planning slots of the plan module', () => {
    expect(planningFor([irrigation, waste], 'waste')).toBe(waste.planning)
  })

  it('returns nothing for a module that is not active', () => {
    expect(planningFor([irrigation], 'waste')).toBeUndefined()
  })
})

describe('completeOutcomes', () => {
  it('finishes stops the input could not show as done without an amount', () => {
    expect(
      completeOutcomes(['a', 'hidden', 'b'], [{ stop: 'a', completed: true, amount: 50 }]),
    ).toEqual([
      { stop: 'a', completed: true, amount: 50 },
      { stop: 'hidden', completed: true, amount: null },
      { stop: 'b', completed: true, amount: null },
    ])
  })

  it('keeps the reported outcomes untouched', () => {
    const outcomes = [{ stop: 'a', completed: false, amount: null }]
    expect(completeOutcomes(['a'], outcomes)).toEqual(outcomes)
  })
})

describe('keepUnresolvedStops', () => {
  it('keeps stops the picker could not show when it reports a new selection', () => {
    expect(keepUnresolvedStops(['c2', 'c3'], ['c1', 'hidden', 'c2'], ['c1', 'c2'])).toEqual([
      'c2',
      'c3',
      'hidden',
    ])
  })

  it('drops a shown stop the user removed', () => {
    expect(keepUnresolvedStops([], ['c1'], ['c1'])).toEqual([])
  })
})

describe('planTextKey', () => {
  it('takes the wording the plan module gives', () => {
    const planning = probePlanning({ texts: { listDescription: 'irrigation:label' } })
    expect(planTextKey(planning, 'listDescription', 'wateringPlan:list.description')).toBe(
      'irrigation:label',
    )
  })

  it('keeps the core text the module leaves out, and without a module', () => {
    expect(
      planTextKey(probePlanning(), 'createDescription', 'wateringPlan:create.description'),
    ).toBe('wateringPlan:create.description')
    expect(planTextKey(undefined, 'createDescription', 'wateringPlan:create.description')).toBe(
      'wateringPlan:create.description',
    )
  })
})

it('provides a complete planning fixture', () => {
  expect(probePlanning().capacityKind).toBe('water')
})

describe('draftForModule', () => {
  const draft = { module: 'irrigation', stopIds: ['a'] }

  it('restores a draft of the same module', () => {
    expect(draftForModule(draft, 'irrigation')).toBe(draft)
  })

  it('ignores the draft of another module', () => {
    expect(draftForModule(draft, 'waste')).toBeUndefined()
  })

  it('has nothing without a draft', () => {
    expect(draftForModule(undefined, 'waste')).toBeUndefined()
  })
})
