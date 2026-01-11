import type { Meta, StoryObj } from '@storybook/react'
import { Input } from '../src/components/ui/input'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Input> = {
  title: 'UI/Input',
  component: Input,
  tags: ['autodocs'],
  argTypes: {
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'search', 'tel', 'url'],
    },
    placeholder: {
      control: 'text',
    },
    disabled: {
      control: 'boolean',
    },
  },
  args: {
    type: 'text',
    placeholder: 'Text eingeben...',
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    placeholder: 'Name eingeben',
  },
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full max-w-sm items-center gap-1.5">
      <Label htmlFor="input-email">E-Mail</Label>
      <Input type="email" id="input-email" placeholder="beispiel@email.de" />
    </div>
  ),
}

export const Disabled: Story = {
  args: {
    disabled: true,
    placeholder: 'Deaktiviertes Feld',
  },
}

export const WithValue: Story = {
  args: {
    defaultValue: 'Vorausgefüllter Wert',
  },
}

export const Password: Story = {
  args: {
    type: 'password',
    placeholder: 'Passwort eingeben',
  },
}

export const Search: Story = {
  args: {
    type: 'search',
    placeholder: 'Suchen...',
  },
}

export const Number: Story = {
  args: {
    type: 'number',
    placeholder: '0',
  },
}

export const File: Story = {
  render: () => (
    <div className="grid w-full max-w-sm items-center gap-1.5">
      <Label htmlFor="input-file">Datei hochladen</Label>
      <Input id="input-file" type="file" />
    </div>
  ),
}
