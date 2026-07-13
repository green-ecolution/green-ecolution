import type { Meta, StoryObj } from '@storybook/react-vite'
import { ChevronDown, Pencil, Trash2 } from 'lucide-react'
import { Button } from '../src/components/ui/button'
import {
  ButtonGroup,
  ButtonGroupSeparator,
  ButtonGroupText,
} from '../src/components/ui/button-group'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '../src/components/ui/dropdown-menu'

const meta: Meta<typeof ButtonGroup> = {
  title: 'UI/ButtonGroup',
  component: ButtonGroup,
  tags: ['autodocs'],
  argTypes: {
    orientation: {
      control: 'select',
      options: ['horizontal', 'vertical'],
      description: 'The layout direction of the group',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <ButtonGroup>
      <Button variant="outline">Tag</Button>
      <Button variant="outline">Woche</Button>
      <Button variant="outline">Monat</Button>
    </ButtonGroup>
  ),
}

export const SplitButtonWithDropdown: Story = {
  render: () => (
    <ButtonGroup>
      <Button variant="outline">
        Gruppe bearbeiten
        <Pencil className="stroke-1" />
      </Button>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button
            variant="outline"
            size="icon"
            aria-label="Weitere Aktionen"
            className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-300 data-[state=open]:[&_svg]:rotate-180"
          >
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-[12rem]">
          <DropdownMenuItem className="gap-2 px-3 py-2 text-destructive focus:bg-destructive/10 focus:text-destructive">
            <Trash2 />
            Gruppe löschen
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </ButtonGroup>
  ),
}

export const WithText: Story = {
  render: () => (
    <ButtonGroup>
      <ButtonGroupText>Sortieren</ButtonGroupText>
      <Button variant="outline">Name</Button>
      <Button variant="outline">Datum</Button>
    </ButtonGroup>
  ),
}

export const WithSeparator: Story = {
  render: () => (
    <ButtonGroup>
      <Button>Einsatzplan starten</Button>
      <ButtonGroupSeparator />
      <Button size="icon" aria-label="Weitere Aktionen">
        <ChevronDown />
      </Button>
    </ButtonGroup>
  ),
}

export const Vertical: Story = {
  render: () => (
    <ButtonGroup orientation="vertical">
      <Button variant="outline">Hineinzoomen</Button>
      <Button variant="outline">Herauszoomen</Button>
    </ButtonGroup>
  ),
}
