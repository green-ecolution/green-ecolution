import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { MultiSelect } from '../src/components/ui/multi-select'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof MultiSelect> = {
  title: 'UI/MultiSelect',
  component: MultiSelect,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <MultiSelect
      className="w-[280px]"
      options={[
        { value: '1', label: 'Eiche' },
        { value: '2', label: 'Buche' },
        { value: '3', label: 'Ahorn' },
        { value: '4', label: 'Linde' },
        { value: '5', label: 'Birke' },
      ]}
    />
  ),
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full max-w-sm items-center gap-1.5">
      <Label htmlFor="employees">Mitarbeitende</Label>
      <p className="text-sm text-muted-foreground">
        Indem Sie die Taste »Shift« gedrückt halten, können Sie eine Mehrauswahl tätigen.
      </p>
      <MultiSelect
        id="employees"
        options={[
          { value: 'user-1', label: 'Max Mustermann · Führerschein B' },
          { value: 'user-2', label: 'Erika Musterfrau · Führerschein C' },
          { value: 'user-3', label: 'Hans Schmidt · Führerschein B, C' },
          { value: 'user-4', label: 'Anna Müller · Führerschein B' },
        ]}
      />
    </div>
  ),
}

export const WithPreselectedValues: Story = {
  render: function Render() {
    const [value, setValue] = useState(['user-1', 'user-3'])

    return (
      <div className="grid w-full max-w-sm items-center gap-1.5">
        <Label htmlFor="selected-employees">Ausgewählte Mitarbeitende</Label>
        <MultiSelect
          id="selected-employees"
          value={value}
          onChange={setValue}
          options={[
            { value: 'user-1', label: 'Max Mustermann' },
            { value: 'user-2', label: 'Erika Musterfrau' },
            { value: 'user-3', label: 'Hans Schmidt' },
            { value: 'user-4', label: 'Anna Müller' },
          ]}
        />
        <p className="text-sm text-muted-foreground">
          Ausgewählt: {value.join(', ') || 'Keine'}
        </p>
      </div>
    )
  },
}

export const WithDisabledOptions: Story = {
  render: () => (
    <MultiSelect
      className="w-[280px]"
      options={[
        { value: '1', label: 'Wassertank-LKW 1' },
        { value: '2', label: 'Wassertank-LKW 2 (In Wartung)', disabled: true },
        { value: '3', label: 'Service-Transporter 1' },
        { value: '4', label: 'Service-Transporter 2 (Defekt)', disabled: true },
      ]}
    />
  ),
}

export const Disabled: Story = {
  render: () => (
    <MultiSelect
      className="w-[280px]"
      disabled
      options={[
        { value: '1', label: 'Option 1' },
        { value: '2', label: 'Option 2' },
        { value: '3', label: 'Option 3' },
      ]}
    />
  ),
}

export const WithChildren: Story = {
  render: () => (
    <MultiSelect className="w-[280px]">
      <option value="cluster-1">Bewässerungsgruppe Nord</option>
      <option value="cluster-2">Bewässerungsgruppe Süd</option>
      <option value="cluster-3">Bewässerungsgruppe Ost</option>
      <option value="cluster-4">Bewässerungsgruppe West</option>
    </MultiSelect>
  ),
}
