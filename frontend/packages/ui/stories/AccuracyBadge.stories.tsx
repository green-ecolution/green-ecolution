import type { Meta, StoryObj } from '@storybook/react-vite'
import { AccuracyBadge } from '../src/components/ui/accuracy-badge'

const meta: Meta<typeof AccuracyBadge> = {
  title: 'UI/AccuracyBadge',
  component: AccuracyBadge,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component:
          'Pill-style badge that visualizes a GPS accuracy reading (in meters) as one of four quality levels — `excellent` (< 10 m), `good` (10–30 m), `fair` (30–75 m), `poor` (> 75 m). A 4-bar signal-strength icon mirrors common cellular UX so the level is readable at a glance, and the leading bar gently pulses on the two best levels to communicate live tracking. Pass `null`/`undefined` to render the `searching` state.',
      },
    },
  },
  argTypes: {
    accuracyMeters: {
      control: { type: 'number', min: 0, max: 200, step: 0.5 },
      description: 'Accuracy in meters (e.g. `GeolocationCoordinates.accuracy`).',
    },
    hideValue: {
      control: 'boolean',
      description: 'Hide the numeric value — show only level label + bars.',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Excellent: Story = {
  args: { accuracyMeters: 5 },
}

export const Good: Story = {
  args: { accuracyMeters: 18 },
}

export const Fair: Story = {
  args: { accuracyMeters: 50 },
}

export const Poor: Story = {
  args: { accuracyMeters: 120 },
}

export const Searching: Story = {
  args: { accuracyMeters: null },
}

export const ValueHidden: Story = {
  args: { accuracyMeters: 12, hideValue: true },
}

export const AllLevels: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-2">
      <AccuracyBadge accuracyMeters={4.2} />
      <AccuracyBadge accuracyMeters={18} />
      <AccuracyBadge accuracyMeters={50} />
      <AccuracyBadge accuracyMeters={120} />
      <AccuracyBadge accuracyMeters={null} />
    </div>
  ),
}

export const Interactive: Story = {
  args: { accuracyMeters: 12 },
}

export const InContext: Story = {
  render: () => (
    <div className="flex flex-col gap-3 max-w-sm">
      <div className="flex items-baseline justify-between gap-3 rounded-lg border border-dark-100 bg-white p-3">
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">Aktueller Standort</span>
          <span className="font-mono text-sm">54.79228, 9.43581</span>
        </div>
        <AccuracyBadge accuracyMeters={6.4} />
      </div>
      <div className="flex items-baseline justify-between gap-3 rounded-lg border border-dark-100 bg-white p-3">
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">Aktueller Standort</span>
          <span className="font-mono text-sm">54.79231, 9.43577</span>
        </div>
        <AccuracyBadge accuracyMeters={42} />
      </div>
      <div className="flex items-baseline justify-between gap-3 rounded-lg border border-dark-100 bg-white p-3">
        <div className="flex flex-col">
          <span className="text-xs text-muted-foreground">Aktueller Standort</span>
          <span className="font-mono text-sm">— —</span>
        </div>
        <AccuracyBadge accuracyMeters={null} />
      </div>
    </div>
  ),
}
