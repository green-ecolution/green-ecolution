import type { ReactNode } from 'react'
import { describe, it, expect } from 'vitest'
import { renderHook } from '@testing-library/react'
import { I18nextProvider } from 'react-i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { getI18n } from '@/lib/i18n'
import { useWateringPlanStatusTransitionOptions } from './useDetailsForWateringPlanStatus'

const wrapper = ({ children }: { children: ReactNode }) => (
  <I18nextProvider i18n={getI18n()}>{children}</I18nextProvider>
)

const valuesFor = (status: WateringPlanStatus) => {
  const { result } = renderHook(() => useWateringPlanStatusTransitionOptions(status), { wrapper })
  return result.current.map((option) => option.value)
}

describe('useWateringPlanStatusTransitionOptions', () => {
  it('offers start and cancel from a planned plan', () => {
    expect(valuesFor(WateringPlanStatus.Planned)).toEqual([
      WateringPlanStatus.Planned,
      WateringPlanStatus.Active,
      WateringPlanStatus.Canceled,
    ])
  })

  it('offers revert, finish, fail and cancel from an active plan', () => {
    expect(valuesFor(WateringPlanStatus.Active)).toEqual([
      WateringPlanStatus.Active,
      WateringPlanStatus.Planned,
      WateringPlanStatus.Finished,
      WateringPlanStatus.NotCompleted,
      WateringPlanStatus.Canceled,
    ])
  })

  it.each([
    WateringPlanStatus.Finished,
    WateringPlanStatus.Canceled,
    WateringPlanStatus.NotCompleted,
    WateringPlanStatus.Unknown,
  ])('offers no transition out of the terminal status %s', (status) => {
    expect(valuesFor(status)).toEqual([])
  })

  it('never offers "unknown" as a target', () => {
    for (const status of Object.values(WateringPlanStatus)) {
      expect(valuesFor(status)).not.toContain(WateringPlanStatus.Unknown)
    }
  })

  it('carries the label and colour from the status catalogue', () => {
    const { result } = renderHook(
      () => useWateringPlanStatusTransitionOptions(WateringPlanStatus.Planned),
      { wrapper },
    )
    const [current, next] = result.current
    expect(current.label).toBe('Geplant')
    expect(next.label).toBe('Aktiv')
  })
})
