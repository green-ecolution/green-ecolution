import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import type { ReactNode } from 'react'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import type { StopOutcomeDto } from '@green-ecolution/backend-client'
import type { OutcomeInputProps } from '@/modules/types'
import { probePlanning } from '@/test/planningManifest'
import { planInList } from '@/test/plans'

const finishPlan = vi.fn()
const cancelPlan = vi.fn()
vi.mock('@/hooks/useWateringPlanBoardMutations', () => ({
  useWateringPlanBoardMutations: () => ({
    finishPlan: { mutate: finishPlan },
    cancelPlan: { mutate: cancelPlan },
  }),
}))
vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: ReactNode }) => <a>{children}</a>,
}))

const { default: CompletePlanDialog } = await import('./CompletePlanDialog')

const reported: StopOutcomeDto[] = [{ stop: 'c1', completed: true, amount: 250 }]

const OutcomeInput = ({ plan, onChange }: OutcomeInputProps) => (
  <div>
    <span data-testid="outcome-plan">{plan.id}</span>
    <button type="button" onClick={() => onChange(reported, true)}>
      gültig melden
    </button>
    <button type="button" onClick={() => onChange([], false)}>
      ungültig melden
    </button>
  </div>
)

const plan = planInList({ status: WateringPlanStatus.Active, stops: ['c1', 'hidden'] })
const planning = probePlanning({ OutcomeInput })

describe('CompletePlanDialog', () => {
  beforeEach(() => vi.clearAllMocks())

  const submit = () => screen.getByRole('button', { name: 'Einsatzplan abschließen' })

  it('renders the outcome input of the plan module', async () => {
    render(<CompletePlanDialog plan={plan} planning={planning} onClose={vi.fn()} />)

    expect(await screen.findByTestId('outcome-plan')).toHaveTextContent(plan.id)
  })

  it('passes the outcomes of the input unchanged to finishPlan', async () => {
    const user = userEvent.setup()
    const onClose = vi.fn()
    render(<CompletePlanDialog plan={plan} planning={planning} onClose={onClose} />)

    await user.click(await screen.findByRole('button', { name: 'gültig melden' }))
    await user.click(submit())

    expect(finishPlan).toHaveBeenCalledWith(
      { plan, outcomes: reported },
      expect.objectContaining({ onSuccess: onClose }),
    )
  })

  it('blocks finishing while the input reports invalid outcomes', async () => {
    const user = userEvent.setup()
    render(<CompletePlanDialog plan={plan} planning={planning} onClose={vi.fn()} />)

    expect(submit()).toBeDisabled()
    await user.click(await screen.findByRole('button', { name: 'gültig melden' }))
    expect(submit()).toBeEnabled()
    await user.click(screen.getByRole('button', { name: 'ungültig melden' }))
    expect(submit()).toBeDisabled()
  })

  it('submits a cancellation with note', async () => {
    const user = userEvent.setup()
    render(<CompletePlanDialog plan={plan} planning={planning} onClose={vi.fn()} />)

    await user.click(screen.getByRole('radio', { name: /abgebrochen/i }))
    await user.type(screen.getByLabelText(/grund des abbruchs/i), 'Fahrzeug defekt')
    await user.click(screen.getByRole('button', { name: /einsatzplan abbrechen/i }))

    expect(cancelPlan).toHaveBeenCalledWith(
      { plan, note: 'Fahrzeug defekt' },
      expect.objectContaining({ onSuccess: expect.any(Function) as unknown }),
    )
  })
})
