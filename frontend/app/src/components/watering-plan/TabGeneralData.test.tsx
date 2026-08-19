import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlan } from '@/api/backendApi'

const users = [
  { id: 'u1', firstName: 'Max', lastName: 'Mustermann' },
  { id: 'u2', firstName: 'Erika', lastName: 'Musterfrau' },
]

vi.mock('@tanstack/react-query', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@tanstack/react-query')>()
  return {
    ...actual,
    useSuspenseQuery: () => ({ data: { data: users } }),
    useQuery: () => ({ data: [] }),
  }
})

const { default: TabGeneralData } = await import('./TabGeneralData')

const plan = (userIds: string[]) =>
  ({
    id: 'p1',
    date: '2026-03-01T00:00:00Z',
    updatedAt: '2026-03-01T00:00:00Z',
    status: WateringPlanStatus.Planned,
    treeclusters: [],
    evaluation: [],
    distance: 1000,
    userIds,
  }) as unknown as WateringPlan

const assignedValue = () =>
  screen.getByText('Eingeteilte Mitarbeitende').nextElementSibling?.textContent

describe('TabGeneralData', () => {
  it('lists only the users assigned to the plan', () => {
    render(<TabGeneralData wateringPlan={plan(['u1'])} />)

    expect(assignedValue()).toBe('Max Mustermann')
  })

  it('shows a placeholder when nobody is assigned', () => {
    render(<TabGeneralData wateringPlan={plan([])} />)

    expect(assignedValue()).toBe('Keine Angabe')
  })
})
