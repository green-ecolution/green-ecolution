import type { Meta, StoryObj } from '@storybook/react'
import { Separator } from '../src/components/ui/separator'

const meta: Meta<typeof Separator> = {
  title: 'UI/Separator',
  component: Separator,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Horizontal: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium">Tree Information</h4>
        <p className="text-sm text-muted-foreground">
          Basic details about the selected tree.
        </p>
      </div>
      <Separator />
      <div>
        <h4 className="text-sm font-medium">Sensor Data</h4>
        <p className="text-sm text-muted-foreground">
          Live readings from connected sensors.
        </p>
      </div>
    </div>
  ),
}

export const Vertical: Story = {
  render: () => (
    <div className="flex h-5 items-center space-x-4 text-sm">
      <div>Trees</div>
      <Separator orientation="vertical" />
      <div>Vehicles</div>
      <Separator orientation="vertical" />
      <div>Sensors</div>
      <Separator orientation="vertical" />
      <div>Plans</div>
    </div>
  ),
}

export const InCard: Story = {
  render: () => (
    <div className="rounded-lg border p-4">
      <div className="space-y-1">
        <h4 className="text-sm font-medium leading-none">Oak Tree #1234</h4>
        <p className="text-sm text-muted-foreground">Main Street 42</p>
      </div>
      <Separator className="my-4" />
      <div className="flex h-5 items-center space-x-4 text-sm">
        <div>Status: Healthy</div>
        <Separator orientation="vertical" />
        <div>Last watered: 3 days ago</div>
      </div>
    </div>
  ),
}

export const FormSections: Story = {
  render: () => (
    <div className="space-y-6">
      <div className="space-y-2">
        <h3 className="text-lg font-medium">General Information</h3>
        <p className="text-sm text-muted-foreground">
          Enter the basic information about the tree.
        </p>
        <div className="h-10 rounded bg-muted" />
        <div className="h-10 rounded bg-muted" />
      </div>

      <Separator />

      <div className="space-y-2">
        <h3 className="text-lg font-medium">Location</h3>
        <p className="text-sm text-muted-foreground">
          Specify the tree&apos;s location.
        </p>
        <div className="h-10 rounded bg-muted" />
      </div>

      <Separator />

      <div className="space-y-2">
        <h3 className="text-lg font-medium">Additional Details</h3>
        <p className="text-sm text-muted-foreground">
          Optional information about the tree.
        </p>
        <div className="h-20 rounded bg-muted" />
      </div>
    </div>
  ),
}

export const MenuDivider: Story = {
  render: () => (
    <div className="w-48 rounded-lg border p-2">
      <div className="px-2 py-1.5 text-sm">Dashboard</div>
      <div className="px-2 py-1.5 text-sm">Trees</div>
      <div className="px-2 py-1.5 text-sm">Vehicles</div>
      <Separator className="my-2" />
      <div className="px-2 py-1.5 text-sm">Settings</div>
      <div className="px-2 py-1.5 text-sm">Help</div>
      <Separator className="my-2" />
      <div className="px-2 py-1.5 text-sm text-destructive">Logout</div>
    </div>
  ),
}
