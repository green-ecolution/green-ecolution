import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  InlineAlert,
} from '../src/components/ui/alert'

const meta: Meta<typeof Alert> = {
  title: 'UI/Alert',
  component: Alert,
  tags: ['autodocs'],
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'info', 'destructive', 'warning', 'success'],
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
    <Alert className="flex gap-4">
      <AlertIcon variant="default" />
      <AlertContent>
        <AlertTitle>Information</AlertTitle>
        <AlertDescription>Dies ist eine informative Hinweismeldung.</AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const Info: Story = {
  render: () => (
    <Alert variant="info" className="flex gap-4">
      <AlertIcon variant="info" />
      <AlertContent>
        <AlertTitle>Hinweis</AlertTitle>
        <AlertDescription>
          Dieser Standort wird regelmäßig überwacht und ausgewertet.
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const Destructive: Story = {
  render: () => (
    <Alert variant="destructive" className="flex gap-4">
      <AlertIcon variant="destructive" />
      <AlertContent>
        <AlertTitle>Fehler</AlertTitle>
        <AlertDescription>
          Ein Fehler ist aufgetreten. Bitte versuche es später erneut.
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const Warning: Story = {
  render: () => (
    <Alert variant="warning" className="flex gap-4">
      <AlertIcon variant="warning" />
      <AlertContent>
        <AlertTitle>Warnung</AlertTitle>
        <AlertDescription>Bitte überprüfe deine Eingaben, bevor du fortfährst.</AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const Success: Story = {
  render: () => (
    <Alert variant="success" className="flex gap-4">
      <AlertIcon variant="success" />
      <AlertContent>
        <AlertTitle>Erfolg</AlertTitle>
        <AlertDescription>Deine Änderungen wurden erfolgreich gespeichert.</AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const AllVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <Alert className="flex gap-4">
        <AlertIcon variant="default" />
        <AlertContent>
          <AlertTitle>Standard</AlertTitle>
          <AlertDescription>Standard-Hinweismeldung ohne spezielle Bedeutung.</AlertDescription>
        </AlertContent>
      </Alert>
      <Alert variant="info" className="flex gap-4">
        <AlertIcon variant="info" />
        <AlertContent>
          <AlertTitle>Information</AlertTitle>
          <AlertDescription>Informative Hinweismeldung für den Benutzer.</AlertDescription>
        </AlertContent>
      </Alert>
      <Alert variant="success" className="flex gap-4">
        <AlertIcon variant="success" />
        <AlertContent>
          <AlertTitle>Erfolg</AlertTitle>
          <AlertDescription>Aktion wurde erfolgreich durchgeführt.</AlertDescription>
        </AlertContent>
      </Alert>
      <Alert variant="warning" className="flex gap-4">
        <AlertIcon variant="warning" />
        <AlertContent>
          <AlertTitle>Warnung</AlertTitle>
          <AlertDescription>Achtung erforderlich, mögliche Probleme.</AlertDescription>
        </AlertContent>
      </Alert>
      <Alert variant="destructive" className="flex gap-4">
        <AlertIcon variant="destructive" />
        <AlertContent>
          <AlertTitle>Fehler</AlertTitle>
          <AlertDescription>Ein kritischer Fehler ist aufgetreten.</AlertDescription>
        </AlertContent>
      </Alert>
    </div>
  ),
}

export const SensorNotice: Story = {
  render: () => (
    <Alert variant="info" className="flex gap-4">
      <AlertIcon variant="info" />
      <AlertContent>
        <AlertTitle>Dieser Baum ist nicht mit einem Sensor ausgestattet</AlertTitle>
        <AlertDescription>
          Es können keine Informationen über den aktuellen Bewässerungszustand angezeigt werden. Der
          Bewässerungszustand wird als unbekannt ausgezeichnet.
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const WateringAlert: Story = {
  render: () => (
    <Alert variant="warning" className="flex gap-4">
      <AlertIcon variant="warning" />
      <AlertContent>
        <AlertTitle>Bewässerung erforderlich</AlertTitle>
        <AlertDescription>
          Die Bodenfeuchtigkeit liegt unter dem kritischen Schwellenwert. Bitte plane eine
          Bewässerung für diesen Standort ein.
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const RouteCompleted: Story = {
  render: () => (
    <Alert variant="success" className="flex gap-4">
      <AlertIcon variant="success" />
      <AlertContent>
        <AlertTitle>Route abgeschlossen</AlertTitle>
        <AlertDescription>
          Die Bewässerungsroute wurde erfolgreich abgeschlossen. Alle 24 Bäume wurden bewässert.
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}

export const InlineAlertExample: Story = {
  render: () => <InlineAlert description="Bitte füllen Sie alle Pflichtfelder aus." />,
}

export const InlineAlertVariants: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="mb-2 text-sm font-medium text-muted-foreground">Standard</h4>
        <InlineAlert variant="default" description="Eine neutrale Inline-Meldung." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium text-muted-foreground">Information</h4>
        <InlineAlert variant="info" description="Eine informative Hinweismeldung." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium text-muted-foreground">Erfolg</h4>
        <InlineAlert variant="success" description="Die Aktion war erfolgreich." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium text-muted-foreground">Warnung</h4>
        <InlineAlert variant="warning" description="Bitte überprüfe deine Eingaben." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium text-muted-foreground">Fehler</h4>
        <InlineAlert variant="destructive" description="Bitte füllen Sie alle Pflichtfelder aus." />
      </div>
    </div>
  ),
}

export const InlineAlertUseCases: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="mb-2 text-sm font-medium">Validierungsfehler</h4>
        <InlineAlert description="Bitte füllen Sie alle Pflichtfelder aus." />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Systemwarnung</h4>
        <InlineAlert
          variant="warning"
          description="Die Verbindung zum Server konnte nicht hergestellt werden."
        />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Kritischer Zustand</h4>
        <InlineAlert
          variant="destructive"
          description="Der Wasserstand ist kritisch niedrig. Sofortige Bewässerung erforderlich."
        />
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Erfolgsmeldung</h4>
        <InlineAlert variant="success" description="Sensor erfolgreich verbunden." />
      </div>
    </div>
  ),
}

export const InFormContext: Story = {
  render: () => (
    <div className="max-w-md space-y-4 rounded-xl border border-dark-100 p-6">
      <h3 className="font-lato text-lg font-semibold">Baum bearbeiten</h3>
      <InlineAlert
        variant="warning"
        description="Dieser Baum hat seit 14 Tagen keine Bewässerung erhalten."
      />
      <div className="space-y-2">
        <label className="text-sm font-medium">Baumart</label>
        <input
          type="text"
          className="w-full rounded-lg border border-dark-200 px-3 py-2 text-sm focus:border-green-dark focus:outline-none focus:ring-1 focus:ring-green-dark"
          defaultValue="Eiche"
        />
      </div>
      <div className="space-y-2">
        <label className="text-sm font-medium">Standort</label>
        <input
          type="text"
          className="w-full rounded-lg border border-dark-200 px-3 py-2 text-sm focus:border-green-dark focus:outline-none focus:ring-1 focus:ring-green-dark"
          defaultValue="Hauptstraße 42"
        />
      </div>
    </div>
  ),
}

export const WithAriaLive: Story = {
  render: () => (
    <Alert variant="success" aria-live="polite" className="flex gap-4">
      <AlertIcon variant="success" />
      <AlertContent>
        <AlertTitle>Speichern erfolgreich</AlertTitle>
        <AlertDescription>
          Diese Meldung wird von Screenreadern automatisch vorgelesen (aria-live="polite").
        </AlertDescription>
      </AlertContent>
    </Alert>
  ),
}
