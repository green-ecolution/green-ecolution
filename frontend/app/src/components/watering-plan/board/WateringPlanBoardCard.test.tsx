import { describe, it, expect, vi } from 'vitest'
import { render, screen } from '@testing-library/react'
import type { ReactNode } from 'react'
import { probePlanning } from '@/test/planningManifest'
import { planInList } from '@/test/plans'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const { default: WateringPlanBoardCard } = await import('./WateringPlanBoardCard')

describe('WateringPlanBoardCard', () => {
  it('counts the stops with the module wording and shows the demand in its capacity', () => {
    const plan = planInList({ stops: ['c1', 'c2', 'c3'], totalDemand: 4200 })
    render(
      <WateringPlanBoardCard
        plan={plan}
        planning={probePlanning({ capacityKind: 'water' })}
        users={[]}
      />,
    )

    expect(screen.getByText('3 Gruppen')).toBeInTheDocument()
    expect(screen.getByTitle('Benötigtes Wasser')).toHaveTextContent('4.200 L')
  })

  it('names the demand after the capacity kind of the module', () => {
    render(
      <WateringPlanBoardCard
        plan={planInList({ totalDemand: 900 })}
        planning={probePlanning({ capacityKind: 'cargo_volume' })}
        users={[]}
      />,
    )

    expect(screen.getByTitle('Benötigtes Ladevolumen')).toHaveTextContent('900 L')
  })
})
