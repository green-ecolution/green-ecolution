import type { Meta, StoryObj } from '@storybook/react-vite'
import { Trees, Truck, Radio, Droplets, File, Settings } from 'lucide-react'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '../src/components/ui/tabs'

const meta: Meta<typeof Tabs> = {
  title: 'UI/Tabs',
  component: Tabs,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Tabs defaultValue="watering">
      <TabsList aria-label="Baumdaten verwalten">
        <TabsTrigger value="watering">
          <Droplets />
          Bewässerungsdaten
        </TabsTrigger>
        <TabsTrigger value="general">
          <File />
          Allgemeine Daten
        </TabsTrigger>
        <TabsTrigger value="sensors">
          <Radio />
          Sensordaten
        </TabsTrigger>
      </TabsList>
      <TabsContent value="watering">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Bewässerungsstatus</h3>
          <p className="text-muted-foreground">
            Aktuelle Informationen zum Bewässerungszustand des Baums.
          </p>
        </div>
      </TabsContent>
      <TabsContent value="general">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Allgemeine Informationen</h3>
          <p className="text-muted-foreground">
            Stammdaten und allgemeine Eigenschaften des Baums.
          </p>
        </div>
      </TabsContent>
      <TabsContent value="sensors">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Sensormesswerte</h3>
          <p className="text-muted-foreground">Live-Daten der verbundenen Sensoren.</p>
        </div>
      </TabsContent>
    </Tabs>
  ),
}

export const TreeDashboard: Story = {
  render: () => (
    <Tabs defaultValue="watering">
      <TabsList aria-label="Bauminformationen">
        <TabsTrigger value="watering">
          <Trees />
          Bewässerungsdaten
        </TabsTrigger>
        <TabsTrigger value="general">
          <File />
          Allgemeine Daten
        </TabsTrigger>
        <TabsTrigger value="sensors">
          <Radio />
          Sensordaten
        </TabsTrigger>
      </TabsList>
      <TabsContent value="watering">
        <div className="bg-white border border-dark-50 shadow-cards p-6 rounded-xl">
          <h3 className="font-lato text-lg font-semibold mb-4">Bewässerungsstatus</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-dark-600">Letztes Gießen</p>
              <p className="font-medium">vor 3 Tagen</p>
            </div>
            <div>
              <p className="text-sm text-dark-600">Bodenfeuchtigkeit</p>
              <p className="font-medium">45%</p>
            </div>
          </div>
        </div>
      </TabsContent>
      <TabsContent value="general">
        <div className="bg-white border border-dark-50 shadow-cards p-6 rounded-xl">
          <h3 className="font-lato text-lg font-semibold mb-4">Allgemeine Daten</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-dark-600">Baumart</p>
              <p className="font-medium">Eiche</p>
            </div>
            <div>
              <p className="text-sm text-dark-600">Pflanzjahr</p>
              <p className="font-medium">2018</p>
            </div>
          </div>
        </div>
      </TabsContent>
      <TabsContent value="sensors">
        <div className="bg-white border border-dark-50 shadow-cards p-6 rounded-xl">
          <h3 className="font-lato text-lg font-semibold mb-4">Sensordaten</h3>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-dark-600">Sensor-ID</p>
              <p className="font-medium">SNS-2024-0042</p>
            </div>
            <div>
              <p className="text-sm text-dark-600">Batterie</p>
              <p className="font-medium">87%</p>
            </div>
          </div>
        </div>
      </TabsContent>
    </Tabs>
  ),
}

export const VehicleTabs: Story = {
  render: () => (
    <Tabs defaultValue="info">
      <TabsList aria-label="Fahrzeuginformationen">
        <TabsTrigger value="info">
          <Truck />
          Fahrzeugdaten
        </TabsTrigger>
        <TabsTrigger value="routes">
          <Droplets />
          Bewässerungspläne
        </TabsTrigger>
        <TabsTrigger value="settings">
          <Settings />
          Einstellungen
        </TabsTrigger>
      </TabsList>
      <TabsContent value="info">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Fahrzeuginformationen</h3>
          <p className="text-muted-foreground">Details zum ausgewählten Fahrzeug.</p>
        </div>
      </TabsContent>
      <TabsContent value="routes">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Zugewiesene Routen</h3>
          <p className="text-muted-foreground">Aktive und geplante Bewässerungsrouten.</p>
        </div>
      </TabsContent>
      <TabsContent value="settings">
        <div className="flex flex-col gap-y-4">
          <h3 className="font-semibold">Fahrzeugeinstellungen</h3>
          <p className="text-muted-foreground">Konfiguration und Wartungsintervalle.</p>
        </div>
      </TabsContent>
    </Tabs>
  ),
}

export const Disabled: Story = {
  render: () => (
    <Tabs defaultValue="active">
      <TabsList aria-label="Tab-Beispiel">
        <TabsTrigger value="active">
          <File />
          Aktiv
        </TabsTrigger>
        <TabsTrigger value="disabled" disabled>
          <Radio />
          Deaktiviert
        </TabsTrigger>
      </TabsList>
      <TabsContent value="active">
        <p className="text-muted-foreground">Der zweite Tab ist deaktiviert.</p>
      </TabsContent>
    </Tabs>
  ),
}
