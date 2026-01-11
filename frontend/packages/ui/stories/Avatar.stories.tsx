import type { Meta, StoryObj } from '@storybook/react'
import { UserRound } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '../src/components/ui/avatar'

const meta: Meta<typeof Avatar> = {
  title: 'UI/Avatar',
  component: Avatar,
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
  render: () => (
    <Avatar>
      <AvatarImage src="https://avatars.githubusercontent.com/u/73289312?v=4" alt="@choffmann" />
      <AvatarFallback>CH</AvatarFallback>
    </Avatar>
  ),
}

export const WithInitials: Story = {
  render: () => (
    <Avatar>
      <AvatarFallback>CH</AvatarFallback>
    </Avatar>
  ),
}

export const UserVariant: Story = {
  render: () => (
    <div className="group">
      <Avatar>
        <AvatarFallback variant="user">CH</AvatarFallback>
      </Avatar>
    </div>
  ),
}

export const GuestVariant: Story = {
  render: () => (
    <div className="group">
      <Avatar>
        <AvatarFallback variant="guest">
          <UserRound className="h-5 w-5 stroke-2" />
        </AvatarFallback>
      </Avatar>
    </div>
  ),
}

export const Sizes: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <Avatar size="sm">
        <AvatarFallback variant="user">SM</AvatarFallback>
      </Avatar>
      <Avatar size="default">
        <AvatarFallback variant="user">MD</AvatarFallback>
      </Avatar>
      <Avatar size="lg">
        <AvatarFallback variant="user">LG</AvatarFallback>
      </Avatar>
      <Avatar size="xl">
        <AvatarFallback variant="user">XL</AvatarFallback>
      </Avatar>
    </div>
  ),
}

export const WithImage: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <Avatar>
        <AvatarImage src="https://avatars.githubusercontent.com/u/73289312?v=4" alt="@choffmann" />
        <AvatarFallback>CH</AvatarFallback>
      </Avatar>
      <Avatar size="lg">
        <AvatarImage src="https://avatars.githubusercontent.com/u/73289312?v=4" alt="@choffmann" />
        <AvatarFallback>CH</AvatarFallback>
      </Avatar>
      <Avatar size="xl">
        <AvatarImage src="https://avatars.githubusercontent.com/u/73289312?v=4" alt="@choffmann" />
        <AvatarFallback>CH</AvatarFallback>
      </Avatar>
    </div>
  ),
}

export const ImageFallback: Story = {
  render: () => (
    <Avatar>
      <AvatarImage src="https://invalid-url.example/broken.png" alt="Broken" />
      <AvatarFallback variant="user">FB</AvatarFallback>
    </Avatar>
  ),
}

export const AvatarGroup: Story = {
  render: () => (
    <div className="flex -space-x-3">
      <Avatar className="ring-2 ring-white">
        <AvatarFallback variant="user">A1</AvatarFallback>
      </Avatar>
      <Avatar className="ring-2 ring-white">
        <AvatarFallback variant="user">A2</AvatarFallback>
      </Avatar>
      <Avatar className="ring-2 ring-white">
        <AvatarFallback variant="user">A3</AvatarFallback>
      </Avatar>
      <Avatar className="ring-2 ring-white">
        <AvatarFallback variant="default">+5</AvatarFallback>
      </Avatar>
    </div>
  ),
}

export const UserProfileExample: Story = {
  render: () => (
    <div className="flex items-center gap-3">
      <div className="group">
        <Avatar>
          <AvatarFallback variant="user">CH</AvatarFallback>
        </Avatar>
      </div>
      <div>
        <p className="font-semibold text-sm">Cedrik Hoffmann</p>
        <p className="text-xs text-muted-foreground">choffmann@green-ecolution.de</p>
      </div>
    </div>
  ),
}
