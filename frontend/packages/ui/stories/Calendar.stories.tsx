import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { Calendar } from '../src/components/ui/calendar'
import { de } from 'date-fns/locale'
import { addDays } from 'date-fns'

const meta: Meta<typeof Calendar> = {
  title: 'UI/Calendar',
  component: Calendar,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>(new Date())
    return <Calendar mode="single" selected={date} onSelect={setDate} locale={de} />
  },
}

export const WithoutSelectedDate: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    return <Calendar mode="single" selected={date} onSelect={setDate} locale={de} />
  },
}

export const DisabledDates: Story = {
  render: () => {
    const [date, setDate] = useState<Date | undefined>()
    const today = new Date()
    return (
      <Calendar
        mode="single"
        selected={date}
        onSelect={setDate}
        locale={de}
        disabled={[{ before: today }]}
      />
    )
  },
}

export const DateRange: Story = {
  render: () => {
    const [range, setRange] = useState<{ from: Date; to?: Date } | undefined>({
      from: new Date(),
      to: addDays(new Date(), 7),
    })
    return (
      <Calendar mode="range" selected={range} onSelect={setRange} locale={de} numberOfMonths={2} />
    )
  },
}
