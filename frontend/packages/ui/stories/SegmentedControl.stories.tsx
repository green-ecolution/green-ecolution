import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { SegmentedControl } from '../src/components/ui/segmented-control'

const levels = [
  { value: 'none', label: 'Kein' },
  { value: 'view', label: 'Ansehen' },
  { value: 'edit', label: 'Bearbeiten' },
  { value: 'manage', label: 'Verwalten' },
]

const meta: Meta<typeof SegmentedControl> = {
  title: 'UI/SegmentedControl',
  component: SegmentedControl,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Zugriffsstufe: Story = {
  render: () => {
    const [value, setValue] = useState('edit')
    return (
      <SegmentedControl
        options={levels}
        value={value}
        onChange={setValue}
        ariaLabel="Zugriffsstufe für Bäume"
      />
    )
  },
}

export const OhneAuswahl: Story = {
  name: 'Ohne Auswahl (Individuell)',
  render: () => (
    <div className="flex items-center gap-3">
      <SegmentedControl
        options={levels}
        value={null}
        onChange={() => {}}
        ariaLabel="Zugriffsstufe für Sensoren"
      />
      <span className="text-xs text-dark-600">Individuell</span>
    </div>
  ),
}

export const AufDunklerFlaeche: Story = {
  name: 'Auf dunkler Fläche',
  render: () => {
    const [value, setValue] = useState('de')
    return (
      <div className="rounded-lg bg-dark p-6">
        <SegmentedControl
          options={[
            { value: 'de', label: 'Deutsch' },
            { value: 'en', label: 'Englisch' },
          ]}
          value={value}
          onChange={setValue}
          ariaLabel="Sprache"
          size="sm"
          tone="dark"
        />
      </div>
    )
  },
}
