import type { Meta, StoryObj } from '@storybook/react-vite'
import { Info, Settings, UserPlus } from 'lucide-react'
import { Button } from '../src/components/ui/button'
import { Input } from '../src/components/ui/input'
import { Label } from '../src/components/ui/label'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogIcon,
  DialogTitle,
  DialogTrigger,
} from '../src/components/ui/dialog'

const meta: Meta<typeof Dialog> = {
  title: 'UI/Dialog',
  component: Dialog,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">Dialog öffnen</Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Profil bearbeiten</DialogTitle>
          <DialogDescription>
            Nimm hier Änderungen an deinem Profil vor. Klick auf Speichern, wenn du fertig bist.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="dialog-name">Name</Label>
            <Input id="dialog-name" defaultValue="Max Mustermann" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="dialog-email">E-Mail</Label>
            <Input id="dialog-email" defaultValue="max@beispiel.de" />
          </div>
        </div>
        <DialogFooter>
          <Button type="submit">Speichern</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const WithIcon: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">
          <Settings className="mr-2 h-4 w-4" />
          Einstellungen
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogIcon>
          <Settings />
        </DialogIcon>
        <DialogHeader className="text-center sm:text-center">
          <DialogTitle>Einstellungen</DialogTitle>
          <DialogDescription>
            Pass hier deine Anwendungseinstellungen an.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="settings-theme">Theme</Label>
            <Input id="settings-theme" defaultValue="Hell" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="settings-lang">Sprache</Label>
            <Input id="settings-lang" defaultValue="Deutsch" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline">Abbrechen</Button>
          <Button type="submit">Speichern</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const SimpleInfo: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button variant="outline">Info anzeigen</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Information</DialogTitle>
          <DialogDescription>
            Dies ist ein einfacher informativer Dialog ohne Icon.
          </DialogDescription>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          Hier können zusätzliche Informationen angezeigt werden, die für dich wichtig sind.
        </p>
        <DialogFooter>
          <Button>Verstanden</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const InfoDialog: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button>Info anzeigen</Button>
      </DialogTrigger>
      <DialogContent>
        <DialogIcon className="bg-blue-100 [&>svg]:text-blue-600">
          <Info />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>Information</DialogTitle>
          <DialogDescription>
            Dies ist ein informativer Dialog mit Icon. Der Text ist linksbündig für bessere Lesbarkeit.
          </DialogDescription>
        </DialogHeader>
        <p className="text-sm text-muted-foreground">
          Hier können zusätzliche Informationen angezeigt werden, die für dich wichtig sind.
        </p>
        <DialogFooter>
          <Button>Verstanden</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}

export const AddUserDialog: Story = {
  render: () => (
    <Dialog>
      <DialogTrigger asChild>
        <Button>
          <UserPlus className="mr-2 h-4 w-4" />
          Benutzer hinzufügen
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[500px]">
        <DialogIcon className="bg-green-dark/10 [&>svg]:text-green-dark">
          <UserPlus />
        </DialogIcon>
        <DialogHeader className="text-center sm:text-center">
          <DialogTitle>Neuen Benutzer hinzufügen</DialogTitle>
          <DialogDescription>
            Gib die Daten des neuen Benutzers ein.
          </DialogDescription>
        </DialogHeader>
        <div className="grid gap-4 py-4">
          <div className="grid gap-2">
            <Label htmlFor="add-firstname">Vorname</Label>
            <Input id="add-firstname" placeholder="Max" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="add-lastname">Nachname</Label>
            <Input id="add-lastname" placeholder="Mustermann" />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="add-email">E-Mail</Label>
            <Input id="add-email" type="email" placeholder="max@beispiel.de" />
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline">Abbrechen</Button>
          <Button type="submit">Hinzufügen</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  ),
}
