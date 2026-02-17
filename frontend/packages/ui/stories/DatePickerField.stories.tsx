import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { DatePickerField } from '../src/components/ui/date-picker-field'
import { startOfToday, addMonths } from 'date-fns'

const meta: Meta<typeof DatePickerField> = {
  title: 'UI/DatePickerField',
  component: DatePickerField,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return (
      <div className="max-w-sm">
        <DatePickerField
          label="Datum"
          value={date}
          onChange={setDate}
        />
      </div>
    )
  },
}

export const WithPreselectedDate: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>(new Date())
    return (
      <div className="max-w-sm">
        <DatePickerField
          label="Datum des Einsatzes"
          value={date}
          onChange={setDate}
          required
        />
      </div>
    )
  },
}

export const WithError: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return (
      <div className="max-w-sm">
        <DatePickerField
          label="Startdatum"
          value={date}
          onChange={setDate}
          error="Datum muss heute oder in der Zukunft liegen"
          required
        />
      </div>
    )
  },
}

export const WithDescription: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return (
      <div className="max-w-sm">
        <DatePickerField
          label="Bewässerungsdatum"
          value={date}
          onChange={setDate}
          description="Wählen Sie das Datum für den nächsten Einsatz."
        />
      </div>
    )
  },
}

export const WithDateConstraints: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return (
      <div className="max-w-sm">
        <DatePickerField
          label="Einsatzdatum"
          value={date}
          onChange={setDate}
          fromDate={startOfToday()}
          toDate={addMonths(startOfToday(), 3)}
          description="Nur Termine innerhalb der nächsten 3 Monate wählbar."
          required
        />
      </div>
    )
  },
}

export const Disabled: Story = {
  render: () => (
    <div className="max-w-sm">
      <DatePickerField
        label="Datum"
        value={new Date()}
        disabled
      />
    </div>
  ),
}

export const FormExample: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return (
      <form className="flex flex-col gap-y-4 max-w-md">
        <DatePickerField
          label="Datum des Einsatzes"
          value={date}
          onChange={setDate}
          required
          fromDate={startOfToday()}
        />
      </form>
    )
  },
}
