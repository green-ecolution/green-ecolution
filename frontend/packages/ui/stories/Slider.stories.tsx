import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { Slider } from '../src/components/ui/slider'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Slider> = {
  title: 'UI/Slider',
  component: Slider,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => <Slider defaultValue={[50]} max={100} step={1} />,
}

export const RangeSlider: Story = {
  render: () => <Slider defaultValue={[25, 75]} max={100} step={1} />,
}

export const PlantingYearFilter: Story = {
  render: () => {
    const currentYear = new Date().getFullYear()
    const minYear = currentYear - 10
    const [range, setRange] = useState([currentYear - 3, currentYear])

    return (
      <div className="w-64">
        <Label className="mb-4 block">
          Pflanzjahr: {range[0]} - {range[1]}
        </Label>
        <Slider
          value={range}
          onValueChange={setRange}
          min={minYear}
          max={currentYear}
          step={1}
          showLabels
        />
      </div>
    )
  },
}

export const WithLabels: Story = {
  render: () => (
    <div className="w-64">
      <Label className="mb-4 block">Bewertung</Label>
      <Slider defaultValue={[3]} min={1} max={5} step={1} showLabels />
    </div>
  ),
}

export const Disabled: Story = {
  render: () => <Slider defaultValue={[50]} max={100} step={1} disabled />,
}
