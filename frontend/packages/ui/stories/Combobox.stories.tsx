import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { Combobox, type ComboboxOption } from '../src/components/ui/combobox'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Combobox> = {
  title: 'UI/Combobox',
  component: Combobox,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

const soilOptions: ComboboxOption[] = [
  { value: 'Ss', label: 'Ss – Reinsand', group: 'Sande' },
  { value: 'Sl2', label: 'Sl2 – schwach lehmiger Sand', group: 'Sande' },
  { value: 'Su3', label: 'Su3 – mittel schluffiger Sand', group: 'Sande' },
  { value: 'Uu', label: 'Uu – reiner Schluff', group: 'Schluffe' },
  { value: 'Us', label: 'Us – sandiger Schluff', group: 'Schluffe' },
  { value: 'Lu', label: 'Lu – schluffiger Lehm', group: 'Lehme' },
  { value: 'Ls3', label: 'Ls3 – mittel sandiger Lehm', group: 'Lehme' },
  { value: 'Tt', label: 'Tt – reiner Ton', group: 'Tone' },
  { value: 'Tu2', label: 'Tu2 – schwach schluffiger Ton', group: 'Tone' },
  { value: 'unknown', label: 'Unbekannt', group: 'Sonstige' },
]

export const Default: Story = {
  render: () => {
    const [value, setValue] = useState<string>()
    return (
      <div className="w-[320px]">
        <Combobox
          options={soilOptions}
          value={value}
          onChange={setValue}
          placeholder="Bodenart auswählen"
          searchPlaceholder="Code oder Bezeichnung suchen…"
        />
      </div>
    )
  },
}

export const WithLabel: Story = {
  render: () => {
    const [value, setValue] = useState<string>('Lu')
    return (
      <div className="grid w-full max-w-sm items-center gap-1.5">
        <Label htmlFor="soil-condition">Bodenart (KA5)</Label>
        <Combobox
          id="soil-condition"
          options={soilOptions}
          value={value}
          onChange={setValue}
          placeholder="Bodenart auswählen"
        />
      </div>
    )
  },
}

export const Disabled: Story = {
  render: () => (
    <div className="w-[320px]">
      <Combobox options={soilOptions} disabled placeholder="Bodenart auswählen" />
    </div>
  ),
}
