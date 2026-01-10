import type { Meta, StoryObj } from '@storybook/react'
import { Textarea } from '../src/components/ui/textarea'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Textarea> = {
  title: 'UI/Textarea',
  component: Textarea,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    placeholder: 'Type your message here.',
  },
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="message">Your message</Label>
      <Textarea placeholder="Type your message here." id="message" />
    </div>
  ),
}

export const WithDescription: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="notes">Notes</Label>
      <Textarea placeholder="Add notes about this tree..." id="notes" />
      <p className="text-sm text-muted-foreground">
        Notes are only visible to administrators.
      </p>
    </div>
  ),
}

export const Disabled: Story = {
  args: {
    placeholder: 'This textarea is disabled.',
    disabled: true,
  },
}

export const WithDefaultValue: Story = {
  args: {
    defaultValue: 'This tree was planted in 2019 as part of the urban greening initiative. It requires regular watering during summer months.',
  },
}

export const CustomRows: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="grid gap-1.5">
        <Label>Small (2 rows)</Label>
        <Textarea placeholder="Short input..." rows={2} />
      </div>
      <div className="grid gap-1.5">
        <Label>Default (3 rows)</Label>
        <Textarea placeholder="Default input..." rows={3} />
      </div>
      <div className="grid gap-1.5">
        <Label>Large (6 rows)</Label>
        <Textarea placeholder="Large input..." rows={6} />
      </div>
    </div>
  ),
}

export const TreeNotesExample: Story = {
  render: () => (
    <div className="max-w-md space-y-4">
      <div className="grid gap-1.5">
        <Label htmlFor="tree-notes">Tree Notes</Label>
        <Textarea
          id="tree-notes"
          placeholder="Add observations, maintenance notes, or other relevant information about this tree..."
          rows={4}
        />
        <p className="text-sm text-muted-foreground">
          Maximum 1000 characters. Last updated: Never
        </p>
      </div>
    </div>
  ),
}

export const WateringReportExample: Story = {
  render: () => (
    <div className="max-w-md space-y-4">
      <div className="grid gap-1.5">
        <Label htmlFor="report">Watering Report</Label>
        <Textarea
          id="report"
          placeholder="Describe any issues encountered during the watering route..."
          rows={5}
          defaultValue="Route completed successfully. Tree #456 on Elm Street had blocked drainage - notified maintenance team. All other trees watered as planned."
        />
        <div className="flex justify-between text-sm text-muted-foreground">
          <span>Be specific about any problems encountered</span>
          <span>245/1000</span>
        </div>
      </div>
    </div>
  ),
}
