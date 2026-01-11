import type { Meta, StoryObj } from '@storybook/react-vite'
import { Bluetooth, Trash2, LogOut, MoveRight, X } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogIcon,
  AlertDialogTitle,
  AlertDialogTrigger,
} from '../src/components/ui/alert-dialog'
import { Button } from '../src/components/ui/button'

const meta: Meta<typeof AlertDialog> = {
  title: 'UI/AlertDialog',
  component: AlertDialog,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="outline">Dialog öffnen</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Bist du sicher?</AlertDialogTitle>
          <AlertDialogDescription>
            Diese Aktion kann nicht rückgängig gemacht werden.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const DeleteTree: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="destructive">Baum löschen</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Soll der Baum wirklich gelöscht werden?</AlertDialogTitle>
          <AlertDialogDescription>
            Sobald der Baum gelöscht wurde, können die Daten nicht wieder hergestellt werden.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const DeleteWateringPlan: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="destructive" size="sm">Bewässerungsgruppe löschen</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Soll die Bewässerungsgruppe wirklich gelöscht werden?</AlertDialogTitle>
          <AlertDialogDescription>
            Sobald die Bewässerungsgruppe gelöscht wurde, können die Daten nicht wieder hergestellt werden.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const ConfirmAction: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button>Einsatz starten</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Soll der Einsatz wirklich gestartet werden?</AlertDialogTitle>
          <AlertDialogDescription>
            Der Einsatzplan enthält 15 Bäume. Die geschätzte Dauer beträgt 2 Stunden 30 Minuten.
            Stelle sicher, dass der Tank des Fahrzeugs voll ist.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const LogoutConfirm: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="ghost">Abmelden</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Möchtest du dich wirklich abmelden?</AlertDialogTitle>
          <AlertDialogDescription>
            Du musst dich erneut anmelden, um auf das System zugreifen zu können.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Abmelden
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const UnsavedChanges: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="outline">Seite verlassen</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Möchtest du die Seite wirklich verlassen?</AlertDialogTitle>
          <AlertDialogDescription>
            Deine Änderungen gehen verloren, wenn du jetzt gehst.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-start">
          <AlertDialogAction>
            Verlassen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Auf Seite bleiben
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const WithIcon: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="outline">Gerät verbinden</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogIcon>
          <Bluetooth />
        </AlertDialogIcon>
        <AlertDialogHeader className="text-center sm:text-center">
          <AlertDialogTitle>Gerät verbinden?</AlertDialogTitle>
          <AlertDialogDescription>
            Möchtest du die Verbindung mit diesem Gerät herstellen?
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-center">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const DeleteWithIcon: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="destructive">Bewässerungsgruppe löschen</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogIcon className="bg-red-50 [&>svg]:text-destructive">
          <Trash2 />
        </AlertDialogIcon>
        <AlertDialogHeader className="text-center sm:text-center">
          <AlertDialogTitle>Soll die Bewässerungsgruppe wirklich gelöscht werden?</AlertDialogTitle>
          <AlertDialogDescription>
            Sobald die Bewässerungsgruppe gelöscht wurde, können die Daten nicht wieder hergestellt werden.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-center">
          <AlertDialogAction>
            Bestätigen
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}

export const LogoutWithIcon: Story = {
  render: () => (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button variant="ghost">Abmelden</Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogIcon>
          <LogOut />
        </AlertDialogIcon>
        <AlertDialogHeader className="text-center sm:text-center">
          <AlertDialogTitle>Möchtest du dich wirklich abmelden?</AlertDialogTitle>
          <AlertDialogDescription>
            Du musst dich erneut anmelden, um auf das System zugreifen zu können.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter className="sm:justify-center">
          <AlertDialogAction>
            Abmelden
            <MoveRight className="transition-all ease-in-out duration-300 group-hover:translate-x-2" />
          </AlertDialogAction>
          <AlertDialogCancel>
            Abbrechen
            <X />
          </AlertDialogCancel>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  ),
}
