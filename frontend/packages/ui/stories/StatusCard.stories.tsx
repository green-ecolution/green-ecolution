import type { Meta, StoryObj } from '@storybook/react-vite'
import { StatusCard } from '../src/components/ui/status-card'

const meta: Meta<typeof StatusCard> = {
  title: 'UI/StatusCard',
  component: StatusCard,
  tags: ['autodocs'],
  argTypes: {
    status: {
      control: 'select',
      options: ['default', 'red', 'yellow', 'green-dark', 'green-light'],
      description: 'Background color status',
    },
    indicator: {
      control: 'select',
      options: ['none', 'dot', 'badge'],
      description: 'Type of status indicator',
    },
    isLarge: {
      control: 'boolean',
      description: 'Use larger text for value',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    label: 'Baumanzahl in der Gruppe',
    value: '8 Bäume',
    description: 'Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind.',
  },
}

export const WithDotIndicator: Story = {
  args: {
    label: 'Bewässerungszustand (ø)',
    value: 'In Ordnung',
    description: 'Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich.',
    status: 'green-light',
    indicator: 'dot',
  },
}

export const WithBadgeIndicator: Story = {
  args: {
    label: 'Bewässerungszustand (ø)',
    value: 'In Ordnung',
    description: 'Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich.',
    status: 'green-light',
    indicator: 'badge',
  },
}

export const LargeValue: Story = {
  args: {
    label: 'Bodenfeuchte',
    value: '42.5 %',
    description: 'Wert bezeichnet den Wassergehalt im Boden.',
    isLarge: true,
  },
}

export const StatusVariants: Story = {
  render: () => (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
      <StatusCard
        status="default"
        indicator="dot"
        label="Standard"
        value="Unbekannt"
        description="Keine Daten verfügbar."
      />
      <StatusCard
        status="green-light"
        indicator="dot"
        label="Gut"
        value="In Ordnung"
        description="Die Bewässerung ist ausreichend."
      />
      <StatusCard
        status="green-dark"
        indicator="dot"
        label="Sehr gut"
        value="Optimal"
        description="Perfekte Bodenfeuchtigkeit."
      />
      <StatusCard
        status="yellow"
        indicator="dot"
        label="Warnung"
        value="Mäßig"
        description="Bewässerung bald erforderlich."
      />
      <StatusCard
        status="red"
        indicator="dot"
        label="Kritisch"
        value="Schlecht"
        description="Sofortige Bewässerung erforderlich."
      />
    </div>
  ),
}

export const DashboardGrid: Story = {
  render: () => (
    <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4">
      <StatusCard
        status="green-light"
        indicator="dot"
        label="Bewässerungszustand (ø)"
        value="In Ordnung"
        description="Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich."
      />
      <StatusCard
        label="Baumanzahl in der Gruppe"
        value="8 Bäume"
        description="Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind."
      />
      <StatusCard
        label="Standort der Gruppe"
        value="Alte Zob-Brücke, Weiche"
      />
      <StatusCard
        label="Datum der letzten Bewässerung"
        value="Keine Angabe"
        description="Wird aktualisiert, sobald ein Einsatzplan mit dieser Gruppe als »Beendet« markiert wird."
      />
    </div>
  ),
}

export const SensorData: Story = {
  render: () => (
    <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4">
      <StatusCard
        status="green-light"
        indicator="dot"
        label="Bewässerungszustand (ø)"
        value="In Ordnung"
        description="Die Bewässerung ist ausreichend."
      />
      <StatusCard
        label="Bodenfeuchte"
        value="42.5 %"
        isLarge
        description="Wert bezeichnet den Wassergehalt im Boden."
      />
      <StatusCard
        label="Bodentemperatur"
        value="18.3 °C"
        isLarge
        description="Wert bezeichnet die Temperatur in der oberflächlichen Bodenschicht."
      />
      <StatusCard
        label="Datum der letzten Bewässerung"
        value="15.01.2025"
        description="Wird aktualisiert, sobald ein Einsatzplan als »Beendet« markiert wird."
      />
    </div>
  ),
}
