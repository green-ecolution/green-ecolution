import type { Meta, StoryObj } from '@storybook/react'
import { MoveRight, Bell, Droplets, TreeDeciduous, AlertTriangle } from 'lucide-react'
import { Button } from '../src/components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '../src/components/ui/card'
import { Input } from '../src/components/ui/input'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Card> = {
  title: 'UI/Card',
  component: Card,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Card className="w-[350px]">
      <CardHeader>
        <CardTitle>Kartentitel</CardTitle>
        <CardDescription>Hier kommt die Beschreibung.</CardDescription>
      </CardHeader>
      <CardContent>
        <p>Hier kommt der Inhalt.</p>
      </CardContent>
      <CardFooter>
        <Button>Aktion</Button>
      </CardFooter>
    </Card>
  ),
}

export const WithForm: Story = {
  render: () => (
    <Card className="w-[350px]">
      <CardHeader>
        <CardTitle>Konto erstellen</CardTitle>
        <CardDescription>
          Gib deine Daten ein, um ein neues Konto zu erstellen.
        </CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="grid w-full items-center gap-4">
            <div className="flex flex-col space-y-1.5">
              <Label htmlFor="name">Name</Label>
              <Input id="name" placeholder="Name" />
            </div>
            <div className="flex flex-col space-y-1.5">
              <Label htmlFor="email">E-Mail</Label>
              <Input id="email" type="email" placeholder="E-Mail" />
            </div>
          </div>
        </form>
      </CardContent>
      <CardFooter className="flex justify-between">
        <Button variant="outline">Abbrechen</Button>
        <Button>Erstellen</Button>
      </CardFooter>
    </Card>
  ),
}

export const NotificationCard: Story = {
  render: () => (
    <Card className="w-[380px]">
      <CardHeader className="pb-3">
        <div className="flex items-center gap-3">
          <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-green-dark-50">
            <Bell className="h-5 w-5 text-green-dark" />
          </div>
          <div>
            <CardTitle className="text-base">Benachrichtigungen</CardTitle>
            <CardDescription>3 ungelesene Meldungen</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        <div className="flex items-start gap-3 p-3 rounded-lg bg-dark-50">
          <Droplets className="h-5 w-5 text-green-light mt-0.5" />
          <div>
            <p className="text-sm font-medium">Bewässerung abgeschlossen</p>
            <p className="text-xs text-muted-foreground">Gruppe Nord-West wurde bewässert</p>
          </div>
        </div>
        <div className="flex items-start gap-3 p-3 rounded-lg bg-dark-50">
          <TreeDeciduous className="h-5 w-5 text-green-dark mt-0.5" />
          <div>
            <p className="text-sm font-medium">Neuer Baum hinzugefügt</p>
            <p className="text-xs text-muted-foreground">Eiche #1234 wurde registriert</p>
          </div>
        </div>
        <div className="flex items-start gap-3 p-3 rounded-lg bg-yellow-50">
          <AlertTriangle className="h-5 w-5 text-yellow mt-0.5" />
          <div>
            <p className="text-sm font-medium">Sensor-Warnung</p>
            <p className="text-xs text-muted-foreground">Sensor #42 meldet niedrigen Akkustand</p>
          </div>
        </div>
      </CardContent>
    </Card>
  ),
}

export const StatusCard: Story = {
  render: () => (
    <Card className="w-[300px] border-0 bg-dark-50">
      <CardContent className="p-6 space-y-3">
        <p className="text-sm text-dark-700 font-medium">Baumanzahl in der Gruppe</p>
        <p className="font-bold text-xl">8 Bäume</p>
        <p className="text-sm">Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind.</p>
      </CardContent>
    </Card>
  ),
}

export const StatusCardWithIndicator: Story = {
  render: () => (
    <Card className="w-[300px] border-0 bg-green-light-100">
      <CardContent className="p-6 space-y-3">
        <p className="text-sm text-dark-700 font-medium">Bewässerungszustand (ø)</p>
        <p className="font-bold text-xl pl-7 relative before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-1 before:bg-green-light">
          In Ordnung
        </p>
        <p className="text-sm">Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich.</p>
      </CardContent>
    </Card>
  ),
}

export const StatusCardsGrid: Story = {
  render: () => (
    <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4">
      <Card className="border-0 bg-green-light-100">
        <CardContent className="p-6 space-y-3">
          <p className="text-sm text-dark-700 font-medium">Bewässerungszustand (ø)</p>
          <p className="font-bold text-xl pl-7 relative before:absolute before:w-4 before:h-4 before:rounded-full before:left-0 before:top-1 before:bg-green-light">
            In Ordnung
          </p>
          <p className="text-sm">Die Bewässerung ist ausreichend, keine Maßnahmen erforderlich.</p>
        </CardContent>
      </Card>
      <Card className="border-0 bg-dark-50">
        <CardContent className="p-6 space-y-3">
          <p className="text-sm text-dark-700 font-medium">Baumanzahl in der Gruppe</p>
          <p className="font-bold text-xl">8 Bäume</p>
          <p className="text-sm">Nicht alle Bäume haben Sensoren, da Rückschlüsse möglich sind.</p>
        </CardContent>
      </Card>
      <Card className="border-0 bg-dark-50">
        <CardContent className="p-6 space-y-3">
          <p className="text-sm text-dark-700 font-medium">Standort der Gruppe</p>
          <p className="font-bold text-xl">Alte Zob-Brücke, Weiche</p>
        </CardContent>
      </Card>
      <Card className="border-0 bg-dark-50">
        <CardContent className="p-6 space-y-3">
          <p className="text-sm text-dark-700 font-medium">Datum der letzten Bewässerung</p>
          <p className="font-bold text-xl">Keine Angabe</p>
          <p className="text-sm">Wird aktualisiert, sobald ein Einsatzplan mit dieser Gruppe als »Beendet« markiert wird.</p>
        </CardContent>
      </Card>
    </div>
  ),
}

export const DashboardLinkCard: Story = {
  render: () => (
    <div className="grid gap-4 md:grid-cols-3">
      <Card className="h-full flex flex-col gap-4 p-6 border-green-dark bg-green-dark-50 hover:bg-green-dark-100 transition-all duration-300 cursor-pointer group">
        <h3 className="font-lato text-lg text-dark font-semibold">Bewässerungsgruppen</h3>
        <p>Verwalte die Bewässerungsgruppen und deren Bäume.</p>
        <p className="mt-auto font-lato font-semibold text-green-dark flex items-center gap-x-2">
          <span>Alle Gruppen</span>
          <MoveRight className="transition-all duration-300 group-hover:translate-x-2" />
        </p>
      </Card>
      <Card className="h-full flex flex-col gap-4 p-6 border-green-light bg-green-light-50 hover:bg-green-light-100 transition-all duration-300 cursor-pointer group">
        <h3 className="font-lato text-lg text-dark font-semibold">Bäume</h3>
        <p>Alle Bäume im System anzeigen und verwalten.</p>
        <p className="mt-auto font-lato font-semibold text-green-dark flex items-center gap-x-2">
          <span>Alle Bäume</span>
          <MoveRight className="transition-all duration-300 group-hover:translate-x-2" />
        </p>
      </Card>
      <Card className="h-full flex flex-col gap-4 p-6 border-dark-50 bg-white hover:bg-dark-100 transition-all duration-300 cursor-pointer group">
        <h3 className="font-lato text-lg text-dark font-semibold">Einsatzpläne</h3>
        <p>Erstelle und verwalte Einsatzpläne für die Bewässerung.</p>
        <p className="mt-auto font-lato font-semibold text-green-dark flex items-center gap-x-2">
          <span>Alle Pläne</span>
          <MoveRight className="transition-all duration-300 group-hover:translate-x-2" />
        </p>
      </Card>
    </div>
  ),
}
