import type { Meta, StoryObj } from '@storybook/react'
import { FormField, TextareaField } from '../src/components/ui/form-field'

const meta: Meta<typeof FormField> = {
  title: 'UI/FormField',
  component: FormField,
  tags: ['autodocs'],
  argTypes: {
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'tel', 'url'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    label: 'Name',
    placeholder: 'Enter your name',
  },
}

export const WithError: Story = {
  args: {
    label: 'Email',
    type: 'email',
    placeholder: 'Enter your email',
    error: 'Please enter a valid email address',
    defaultValue: 'invalid-email',
  },
}

export const WithDescription: Story = {
  args: {
    label: 'Username',
    placeholder: 'Enter username',
    description: 'This will be your public display name.',
  },
}

export const Required: Story = {
  args: {
    label: 'Full Name',
    placeholder: 'Enter your full name',
    required: true,
  },
}

export const Disabled: Story = {
  args: {
    label: 'Readonly Field',
    defaultValue: 'This value cannot be changed',
    disabled: true,
  },
}

export const HiddenLabel: Story = {
  args: {
    label: 'Search',
    placeholder: 'Search...',
    hideLabel: true,
  },
}

export const TextareaExample: Story = {
  render: () => (
    <TextareaField
      label="Description"
      placeholder="Enter a description..."
      description="Maximum 500 characters."
    />
  ),
}

export const TextareaWithError: Story = {
  render: () => (
    <TextareaField
      label="Notes"
      placeholder="Enter notes..."
      error="Notes are required"
      required
    />
  ),
}

export const FormExample: Story = {
  render: () => (
    <form className="space-y-4 max-w-md">
      <FormField
        label="Tree Name"
        placeholder="e.g., Oak Tree #123"
        required
      />
      <FormField
        label="Location"
        placeholder="Street address or coordinates"
        description="Enter the exact location of the tree."
      />
      <FormField
        label="Planting Year"
        type="number"
        placeholder="e.g., 2020"
        min={1900}
        max={new Date().getFullYear()}
      />
      <TextareaField
        label="Notes"
        placeholder="Additional information about the tree..."
        rows={4}
      />
    </form>
  ),
}
