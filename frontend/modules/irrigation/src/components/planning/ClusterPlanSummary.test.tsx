import { describe, it, expect } from 'vitest'
import { PlanStatus } from '@green-ecolution/backend-client'
import { render, screen } from '@/test/utils'
import { planResponse } from '@/test/plans'
import ClusterPlanSummary from './ClusterPlanSummary'

describe('ClusterPlanSummary', () => {
  it('sums the water of the completed stops of a finished plan', () => {
    render(
      <ul>
        <ClusterPlanSummary
          plan={planResponse({
            status: PlanStatus.Finished,
            stops: ['c1', 'c2'],
            outcomes: [
              { stop: 'c1', completed: true, amount: 120 },
              { stop: 'c2', completed: true, amount: null },
            ],
          })}
        />
      </ul>,
    )

    expect(screen.getByText('120 Liter')).toBeInTheDocument()
    expect(screen.getByText('bei 2 Bewässerungsgruppen')).toBeInTheDocument()
  })

  it('shows nothing before the plan is finished', () => {
    const { container } = render(
      <ul>
        <ClusterPlanSummary plan={planResponse({ status: PlanStatus.Active })} />
      </ul>,
    )

    expect(container.querySelector('li')).toBeNull()
  })
})
