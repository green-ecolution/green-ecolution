import type { Meta, StoryObj } from '@storybook/react'
import { Separator } from '../src/components/ui/separator'

const meta: Meta<typeof Separator> = {
  title: 'UI/Separator',
  component: Separator,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Horizontal: Story = {
  render: () => (
    <div className="space-y-4">
      <div>
        <h4 className="text-sm font-medium">Bauminformationen</h4>
        <p className="text-sm text-muted-foreground">
          Grundlegende Details zum ausgewählten Baum.
        </p>
      </div>
      <Separator />
      <div>
        <h4 className="text-sm font-medium">Sensordaten</h4>
        <p className="text-sm text-muted-foreground">
          Live-Messwerte der verbundenen Sensoren.
        </p>
      </div>
    </div>
  ),
}

export const Vertical: Story = {
  render: () => (
    <div className="flex h-5 items-center space-x-4 text-sm">
      <div>Bäume</div>
      <Separator orientation="vertical" />
      <div>Fahrzeuge</div>
      <Separator orientation="vertical" />
      <div>Sensoren</div>
      <Separator orientation="vertical" />
      <div>Pläne</div>
    </div>
  ),
}

export const InCard: Story = {
  render: () => (
    <div className="rounded-lg border p-4">
      <div className="space-y-1">
        <h4 className="text-sm font-medium leading-none">Eiche #1234</h4>
        <p className="text-sm text-muted-foreground">Hauptstraße 42</p>
      </div>
      <Separator className="my-4" />
      <div className="flex h-5 items-center space-x-4 text-sm">
        <div>Status: Gesund</div>
        <Separator orientation="vertical" />
        <div>Zuletzt bewässert: vor 3 Tagen</div>
      </div>
    </div>
  ),
}

export const FormSections: Story = {
  render: () => (
    <div className="space-y-6">
      <div className="space-y-2">
        <h3 className="text-lg font-medium">Allgemeine Informationen</h3>
        <p className="text-sm text-muted-foreground">
          Geben Sie die grundlegenden Informationen zum Baum ein.
        </p>
        <div className="h-10 rounded bg-muted" />
        <div className="h-10 rounded bg-muted" />
      </div>

      <Separator />

      <div className="space-y-2">
        <h3 className="text-lg font-medium">Standort</h3>
        <p className="text-sm text-muted-foreground">
          Geben Sie den Standort des Baums an.
        </p>
        <div className="h-10 rounded bg-muted" />
      </div>

      <Separator />

      <div className="space-y-2">
        <h3 className="text-lg font-medium">Zusätzliche Details</h3>
        <p className="text-sm text-muted-foreground">
          Optionale Informationen zum Baum.
        </p>
        <div className="h-20 rounded bg-muted" />
      </div>
    </div>
  ),
}

export const MenuDivider: Story = {
  render: () => (
    <div className="w-48 rounded-lg border p-2">
      <div className="px-2 py-1.5 text-sm">Dashboard</div>
      <div className="px-2 py-1.5 text-sm">Bäume</div>
      <div className="px-2 py-1.5 text-sm">Fahrzeuge</div>
      <Separator className="my-2" />
      <div className="px-2 py-1.5 text-sm">Einstellungen</div>
      <div className="px-2 py-1.5 text-sm">Hilfe</div>
      <Separator className="my-2" />
      <div className="px-2 py-1.5 text-sm text-destructive">Abmelden</div>
    </div>
  ),
}
