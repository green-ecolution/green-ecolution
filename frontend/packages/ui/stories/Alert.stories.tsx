import type { Meta, StoryObj } from '@storybook/react'
import { AlertCircle, CheckCircle2, Info, TriangleAlert } from 'lucide-react'
import { Alert, AlertDescription, AlertTitle, InlineAlert } from '../src/components/ui/alert'

const meta: Meta<typeof Alert> = {
  title: 'UI/Alert',
  component: Alert,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'destructive', 'warning', 'success'],
    },
    size: {
      control: 'select',
      options: ['default', 'compact'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Alert>
      <Info />
      <AlertTitle>Information</AlertTitle>
      <AlertDescription>
        Dies ist eine informative Hinweismeldung.
      </AlertDescription>
    </Alert>
  ),
}

export const Destructive: Story = {
  render: () => (
    <Alert variant="destructive">
      <AlertCircle />
      <AlertTitle>Fehler</AlertTitle>
      <AlertDescription>
        Ein Fehler ist aufgetreten. Bitte versuche es später erneut.
      </AlertDescription>
    </Alert>
  ),
}

export const Warning: Story = {
  render: () => (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Warnung</AlertTitle>
      <AlertDescription>
        Bitte überprüfe deine Eingaben, bevor du fortfährst.
      </AlertDescription>
    </Alert>
  ),
}

export const Success: Story = {
  render: () => (
    <Alert variant="success">
      <CheckCircle2 />
      <AlertTitle>Erfolg</AlertTitle>
      <AlertDescription>
        Deine Änderungen wurden erfolgreich gespeichert.
      </AlertDescription>
    </Alert>
  ),
}

export const AllVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <Alert>
        <Info />
        <AlertTitle>Standard</AlertTitle>
        <AlertDescription>Standard-Hinweismeldung.</AlertDescription>
      </Alert>
      <Alert variant="success">
        <CheckCircle2 />
        <AlertTitle>Erfolg</AlertTitle>
        <AlertDescription>Erfolgs-Hinweismeldung.</AlertDescription>
      </Alert>
      <Alert variant="warning">
        <TriangleAlert />
        <AlertTitle>Warnung</AlertTitle>
        <AlertDescription>Warnungs-Hinweismeldung.</AlertDescription>
      </Alert>
      <Alert variant="destructive">
        <AlertCircle />
        <AlertTitle>Fehler</AlertTitle>
        <AlertDescription>Fehler-Hinweismeldung.</AlertDescription>
      </Alert>
    </div>
  ),
}

export const SensorNotice: Story = {
  render: () => (
    <Alert>
      <Info />
      <AlertTitle>Hinweis: Dieser Baum ist nicht mit einem Sensor ausgestattet.</AlertTitle>
      <AlertDescription>
        Dieser Baum wurde bisher nicht mit einem Sensor ausgestattet, sodass keine
        Informationen über den aktuellen Bewässerungszustand angezeigt werden können.
        Aus diesem Grund wird der Bewässerungszustand als unbekannt ausgezeichnet.
      </AlertDescription>
    </Alert>
  ),
}

export const WateringAlert: Story = {
  render: () => (
    <Alert variant="warning">
      <TriangleAlert />
      <AlertTitle>Bewässerung erforderlich</AlertTitle>
      <AlertDescription>
        Die Bodenfeuchtigkeit liegt unter dem kritischen Schwellenwert.
        Bitte plane eine Bewässerung für diesen Standort ein.
      </AlertDescription>
    </Alert>
  ),
}

export const RouteCompleted: Story = {
  render: () => (
    <Alert variant="success">
      <CheckCircle2 />
      <AlertTitle>Route abgeschlossen</AlertTitle>
      <AlertDescription>
        Die Bewässerungsroute wurde erfolgreich abgeschlossen.
        Alle 24 Bäume wurden bewässert.
      </AlertDescription>
    </Alert>
  ),
}

export const Compact: Story = {
  render: () => (
    <Alert variant="destructive" size="compact">
      <TriangleAlert />
      <p>Dies ist eine kompakte Warnmeldung.</p>
    </Alert>
  ),
}

export const InlineAlertExample: Story = {
  render: () => (
    <InlineAlert description="Bitte füllen Sie alle Pflichtfelder aus." />
  ),
}

export const InlineAlertExamples: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="mb-2 text-sm font-medium">Validierungsfehler</h4>
        <InlineAlert description="Bitte füllen Sie alle Pflichtfelder aus." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Systemwarnung</h4>
        <InlineAlert description="Die Verbindung zum Server konnte nicht hergestellt werden." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Kritischer Zustand</h4>
        <InlineAlert description="Der Wasserstand ist kritisch niedrig. Sofortige Bewässerung erforderlich." />
      </div>
    </div>
  ),
}

export const InFormContext: Story = {
  render: () => (
    <div className="max-w-md space-y-4 rounded-xl border p-6">
      <h3 className="text-lg font-semibold">Baum bearbeiten</h3>
      <InlineAlert description="Dieser Baum hat seit 14 Tagen keine Bewässerung erhalten." />
      <div className="space-y-2">
        <label className="text-sm font-medium">Baumart</label>
        <input
          type="text"
          className="w-full rounded-md border px-3 py-2"
          defaultValue="Eiche"
        />
      </div>
    </div>
  ),
}
