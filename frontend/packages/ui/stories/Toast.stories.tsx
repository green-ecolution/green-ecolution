import type { Meta, StoryObj } from '@storybook/react'
import { Button } from '../src/components/ui/button'
import { Toaster, toast } from '../src/components/ui/sonner'

const meta: Meta<typeof Toaster> = {
  title: 'UI/Toast',
  component: Toaster,
  tags: ['autodocs'],
  decorators: [
    (Story) => (
      <div className="min-h-[200px]">
        <Toaster />
        <Story />
      </div>
    ),
  ],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Button onClick={() => toast('Nachricht wurde gesendet')}>
      Toast anzeigen
    </Button>
  ),
}

export const Success: Story = {
  render: () => (
    <Button onClick={() => toast.success('Erfolgreich gespeichert!')}>
      Erfolg anzeigen
    </Button>
  ),
}

export const Error: Story = {
  render: () => (
    <Button onClick={() => toast.error('Ein Fehler ist aufgetreten')}>
      Fehler anzeigen
    </Button>
  ),
}

export const WithDescription: Story = {
  render: () => (
    <Button
      onClick={() =>
        toast('Baum aktualisiert', {
          description: 'Die Baumdaten wurden erfolgreich aktualisiert.',
        })
      }
    >
      Mit Beschreibung
    </Button>
  ),
}

export const AllVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <Button
        variant="outline"
        onClick={() => toast('Standard-Benachrichtigung')}
      >
        Standard
      </Button>
      <Button
        variant="outline"
        onClick={() => toast.success('Daten wurden gespeichert')}
      >
        Erfolg
      </Button>
      <Button
        variant="outline"
        onClick={() => toast.error('Speichern fehlgeschlagen')}
      >
        Fehler
      </Button>
      <Button
        variant="outline"
        onClick={() => toast.warning('Achtung: Niedriger Akkustand')}
      >
        Warnung
      </Button>
      <Button
        variant="outline"
        onClick={() => toast.info('Neue Updates verfügbar')}
      >
        Info
      </Button>
    </div>
  ),
}

export const ExampleUseCases: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="mb-2 text-sm font-medium">Baum-Aktionen</h4>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => toast.success('Baum wurde erfolgreich angelegt')}
          >
            Baum anlegen
          </Button>
          <Button
            variant="outline"
            onClick={() => toast.success('Baum wurde aktualisiert')}
          >
            Baum aktualisieren
          </Button>
          <Button
            variant="outline"
            onClick={() => toast.error('Baum konnte nicht gelöscht werden')}
          >
            Baum löschen
          </Button>
        </div>
      </div>
      <div>
        <h4 className="mb-2 text-sm font-medium">Bewässerungsplan</h4>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() =>
              toast.success('Bewässerungsplan wurde gespeichert')
            }
          >
            Plan speichern
          </Button>
          <Button
            variant="outline"
            onClick={() =>
              toast.error('Route konnte nicht berechnet werden')
            }
          >
            Route berechnen
          </Button>
        </div>
      </div>
    </div>
  ),
}
