import type { Meta, StoryObj } from '@storybook/react'
import { Checkbox } from '../src/components/ui/checkbox'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Checkbox> = {
  title: 'UI/Checkbox',
  component: Checkbox,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="terms" />
      <Label htmlFor="terms">Accept terms and conditions</Label>
    </div>
  ),
}

export const Checked: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="checked" defaultChecked />
      <Label htmlFor="checked">Checked by default</Label>
    </div>
  ),
}

export const Disabled: Story = {
  render: () => (
    <div className="space-y-2">
      <div className="flex items-center space-x-2">
        <Checkbox id="disabled" disabled />
        <Label htmlFor="disabled" className="text-muted-foreground">
          Disabled
        </Label>
      </div>
      <div className="flex items-center space-x-2">
        <Checkbox id="disabled-checked" disabled defaultChecked />
        <Label htmlFor="disabled-checked" className="text-muted-foreground">
          Disabled & Checked
        </Label>
      </div>
    </div>
  ),
}

export const FilterExample: Story = {
  render: () => (
    <div className="space-y-4">
      <h4 className="text-sm font-medium">Filter by Status</h4>
      <div className="space-y-2">
        <div className="flex items-center space-x-2">
          <Checkbox id="healthy" defaultChecked />
          <Label htmlFor="healthy">Healthy</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="needs-water" defaultChecked />
          <Label htmlFor="needs-water">Needs Water</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="critical" />
          <Label htmlFor="critical">Critical</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="unknown" />
          <Label htmlFor="unknown">Unknown</Label>
        </div>
      </div>
    </div>
  ),
}

export const WithDescription: Story = {
  render: () => (
    <div className="items-top flex space-x-2">
      <Checkbox id="notifications" />
      <div className="grid gap-1.5 leading-none">
        <Label htmlFor="notifications">Enable notifications</Label>
        <p className="text-sm text-muted-foreground">
          You will receive daily updates about your trees.
        </p>
      </div>
    </div>
  ),
}
