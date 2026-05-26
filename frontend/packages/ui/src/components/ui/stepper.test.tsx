import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { Stepper } from './stepper'

const STEPS = [
  { id: 'scan', label: 'Scan' },
  { id: 'gps', label: 'GPS' },
  { id: 'tree', label: 'Baum' },
  { id: 'confirm', label: 'Bestätigen' },
]

describe('Stepper', () => {
  it('marks the current step with aria-current="step"', () => {
    render(<Stepper steps={STEPS} currentStep={2} completedSteps={[1]} />)
    const items = screen.getAllByRole('listitem')
    expect(items[1]).toHaveAttribute('aria-current', 'step')
    expect(items[0]).not.toHaveAttribute('aria-current')
  })

  it('calls onStepClick only for completed steps', async () => {
    const user = userEvent.setup()
    const onStepClick = vi.fn()
    render(
      <Stepper
        steps={STEPS}
        currentStep={3}
        completedSteps={[1, 2]}
        onStepClick={onStepClick}
      />,
    )

    await user.click(screen.getByRole('button', { name: /scan/i }))
    expect(onStepClick).toHaveBeenCalledWith(1)

    const upcoming = screen.getByText('Bestätigen').closest('li')
    expect(upcoming).toHaveAttribute('aria-disabled', 'true')
  })

  it('renders all step labels', () => {
    render(<Stepper steps={STEPS} currentStep={1} completedSteps={[]} />)
    for (const step of STEPS) {
      expect(screen.getByText(step.label)).toBeInTheDocument()
    }
  })
})
