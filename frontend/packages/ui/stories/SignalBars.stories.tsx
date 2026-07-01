import type { Meta, StoryObj } from '@storybook/react-vite'
import { SignalBars } from '../src/components/ui/signal-bars'

const meta: Meta<typeof SignalBars> = {
  title: 'UI/SignalBars',
  component: SignalBars,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component:
          'Vier aufsteigende Balken für Signalstärke (0–4). Die Balken nutzen `bg-current`; die Farbe wird über eine `text-*`-Klasse gesetzt. Genutzt für GPS-Genauigkeit und LoRaWAN-Signalstärke.',
      },
    },
  },
}
export default meta

type Story = StoryObj<typeof SignalBars>

export const Gut: Story = {
  render: () => (
    <span className="text-green-dark">
      <SignalBars filled={4} className="h-8" />
    </span>
  ),
}

export const Ausreichend: Story = {
  render: () => (
    <span className="text-yellow-900">
      <SignalBars filled={2} className="h-8" />
    </span>
  ),
}

export const Schwach: Story = {
  render: () => (
    <span className="text-red">
      <SignalBars filled={1} className="h-8" />
    </span>
  ),
}
