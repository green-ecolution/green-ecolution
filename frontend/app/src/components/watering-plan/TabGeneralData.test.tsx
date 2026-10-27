import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { probePlanning } from '@/test/planningManifest'
import { planResponse } from '@/test/plans'

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
  planResponse({ status: WateringPlanStatus.Planned, distance: 1000, userIds })
const planning = probePlanning()

const assignedValue = () =>
  screen.getByText('Eingeteilte Mitarbeitende').nextElementSibling?.textContent

describe('TabGeneralData', () => {
  it('lists only the users assigned to the plan', () => {
    render(<TabGeneralData wateringPlan={plan(['u1'])} planning={planning} />)

    expect(assignedValue()).toBe('Max Mustermann')
  })

  it('shows a placeholder when nobody is assigned', () => {
    render(<TabGeneralData wateringPlan={plan([])} planning={planning} />)

    expect(assignedValue()).toBe('Keine Angabe')
  })

  it('says when only the trailer carries water', () => {
    const trailerOnly = planResponse({
      trailer: { numberPlate: 'FL-TR 1' } as never,
      transporterCapacityUsed: false,
    })
    render(<TabGeneralData wateringPlan={trailerOnly} planning={planning} />)

    expect(screen.getByText('Wassertank des Fahrzeugs').nextElementSibling?.textContent).toBe(
      'Bleibt leer, nur der Anhänger führt Wasser',
    )
  })

  it('words the vehicle row as cargo space for a cargo volume plan', () => {
    const trailerOnly = planResponse({
      trailer: { numberPlate: 'FL-TR 1' } as never,
      transporterCapacityUsed: false,
    })
    render(
      <TabGeneralData
        wateringPlan={trailerOnly}
        planning={probePlanning({ capacityKind: 'cargo_volume' })}
      />,
    )

    expect(screen.getByText('Laderaum des Fahrzeugs').nextElementSibling?.textContent).toBe(
      'Bleibt leer, nur der Anhänger lädt',
    )
  })

  it('leaves out the tank row for a plan without trailer', () => {
    render(<TabGeneralData wateringPlan={plan([])} planning={planning} />)

    expect(screen.queryByText('Wassertank des Fahrzeugs')).not.toBeInTheDocument()
  })

  it('renders the summary of the plan module and counts its stops in its words', async () => {
    render(
      <TabGeneralData
        wateringPlan={planResponse({ stops: ['c1', 'c2'], totalDemand: 160 })}
        planning={planning}
      />,
    )

    expect(await screen.findByTestId('plan-summary')).toBeInTheDocument()
    expect(screen.getByText('Anzahl der Bewässerungsgruppen').nextElementSibling?.textContent).toBe(
      '2 Gruppen',
    )
    expect(screen.getByText('Benötigtes Wasser').nextElementSibling?.textContent).toBe('160 Liter')
  })
})
