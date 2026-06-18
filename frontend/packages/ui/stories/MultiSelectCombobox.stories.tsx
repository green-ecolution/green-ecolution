import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { MultiSelectCombobox } from '../src/components/ui/multi-select-combobox'

const meta: Meta<typeof MultiSelectCombobox> = {
  title: 'UI/MultiSelectCombobox',
  component: MultiSelectCombobox,
  tags: ['autodocs'],
  parameters: {
    docs: {
      description: {
        component:
          'Durchsuchbare Mehrfachauswahl auf Basis von Popover + Command. Optionen lassen sich gruppieren, die Auswahl bleibt beim Anklicken offen, und der Trigger zeigt entweder die gewählten Labels oder „N ausgewählt". Wird u. a. für die Filterung von Bewässerungsgruppen (Zustand, Bezirk, Bodenart) genutzt.',
      },
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

const soilOptions = [
  { value: 'Ss', label: 'Ss – Reinsand', group: 'Sande' },
  { value: 'Sl2', label: 'Sl2 – schwach lehmiger Sand', group: 'Sande' },
  { value: 'Uu', label: 'Uu – reiner Schluff', group: 'Schluffe' },
  { value: 'Us', label: 'Us – sandiger Schluff', group: 'Schluffe' },
  { value: 'Lt3', label: 'Lt3 – stark toniger Lehm', group: 'Lehme' },
  { value: 'Tt', label: 'Tt – reiner Ton', group: 'Tone' },
]

const statusOptions = [
  { value: 'bad', label: 'Sehr trocken' },
  { value: 'moderate', label: 'Leicht trocken' },
  { value: 'good', label: 'In Ordnung' },
  { value: 'just watered', label: 'Soeben bewässert' },
  { value: 'unknown', label: 'Unbekannt' },
]

export const Bodenart: Story = {
  render: () => {
    const [value, setValue] = useState<string[]>(['Uu'])
    return (
      <div className="w-72">
        <MultiSelectCombobox
          options={soilOptions}
          value={value}
          onChange={setValue}
          placeholder="Alle Bodenarten"
          searchPlaceholder="Bodenart suchen"
        />
      </div>
    )
  },
}

export const Zustand: Story = {
  render: () => {
    const [value, setValue] = useState<string[]>(['bad', 'moderate'])
    return (
      <div className="w-72">
        <MultiSelectCombobox
          options={statusOptions}
          value={value}
          onChange={setValue}
          searchable={false}
          placeholder="Alle Zustände"
        />
      </div>
    )
  },
}
