import type { ReactNode } from 'react'
import { describe, it, expect } from 'vitest'
import { renderHook } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import { PlanStatus } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { usePlanStatusTransitionOptions } from './useDetailsForPlanStatus'

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nextProvider i18n={getI18n()}>{children}</I18nextProvider>
)

const valuesFor = (status: PlanStatus) => {
  const { result } = renderHook(() => usePlanStatusTransitionOptions(status), { wrapper })
  return result.current.map((option) => option.value)
}

describe('usePlanStatusTransitionOptions', () => {
  it('offers start and cancel from a planned plan', () => {
    expect(valuesFor(PlanStatus.Planned)).toEqual([
      PlanStatus.Planned,
      PlanStatus.Active,
      PlanStatus.Canceled,
    ])
  })

  it('offers revert, finish, fail and cancel from an active plan', () => {
    expect(valuesFor(PlanStatus.Active)).toEqual([
      PlanStatus.Active,
      PlanStatus.Planned,
      PlanStatus.Finished,
      PlanStatus.NotCompleted,
      PlanStatus.Canceled,
    ])
  })

  it.each([PlanStatus.Finished, PlanStatus.Canceled, PlanStatus.NotCompleted])(
    'offers no transition out of the terminal status %s',
    (status) => {
      expect(valuesFor(status)).toEqual([])
    },
  )

  it('carries the label and colour from the status catalogue', () => {
    const { result } = renderHook(() => usePlanStatusTransitionOptions(PlanStatus.Planned), {
      wrapper,
    })
    const [current, next] = result.current
    expect(current.label).toBe('Geplant')
    expect(next.label).toBe('Aktiv')
  })
})
