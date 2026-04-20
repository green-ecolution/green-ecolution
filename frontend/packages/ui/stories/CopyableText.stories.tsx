import type { Meta, StoryObj } from '@storybook/react-vite'
import { CopyableText } from '../src/components/ui/copyable-text'

const meta: Meta<typeof CopyableText> = {
  title: 'UI/CopyableText',
  component: CopyableText,
  tags: ['autodocs'],
  argTypes: {
    value: { control: 'text' },
    label: { control: 'text' },
  },
  args: {
    value: 'eui-a84041d55188a64d',
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const WithLabel: Story = {
  args: {
    label: 'Sensor-ID',
    value: 'eui-a84041d55188a64d',
  },
}

export const ShortValue: Story = {
  args: {
    label: 'Code',
    value: 'ABC-123',
  },
}

export const LongValue: Story = {
  args: {
    label: 'API Key',
    value: 'sk-proj-a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6q7r8s9t0u1v2w3x4y5z6',
  },
}
