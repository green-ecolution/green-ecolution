import type { Meta, StoryObj } from '@storybook/react'
import { Badge } from '../src/components/ui/badge'

const meta: Meta<typeof Badge> = {
  title: 'UI/Badge',
  component: Badge,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: [
        'default',
        'secondary',
        'destructive',
        'outline',
        'success',
        'warning',
        'error',
        'muted',
        'green-dark',
        'green-light',
      ],
    },
  },
  args: {
    children: 'Badge',
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    children: 'Default',
    variant: 'default',
  },
}

export const Secondary: Story = {
  args: {
    children: 'Secondary',
    variant: 'secondary',
  },
}

export const Destructive: Story = {
  args: {
    children: 'Destructive',
    variant: 'destructive',
  },
}

export const Outline: Story = {
  args: {
    children: 'Outline',
    variant: 'outline',
  },
}

export const Success: Story = {
  args: {
    children: 'Active',
    variant: 'success',
  },
}

export const Warning: Story = {
  args: {
    children: 'Pending',
    variant: 'warning',
  },
}

export const Error: Story = {
  args: {
    children: 'Inactive',
    variant: 'error',
  },
}

export const Muted: Story = {
  args: {
    children: 'Unknown',
    variant: 'muted',
  },
}

export const GreenDark: Story = {
  args: {
    children: 'Healthy',
    variant: 'green-dark',
  },
}

export const GreenLight: Story = {
  args: {
    children: 'Growing',
    variant: 'green-light',
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge variant="default">Default</Badge>
      <Badge variant="secondary">Secondary</Badge>
      <Badge variant="destructive">Destructive</Badge>
      <Badge variant="outline">Outline</Badge>
      <Badge variant="success">Success</Badge>
      <Badge variant="warning">Warning</Badge>
      <Badge variant="error">Error</Badge>
      <Badge variant="muted">Muted</Badge>
      <Badge variant="green-dark">Green Dark</Badge>
      <Badge variant="green-light">Green Light</Badge>
    </div>
  ),
}

export const StatusExamples: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium mb-2">Tree Status</h4>
        <div className="flex gap-2">
          <Badge variant="success">Healthy</Badge>
          <Badge variant="warning">Needs Water</Badge>
          <Badge variant="error">Critical</Badge>
          <Badge variant="muted">Unknown</Badge>
        </div>
      </div>
      <div>
        <h4 className="text-sm font-medium mb-2">Sensor Status</h4>
        <div className="flex gap-2">
          <Badge variant="green-dark">Online</Badge>
          <Badge variant="error">Offline</Badge>
          <Badge variant="warning">Low Battery</Badge>
        </div>
      </div>
    </div>
  ),
}
