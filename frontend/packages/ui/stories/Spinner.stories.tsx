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
  render: () => <Loading label="Bäume werden geladen..." />,
}

export const LoadingVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <Loading label="Daten werden geladen..." size="sm" />
      <Loading label="Anfrage wird verarbeitet..." size="default" />
      <Loading label="Bitte warten..." size="lg" />
    </div>
  ),
}

export const CenteredLoading: Story = {
  render: () => (
    <div className="flex h-64 items-center justify-center border rounded-lg">
      <Loading label="Sensordaten werden abgerufen..." size="lg" />
    </div>
  ),
}

export const FullPageLoading: Story = {
  render: () => (
    <div className="mt-20 flex flex-wrap items-center justify-center gap-x-4">
      <Loading label="Seite wird geladen..." size="default" />
    </div>
  ),
}
