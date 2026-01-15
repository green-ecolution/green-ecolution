import type { Meta, StoryObj } from '@storybook/react-vite'
import { Badge } from '../src/components/ui/badge'

const meta: Meta<typeof Badge> = {
  title: 'UI/Badge',
  component: Badge,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: [
        'default',
        'secondary',
        'destructive',
        'outline',
        'success',
        'warning',
        'error',
        'muted',
        'green-dark',
        'green-light',
        'outline-red',
        'outline-yellow',
        'outline-dark',
        'outline-green-dark',
        'outline-green-light',
      ],
    },
    size: {
      control: 'select',
      options: ['default', 'lg'],
    },
  },
  args: {
    children: 'Badge',
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    children: 'Default',
    variant: 'default',
  },
}

export const Secondary: Story = {
  args: {
    children: 'Secondary',
    variant: 'secondary',
  },
}

export const Destructive: Story = {
  args: {
    children: 'Destructive',
    variant: 'destructive',
  },
}

export const Outline: Story = {
  args: {
    children: 'Outline',
    variant: 'outline',
  },
}

export const Success: Story = {
  args: {
    children: 'Aktiv',
    variant: 'success',
  },
}

export const Warning: Story = {
  args: {
    children: 'Ausstehend',
    variant: 'warning',
  },
}

export const Error: Story = {
  args: {
    children: 'Inaktiv',
    variant: 'error',
  },
}

export const Muted: Story = {
  args: {
    children: 'Unbekannt',
    variant: 'muted',
  },
}

export const GreenDark: Story = {
  args: {
    children: 'Gesund',
    variant: 'green-dark',
  },
}

export const GreenLight: Story = {
  args: {
    children: 'Wachsend',
    variant: 'green-light',
  },
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge variant="default">Default</Badge>
      <Badge variant="secondary">Secondary</Badge>
      <Badge variant="destructive">Destructive</Badge>
      <Badge variant="outline">Outline</Badge>
      <Badge variant="success">Success</Badge>
      <Badge variant="warning">Warning</Badge>
      <Badge variant="error">Error</Badge>
      <Badge variant="muted">Muted</Badge>
      <Badge variant="green-dark">Green Dark</Badge>
      <Badge variant="green-light">Green Light</Badge>
    </div>
  ),
}

export const StatusExamples: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium mb-2">Baumstatus</h4>
        <div className="flex gap-2">
          <Badge variant="success">Gesund</Badge>
          <Badge variant="warning">Bewässerung nötig</Badge>
          <Badge variant="error">Kritisch</Badge>
          <Badge variant="muted">Unbekannt</Badge>
        </div>
      </div>
      <div>
        <h4 className="text-sm font-medium mb-2">Sensorstatus</h4>
        <div className="flex gap-2">
          <Badge variant="green-dark">Online</Badge>
          <Badge variant="error">Offline</Badge>
          <Badge variant="warning">Niedriger Akkustand</Badge>
        </div>
      </div>
    </div>
  ),
}

export const Sizes: Story = {
  render: () => (
    <div className="flex items-center gap-2">
      <Badge size="default" variant="success">
        Standard
      </Badge>
      <Badge size="lg" variant="success">
        Groß
      </Badge>
    </div>
  ),
}

export const OutlineVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge variant="outline-red">Kritisch</Badge>
      <Badge variant="outline-yellow">Warnung</Badge>
      <Badge variant="outline-dark">Inaktiv</Badge>
      <Badge variant="outline-green-dark">Aktiv</Badge>
      <Badge variant="outline-green-light">Abgeschlossen</Badge>
    </div>
  ),
}

export const OutlineLarge: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Badge variant="outline-red" size="lg">
        Kritisch
      </Badge>
      <Badge variant="outline-yellow" size="lg">
        Warnung
      </Badge>
      <Badge variant="outline-dark" size="lg">
        Inaktiv
      </Badge>
      <Badge variant="outline-green-dark" size="lg">
        Aktiv
      </Badge>
      <Badge variant="outline-green-light" size="lg">
        Abgeschlossen
      </Badge>
    </div>
  ),
}

export const WateringPlanStatus: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="mb-2 text-sm font-medium">Bewässerungsplan-Status (Outline/Large)</h4>
        <div className="flex flex-wrap gap-2">
          <Badge variant="outline-dark" size="lg">
            Geplant
          </Badge>
          <Badge variant="outline-yellow" size="lg">
            In Bearbeitung
          </Badge>
          <Badge variant="outline-green-light" size="lg">
            Abgeschlossen
          </Badge>
          <Badge variant="outline-red" size="lg">
            Abgebrochen
          </Badge>
        </div>
      </div>
    </div>
  ),
}
