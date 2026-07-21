import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { TimeRangeToggle } from '../src/components/ui/time-range-toggle'

const meta: Meta<typeof TimeRangeToggle> = {
  title: 'UI/TimeRangeToggle',
  component: TimeRangeToggle,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component:
          'Zeitraum-Umschalter für Verlaufs-Diagramme, z. B. Bodenfeuchte einer Bewässerungsgruppe.',
      },
    },
  },
}
export default meta

type Story = StoryObj<typeof TimeRangeToggle>

export const Default: Story = {
  render: () => {
    const [value, setValue] = useState<'24h' | '7d' | '30d'>('24h')
    return (
      <TimeRangeToggle
        options={[
          { value: '24h', label: '24 h' },
          { value: '7d', label: '7 Tage' },
          { value: '30d', label: '30 Tage' },
        ]}
        value={value}
        onChange={setValue}
      />
    )
  },
}
