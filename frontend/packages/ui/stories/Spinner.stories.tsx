import type { Meta, StoryObj } from '@storybook/react'
import { Spinner, Loading } from '../src/components/ui/spinner'

const meta: Meta<typeof Spinner> = {
  title: 'UI/Spinner',
  component: Spinner,
  tags: ['autodocs'],
  argTypes: {
    size: {
      control: 'select',
      options: ['sm', 'default', 'lg', 'xl'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    size: 'default',
  },
}

export const Small: Story = {
  args: {
    size: 'sm',
  },
}

export const Large: Story = {
  args: {
    size: 'lg',
  },
}

export const ExtraLarge: Story = {
  args: {
    size: 'xl',
  },
}

export const AllSizes: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <Spinner size="sm" />
      <Spinner size="default" />
      <Spinner size="lg" />
      <Spinner size="xl" />
    </div>
  ),
}

export const LoadingWithLabel: Story = {
  render: () => <Loading label="Loading trees..." />,
}

export const LoadingVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <Loading label="Loading data..." size="sm" />
      <Loading label="Processing request..." size="default" />
      <Loading label="Please wait..." size="lg" />
    </div>
  ),
}

export const CenteredLoading: Story = {
  render: () => (
    <div className="flex h-64 items-center justify-center border rounded-lg">
      <Loading label="Fetching sensor data..." size="lg" />
    </div>
  ),
}

export const InButton: Story = {
  render: () => (
    <button
      disabled
      className="inline-flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-primary-foreground opacity-70"
    >
      <Spinner size="sm" className="text-primary-foreground" />
      Saving...
    </button>
  ),
}
