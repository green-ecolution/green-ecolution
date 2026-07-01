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
          'Vier aufsteigende Balken für Signalstärke (0–4). Die Balken nutzen `bg-current`; die Farbe wird über eine `text-*`-Klasse gesetzt. Über `size` (`sm`/`md`/`lg`) lässt sich die Dicke steuern. Genutzt für GPS-Genauigkeit (`sm`) und LoRaWAN-Signalstärke (`lg`).',
      },
    },
  },
}
export default meta

type Story = StoryObj<typeof SignalBars>

export const Gut: Story = {
  render: () => (
    <span className="text-green-dark">
      <SignalBars filled={4} size="lg" />
    </span>
  ),
}

export const Ausreichend: Story = {
  render: () => (
    <span className="text-yellow-900">
      <SignalBars filled={2} size="lg" />
    </span>
  ),
}

export const Schwach: Story = {
  render: () => (
    <span className="text-red">
      <SignalBars filled={1} size="lg" />
    </span>
  ),
}

export const Groessen: Story = {
  render: () => (
    <div className="flex items-end gap-6 text-green-dark">
      <SignalBars filled={3} size="sm" />
      <SignalBars filled={3} size="md" />
      <SignalBars filled={3} size="lg" />
    </div>
  ),
}
