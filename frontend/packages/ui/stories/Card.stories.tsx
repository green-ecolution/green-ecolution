import type { Meta, StoryObj } from '@storybook/react-vite'
import { Bell, Droplets, TreeDeciduous, AlertTriangle } from 'lucide-react'
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
  argTypes: {
    variant: {
      control: 'select',
      options: ['default', 'outlined'],
      description: 'Card visual style',
    },
  },
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
    <Card variant="outlined" className="w-[350px]">
      <CardHeader>
        <CardTitle>Konto erstellen</CardTitle>
        <CardDescription>Gib deine Daten ein, um ein neues Konto zu erstellen.</CardDescription>
      </CardHeader>
      <CardContent>
        <form>
          <div className="grid w-full items-center gap-4">
            <div className="flex flex-col gap-y-1.5">
              <Label htmlFor="name">Name</Label>
              <Input id="name" placeholder="Name" />
            </div>
            <div className="flex flex-col gap-y-1.5">
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
    <Card variant="outlined" className="w-[380px]">
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
      <CardContent className="flex flex-col gap-y-3">
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

export const Variants: Story = {
  render: () => (
    <div className="flex gap-4">
      <Card className="w-[250px]">
        <CardHeader>
          <CardTitle className="text-lg">Default (Filled)</CardTitle>
          <CardDescription>Für Status-Karten und Info-Boxen</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm">Subtiler Hintergrund ohne Border.</p>
        </CardContent>
      </Card>
      <Card variant="outlined" className="w-[250px]">
        <CardHeader>
          <CardTitle className="text-lg">Outlined</CardTitle>
          <CardDescription>Für Formulare und geschachtelte Inhalte</CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm">Mit Border und Shadow.</p>
        </CardContent>
      </Card>
    </div>
  ),
}

export const SemanticHeadings: Story = {
  render: () => (
    <div className="flex flex-col gap-y-4">
      <Card className="w-[350px]">
        <CardHeader>
          <CardTitle asChild>
            <h2>Als H2 Heading</h2>
          </CardTitle>
          <CardDescription>CardTitle mit asChild für semantisches HTML</CardDescription>
        </CardHeader>
        <CardContent>
          <p>Der Titel wird als echtes h2-Element gerendert.</p>
        </CardContent>
      </Card>
      <Card className="w-[350px]">
        <CardHeader>
          <CardTitle asChild>
            <h3>Als H3 Heading</h3>
          </CardTitle>
          <CardDescription>Flexibel für verschiedene Heading-Level</CardDescription>
        </CardHeader>
        <CardContent>
          <p>Der Titel wird als h3-Element gerendert.</p>
        </CardContent>
      </Card>
    </div>
  ),
}
