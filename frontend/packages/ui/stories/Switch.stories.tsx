import { useState } from 'react'
import type { Meta, StoryObj } from '@storybook/react-vite'
import { Switch } from '../src/components/ui/switch'

const meta: Meta<typeof Switch> = {
  title: 'UI/Switch',
  component: Switch,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => {
    const [checked, setChecked] = useState(true)
    return (
      <label className="flex items-center gap-3">
        <Switch checked={checked} onCheckedChange={setChecked} />
        <span className="text-sm">Bäume ansehen</span>
      </label>
    )
  },
}

export const Disabled: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <label className="flex items-center gap-3">
        <Switch checked disabled onCheckedChange={() => {}} />
        <span className="text-sm text-dark-600">Baum löschen (nicht vergebbar)</span>
      </label>
      <label className="flex items-center gap-3">
        <Switch checked={false} disabled onCheckedChange={() => {}} />
        <span className="text-sm text-dark-600">Sensor anlegen (nicht vergebbar)</span>
      </label>
    </div>
  ),
}
