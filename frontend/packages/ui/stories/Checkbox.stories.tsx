import type { Meta, StoryObj } from '@storybook/react-vite'
import { Checkbox } from '../src/components/ui/checkbox'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Checkbox> = {
  title: 'UI/Checkbox',
  component: Checkbox,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="default-terms" />
      <Label htmlFor="default-terms">Nutzungsbedingungen akzeptieren</Label>
    </div>
  ),
}

export const Checked: Story = {
  render: () => (
    <div className="flex items-center space-x-2">
      <Checkbox id="checked-example" defaultChecked />
      <Label htmlFor="checked-example">Standardmäßig aktiviert</Label>
    </div>
  ),
}

export const Disabled: Story = {
  render: () => (
    <div className="space-y-2">
      <div className="flex items-center space-x-2">
        <Checkbox id="disabled-unchecked" disabled />
        <Label htmlFor="disabled-unchecked" className="text-muted-foreground">
          Deaktiviert
        </Label>
      </div>
      <div className="flex items-center space-x-2">
        <Checkbox id="disabled-checked" disabled defaultChecked />
        <Label htmlFor="disabled-checked" className="text-muted-foreground">
          Deaktiviert & ausgewählt
        </Label>
      </div>
    </div>
  ),
}

export const FilterExample: Story = {
  render: () => (
    <div className="space-y-4">
      <h4 className="text-sm font-medium">Nach Status filtern</h4>
      <div className="space-y-2">
        <div className="flex items-center space-x-2">
          <Checkbox id="filter-healthy" defaultChecked />
          <Label htmlFor="filter-healthy">In Ordnung</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="filter-needs-water" defaultChecked />
          <Label htmlFor="filter-needs-water">Mäßig trocken</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="filter-critical" />
          <Label htmlFor="filter-critical">Kritisch</Label>
        </div>
        <div className="flex items-center space-x-2">
          <Checkbox id="filter-unknown" />
          <Label htmlFor="filter-unknown">Unbekannt</Label>
        </div>
      </div>
    </div>
  ),
}

export const WithDescription: Story = {
  render: () => (
    <div className="items-top flex space-x-2">
      <Checkbox id="desc-notifications" />
      <div className="grid gap-1.5 leading-none">
        <Label htmlFor="desc-notifications">Benachrichtigungen aktivieren</Label>
        <p className="text-sm text-muted-foreground">
          Du erhältst tägliche Updates über deine Bäume.
        </p>
      </div>
    </div>
  ),
}
