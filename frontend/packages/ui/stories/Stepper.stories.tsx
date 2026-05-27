import type { Meta, StoryObj } from '@storybook/react-vite'
import { Stepper } from '../src/components/ui/stepper'

const STEPS = [
  { id: 'scan', label: 'QR-Scan' },
  { id: 'gps', label: 'GPS' },
  { id: 'tree', label: 'Baum' },
  { id: 'confirm', label: 'Bestätigen' },
]

const meta: Meta<typeof Stepper> = {
  title: 'UI/Stepper',
  component: Stepper,
  tags: ['autodocs'],
  args: { steps: STEPS },
}

export default meta
type Story = StoryObj<typeof meta>

export const Start: Story = { args: { currentStep: 1, completedSteps: [] } }
export const Midway: Story = { args: { currentStep: 3, completedSteps: [1, 2] } }
export const NearEnd: Story = { args: { currentStep: 4, completedSteps: [1, 2, 3] } }
export const AllDone: Story = { args: { currentStep: 4, completedSteps: [1, 2, 3, 4] } }
