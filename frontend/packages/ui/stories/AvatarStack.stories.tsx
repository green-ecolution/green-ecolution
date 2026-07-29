import type { Meta, StoryObj } from '@storybook/react-vite'
import { AvatarStack } from '../src/components/ui/avatar-stack'

const meta: Meta<typeof AvatarStack> = {
  title: 'UI/AvatarStack',
  component: AvatarStack,
  tags: ['autodocs'],
  argTypes: {
    size: { control: 'select', options: ['xs', 'sm', 'default'] },
    max: { control: { type: 'number', min: 1, max: 8 } },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Team: Story = {
  args: { items: ['AK', 'MB', 'TS'] },
}

export const WithOverflow: Story = {
  name: 'Mit Überlauf',
  args: { items: ['AK', 'MB', 'TS', 'JL', 'CH', 'RS', 'BM', 'GF', 'TD', 'TJ', 'EP', 'GP', 'SN'] },
}

export const Empty: Story = {
  name: 'Ohne Personen',
  args: { items: [] },
}
