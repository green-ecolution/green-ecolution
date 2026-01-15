import type { Meta, StoryObj } from '@storybook/react-vite'
import { ArrowRight, MoveRight, MoveLeft, Loader2 } from 'lucide-react'
import { Button } from '../src/components/ui/button'

const meta: Meta<typeof Button> = {
  title: 'UI/Button',
  component: Button,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'destructive', 'outline', 'secondary', 'ghost', 'link', 'nav'],
      description: 'The visual style of the button',
    },
    size: {
      control: 'select',
      options: ['default', 'sm', 'lg', 'icon'],
      description: 'The size of the button',
    },
    disabled: {
      control: 'boolean',
      description: 'Whether the button is disabled',
    },
    asChild: {
      control: 'boolean',
      description: 'Render as child element (for use with links)',
    },
  },
  args: {
    children: 'Button',
    variant: 'default',
    size: 'default',
    disabled: false,
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    children: 'Primärer Button',
    variant: 'default',
  },
}

export const WithIcon: Story = {
  args: {
    children: (
      <>
        Nächster Schritt
        <ArrowRight className="ml-2 h-4 w-4" />
      </>
    ),
    variant: 'default',
  },
}

export const Destructive: Story = {
  args: {
    children: 'Löschen',
    variant: 'destructive',
  },
}

export const Outline: Story = {
  args: {
    children: 'Umrandet',
    variant: 'outline',
  },
}

export const Secondary: Story = {
  args: {
    children: 'Sekundär',
    variant: 'secondary',
  },
}

export const Ghost: Story = {
  args: {
    children: 'Ghost',
    variant: 'ghost',
  },
}

export const Link: Story = {
  args: {
    children: 'Link-Button',
    variant: 'link',
  },
}

export const Small: Story = {
  args: {
    children: 'Klein',
    size: 'sm',
  },
}

export const Large: Story = {
  args: {
    children: 'Groß',
    size: 'lg',
  },
}

export const Loading: Story = {
  args: {
    children: (
      <>
        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
        Wird geladen...
      </>
    ),
    disabled: true,
  },
}

export const Disabled: Story = {
  args: {
    children: 'Deaktiviert',
    disabled: true,
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      <Button variant="default">Standard</Button>
      <Button variant="secondary">Sekundär</Button>
      <Button variant="destructive">Destruktiv</Button>
      <Button variant="outline">Umrandet</Button>
      <Button variant="ghost">Ghost</Button>
      <Button variant="link">Link</Button>
      <Button variant="nav">Navigation</Button>
    </div>
  ),
}

export const AllSizes: Story = {
  render: () => (
    <div className="flex items-center gap-4">
      <Button size="sm">Klein</Button>
      <Button size="default">Standard</Button>
      <Button size="lg">Groß</Button>
    </div>
  ),
}

export const BackLink: Story = {
  render: () => (
    <Button variant="nav" className="group p-0 h-auto">
      <MoveLeft className="mr-2 h-4 w-4 transition-all duration-300 group-hover:-translate-x-1" />
      Zu allen Bewässerungsgruppen
    </Button>
  ),
}

export const LinkWithArrow: Story = {
  render: () => (
    <Button variant="nav" className="group p-0 h-auto">
      Auf der Karte anzeigen
      <MoveRight className="ml-2 h-4 w-4 transition-all duration-300 group-hover:translate-x-1" />
    </Button>
  ),
}

export const NavigationLinks: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <Button variant="nav" className="group p-0 h-auto">
          <MoveLeft className="mr-2 h-4 w-4 transition-all duration-300 group-hover:-translate-x-1" />
          Zu allen Bewässerungsgruppen
        </Button>
      </div>
      <h1 className="text-2xl font-bold font-lato">Bewässerungsgruppe: Alsterbogen</h1>
      <p className="text-muted-foreground">Straßenbäume mit regelmäßiger Pflege</p>
      <div>
        <Button variant="nav" className="group p-0 h-auto">
          Auf der Karte anzeigen
          <MoveRight className="ml-2 h-4 w-4 transition-all duration-300 group-hover:translate-x-1" />
        </Button>
      </div>
    </div>
  ),
}
