import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { SelectField } from '../src/components/ui/select-field'

const meta: Meta<typeof SelectField> = {
  title: 'UI/SelectField',
  component: SelectField,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

const clusterOptions = [
  { value: '-1', label: 'Keine Bewässerungsgruppe' },
  { value: '1', label: 'Flensburg Altstadt' },
  { value: '2', label: 'Sandberg' },
  { value: '3', label: 'Mürwik' },
]

export const Default: Story = {
  args: {
    label: 'Bewässerungsgruppe',
    placeholder: 'Wähle eine Bewässerungsgruppe aus',
    options: clusterOptions,
  },
}

export const Required: Story = {
  args: {
    label: 'Fahrzeugtyp',
    placeholder: 'Fahrzeugtyp',
    required: true,
    options: [
      { value: 'trailer', label: 'Anhänger' },
      { value: 'transporter', label: 'Transporter' },
    ],
  },
}

export const WithError: Story = {
  args: {
    label: 'Verknüpftes Fahrzeug',
    placeholder: 'Wähle ein Fahrzeug aus',
    required: true,
    error: 'Es muss ein Fahrzeug ausgewählt werden',
    options: [
      { value: '1', label: 'FL-GE 123 · BE' },
      { value: '2', label: 'FL-GE 456 · B' },
    ],
  },
}

export const WithDescription: Story = {
  args: {
    label: 'Verknüpfter Sensor',
    placeholder: 'Wähle einen Sensor aus, sofern vorhanden',
    description: 'Der Sensor misst die Bodenfeuchte am Standort des Baumes.',
    options: [
      { value: '-1', label: 'Kein Sensor' },
      { value: 'sensor-1', label: 'Sensor 1' },
      { value: 'sensor-2', label: 'Sensor 2' },
    ],
  },
}

export const Disabled: Story = {
  args: {
    label: 'Führerscheinklasse',
    value: 'be',
    disabled: true,
    options: [
      { value: 'b', label: 'B' },
      { value: 'be', label: 'BE' },
      { value: 'c', label: 'C' },
      { value: 'ce', label: 'CE' },
    ],
  },
}

const ControlledExample = () => {
  const [value, setValue] = useState('-1')

  return (
    <div className="flex flex-col gap-y-4 max-w-md">
      <SelectField
        label="Bewässerungsgruppe"
        placeholder="Wähle eine Bewässerungsgruppe aus"
        value={value}
        onValueChange={setValue}
        options={clusterOptions}
      />
      <p className="text-sm text-muted-foreground">Ausgewählter Wert: {value}</p>
    </div>
  )
}

export const Controlled: Story = {
  args: { label: 'Bewässerungsgruppe', options: clusterOptions },
  render: () => <ControlledExample />,
}
