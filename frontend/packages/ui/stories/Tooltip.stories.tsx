import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '../src/components/ui/tooltip'
import { Button } from '../src/components/ui/button'
import { Info, Trash2 } from 'lucide-react'

const meta: Meta<typeof Tooltip> = {
  title: 'UI/Tooltip',
  component: Tooltip,
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <TooltipProvider>
        <div className="flex min-h-32 items-center justify-center">
          <Story />
        </div>
      </TooltipProvider>
    ),
  ],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button variant="outline" size="icon" aria-label="Bodenbeschaffenheit bestimmen">
          <Info />
        </Button>
      </TooltipTrigger>
      <TooltipContent>Bodenbeschaffenheit bestimmen</TooltipContent>
    </Tooltip>
  ),
}

export const AufTextTrigger: Story = {
  render: () => (
    <Tooltip>
      <TooltipTrigger className="underline decoration-dotted underline-offset-4">
        Sl3 – lehmiger Sand
      </TooltipTrigger>
      <TooltipContent>KA5-Feinbodenart: Ton 8–12 %, Schluff 10–40 %</TooltipContent>
    </Tooltip>
  ),
}

export const MitSeitenposition: Story = {
  render: () => (
    <Tooltip>
      <TooltipTrigger asChild>
        <Button variant="destructive" size="icon" aria-label="Baum aus Gruppe entfernen">
          <Trash2 />
        </Button>
      </TooltipTrigger>
      <TooltipContent side="right">Baum aus Gruppe entfernen</TooltipContent>
    </Tooltip>
  ),
}
