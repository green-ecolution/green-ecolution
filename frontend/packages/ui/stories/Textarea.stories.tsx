import type { Meta, StoryObj } from '@storybook/react'
import { Textarea } from '../src/components/ui/textarea'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Textarea> = {
  title: 'UI/Textarea',
  component: Textarea,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    placeholder: 'Gib hier deine Nachricht ein.',
  },
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="message">Deine Nachricht</Label>
      <Textarea placeholder="Gib hier deine Nachricht ein." id="message" />
    </div>
  ),
}

export const WithDescription: Story = {
  render: () => (
    <div className="grid w-full gap-1.5">
      <Label htmlFor="notes">Notizen</Label>
      <Textarea placeholder="Notizen zu diesem Baum hinzufügen..." id="notes" />
      <p className="text-sm text-muted-foreground">
        Notizen sind nur für Administratoren sichtbar.
      </p>
    </div>
  ),
}

export const Disabled: Story = {
  args: {
    placeholder: 'Dieses Textfeld ist deaktiviert.',
    disabled: true,
  },
}

export const WithDefaultValue: Story = {
  args: {
    defaultValue: 'Dieser Baum wurde 2019 im Rahmen der städtischen Begrünungsinitiative gepflanzt. Er benötigt in den Sommermonaten regelmäßige Bewässerung.',
  },
}

export const CustomRows: Story = {
  render: () => (
    <div className="space-y-4">
      <div className="grid gap-1.5">
        <Label>Klein (2 Zeilen)</Label>
        <Textarea placeholder="Kurze Eingabe..." rows={2} />
      </div>
      <div className="grid gap-1.5">
        <Label>Standard (3 Zeilen)</Label>
        <Textarea placeholder="Standard Eingabe..." rows={3} />
      </div>
      <div className="grid gap-1.5">
        <Label>Groß (6 Zeilen)</Label>
        <Textarea placeholder="Große Eingabe..." rows={6} />
      </div>
    </div>
  ),
}

export const TreeNotesExample: Story = {
  render: () => (
    <div className="max-w-md space-y-4">
      <div className="grid gap-1.5">
        <Label htmlFor="tree-notes">Baumnotizen</Label>
        <Textarea
          id="tree-notes"
          placeholder="Beobachtungen, Wartungsnotizen oder andere relevante Informationen zu diesem Baum hinzufügen..."
          rows={4}
        />
        <p className="text-sm text-muted-foreground">
          Maximal 1000 Zeichen. Zuletzt aktualisiert: Nie
        </p>
      </div>
    </div>
  ),
}

export const WateringReportExample: Story = {
  render: () => (
    <div className="max-w-md space-y-4">
      <div className="grid gap-1.5">
        <Label htmlFor="report">Bewässerungsbericht</Label>
        <Textarea
          id="report"
          placeholder="Beschreiben Sie Probleme, die während der Bewässerungsroute aufgetreten sind..."
          rows={5}
          defaultValue="Route erfolgreich abgeschlossen. Baum #456 in der Ulmenstraße hatte verstopfte Entwässerung - Wartungsteam wurde informiert. Alle anderen Bäume planmäßig bewässert."
        />
        <div className="flex justify-between text-sm text-muted-foreground">
          <span>Bitte aufgetretene Probleme genau beschreiben</span>
          <span>245/1000</span>
        </div>
      </div>
    </div>
  ),
}
