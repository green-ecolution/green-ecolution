import type { Meta, StoryObj } from '@storybook/react'
import {
  ListCard,
  ListCardHeader,
  ListCardStatus,
  ListCardTitle,
  ListCardDescription,
  ListCardMeta,
  ListCardContent,
  ListCardActions,
} from '../src/components/ui/list-card'
import { Badge } from '../src/components/ui/badge'
import { Button } from '../src/components/ui/button'
import { MapPin, TreeDeciduous, Trash2 } from 'lucide-react'

const meta: Meta<typeof ListCard> = {
  title: 'UI/ListCard',
  component: ListCard,
  tags: ['autodocs'],
  parameters: {
    layout: 'padded',
  },
}

export default meta
type Story = StoryObj<typeof meta>

const treeclusterData = [
  {
    id: 1,
    status: 'green-light',
    statusLabel: 'Leicht trocken',
    name: 'Alsterbogen',
    address: 'Alsterbogen 60',
    region: 'Fruerlund',
    trees: 1,
  },
  {
    id: 2,
    status: 'green-dark',
    statusLabel: 'In Ordnung',
    name: 'Alte Zob-Brücke',
    address: 'Alte Zob-Brücke',
    region: 'Weiche',
    trees: 8,
  },
  {
    id: 3,
    status: 'yellow',
    statusLabel: 'Mäßig trocken',
    name: 'Nordpark',
    address: 'Nordstraße 12',
    region: 'Zentrum',
    trees: 15,
  },
]

const treeData = [
  {
    id: 1,
    status: 'green-dark',
    statusLabel: 'In Ordnung',
    species: 'Quercus robur',
    number: '1001',
    cluster: 'Alsterbogen',
  },
  {
    id: 2,
    status: 'green-light',
    statusLabel: 'Leicht trocken',
    species: 'Tilia cordata',
    number: '1002',
    cluster: 'Alte Zob-Brücke',
  },
  {
    id: 3,
    status: 'yellow',
    statusLabel: 'Mäßig trocken',
    species: 'Acer platanoides',
    number: '1003',
    cluster: 'Nordpark',
  },
  {
    id: 4,
    status: 'red',
    statusLabel: 'Kritisch',
    species: 'Fagus sylvatica',
    number: '1004',
    cluster: 'Nicht zugeordnet',
  },
]

const vehicleData = [
  {
    id: 1,
    status: 'Verfügbar',
    plate: 'FL-GE 123',
    capacity: '10.000 Liter',
    model: 'MAN TGS',
    license: 'C',
  },
  {
    id: 2,
    status: 'Im Einsatz',
    plate: 'FL-GE 456',
    capacity: '3.000 Liter',
    model: 'Mercedes Sprinter',
    license: 'B',
  },
  {
    id: 3,
    status: 'Wartung',
    plate: 'FL-GE 789',
    capacity: '5.000 Liter',
    model: 'Anhänger',
    license: 'BE',
  },
]

export const TreeclusterList: Story = {
  render: () => (
    <div>
      <ListCardHeader columns="1fr 2fr 1.5fr 1fr">
        <p>Status</p>
        <p>Name</p>
        <p>Standort</p>
        <p>Anzahl d. Bäume</p>
      </ListCardHeader>
      <ul className="flex flex-col gap-y-5">
        {treeclusterData.map((cluster) => (
          <li key={cluster.id}>
            <ListCard columns="1fr 2fr 1.5fr 1fr">
              <ListCardStatus status={cluster.status}>{cluster.statusLabel}</ListCardStatus>
              <ListCardTitle>{cluster.name}</ListCardTitle>
              <ListCardMeta>
                <MapPin className="w-5 h-5" />
                <p>
                  <span>{cluster.address}, </span>
                  <br />
                  <span className="text-dark-600 lg:block lg:text-sm">{cluster.region}</span>
                </p>
              </ListCardMeta>
              <ListCardMeta>
                <TreeDeciduous className="w-5 h-5" />
                <p>
                  {cluster.trees} {cluster.trees === 1 ? 'Baum' : 'Bäume'}
                </p>
              </ListCardMeta>
            </ListCard>
          </li>
        ))}
      </ul>
    </div>
  ),
}

export const TreeList: Story = {
  render: () => (
    <div>
      <ListCardHeader columns="1fr 1.5fr 1fr 1fr">
        <p>Status</p>
        <p>Baumart</p>
        <p>Baumnummer</p>
        <p>Bewässerungsgruppe</p>
      </ListCardHeader>
      <ul className="flex flex-col gap-y-5">
        {treeData.map((tree) => (
          <li key={tree.id}>
            <ListCard columns="1fr 1.5fr 1fr 1fr">
              <ListCardStatus status={tree.status}>{tree.statusLabel}</ListCardStatus>
              <ListCardTitle>{tree.species}</ListCardTitle>
              <ListCardDescription>
                <span className="lg:sr-only">Baumnummer: </span>
                {tree.number}
              </ListCardDescription>
              <ListCardDescription>
                <span className="lg:sr-only">Bewässerungsgruppe: </span>
                {tree.cluster}
              </ListCardDescription>
            </ListCard>
          </li>
        ))}
      </ul>
    </div>
  ),
}

export const VehicleListWithBadges: Story = {
  render: () => (
    <div>
      <ListCardHeader columns="repeat(5, 1fr)">
        <p>Status</p>
        <p>Kennzeichen</p>
        <p>Wasserkapazität</p>
        <p>Modell</p>
        <p>Führerschein</p>
      </ListCardHeader>
      <ul className="flex flex-col gap-y-5">
        {vehicleData.map((vehicle) => (
          <li key={vehicle.id}>
            <ListCard columns="repeat(5, 1fr)">
              <div>
                <Badge
                  variant={
                    vehicle.status === 'Verfügbar'
                      ? 'outline-green-dark'
                      : vehicle.status === 'Im Einsatz'
                        ? 'outline-yellow'
                        : 'outline-red'
                  }
                  size="lg"
                >
                  {vehicle.status}
                </Badge>
              </div>
              <div>
                <ListCardTitle className="text-base">{vehicle.plate}</ListCardTitle>
                <p className="text-dark-600 text-sm">{vehicle.model}</p>
              </div>
              <ListCardDescription>{vehicle.capacity}</ListCardDescription>
              <ListCardDescription>{vehicle.model}</ListCardDescription>
              <ListCardDescription>{vehicle.license}</ListCardDescription>
            </ListCard>
          </li>
        ))}
      </ul>
    </div>
  ),
}

export const WateringPlanList: Story = {
  render: () => {
    const wateringPlans = [
      {
        id: 1,
        status: 'Geplant',
        date: '15.01.2026',
        vehicle: 'FL-GE 123',
        distance: '12.5 km',
        staff: 2,
        clusters: 5,
      },
      {
        id: 2,
        status: 'Aktiv',
        date: '14.01.2026',
        vehicle: 'FL-GE 456',
        distance: '8.3 km',
        staff: 3,
        clusters: 3,
      },
      {
        id: 3,
        status: 'Abgeschlossen',
        date: '13.01.2026',
        vehicle: 'FL-GE 123',
        distance: '15.2 km',
        staff: 2,
        clusters: 7,
      },
    ]

    return (
      <div>
        <ListCardHeader columns="1.3fr 1.5fr 1fr 1.5fr 1.5fr">
          <p>Status</p>
          <p>Datum & Fahrzeug</p>
          <p>Länge</p>
          <p>Mitarbeitende</p>
          <p>Bewässerungsgruppen</p>
        </ListCardHeader>
        <ul className="flex flex-col gap-y-5">
          {wateringPlans.map((plan) => (
            <li key={plan.id}>
              <ListCard columns="1.3fr 1.5fr 1fr 1.5fr 1.5fr">
                <div>
                  <Badge
                    variant={
                      plan.status === 'Geplant'
                        ? 'outline-dark'
                        : plan.status === 'Aktiv'
                          ? 'outline-yellow'
                          : 'outline-green-dark'
                    }
                    size="lg"
                  >
                    {plan.status}
                  </Badge>
                </div>
                <div>
                  <ListCardTitle className="text-base">{plan.date}</ListCardTitle>
                  <p className="text-dark-600 text-sm">Fahrzeug: {plan.vehicle}</p>
                </div>
                <ListCardDescription>{plan.distance}</ListCardDescription>
                <ListCardDescription>{plan.staff} Mitarbeitende</ListCardDescription>
                <ListCardDescription>{plan.clusters} Gruppen</ListCardDescription>
              </ListCard>
            </li>
          ))}
        </ul>
      </div>
    )
  },
}

export const SingleCard: Story = {
  render: () => (
    <ListCard columns="1fr 2fr 1.5fr 1fr">
      <ListCardStatus status="green-dark">In Ordnung</ListCardStatus>
      <ListCardTitle>Alsterbogen</ListCardTitle>
      <ListCardMeta>
        <MapPin className="w-5 h-5" />
        <p>Alsterbogen 60, Fruerlund</p>
      </ListCardMeta>
      <ListCardMeta>
        <TreeDeciduous className="w-5 h-5" />
        <p>8 Bäume</p>
      </ListCardMeta>
    </ListCard>
  ),
}

export const NonHoverable: Story = {
  render: () => (
    <ListCard columns="1fr 2fr 1fr" hoverable={false}>
      <ListCardStatus status="yellow">Mäßig trocken</ListCardStatus>
      <ListCardTitle>Nordpark</ListCardTitle>
      <ListCardDescription>15 Bäume</ListCardDescription>
    </ListCard>
  ),
}

export const CompactWithActions: Story = {
  render: () => {
    const selectedTrees = [
      { id: 124, status: 'yellow', species: 'Acer', year: 2023 },
      { id: 125, status: 'green-dark', species: 'Quercus robur', year: 2021 },
      { id: 126, status: 'red', species: 'Tilia cordata', year: 2022 },
    ]

    return (
      <div className="flex flex-col gap-y-3">
        <p className="text-sm font-medium text-dark-800">Zugehörige Bäume</p>
        {selectedTrees.map((tree) => (
          <ListCard key={tree.id} size="compact" hoverable={false}>
            <ListCardStatus status={tree.status} />
            <ListCardContent>
              <span className="font-medium">
                <strong className="font-semibold">Baum:</strong> {tree.species} · {tree.id} ·{' '}
                {tree.year}
              </span>
            </ListCardContent>
            <ListCardActions>
              <Button
                variant="ghost"
                size="icon"
                className="text-dark-600 hover:text-red"
                onClick={() => alert(`Baum ${tree.id} entfernen`)}
              >
                <Trash2 className="w-5 h-5" />
                <span className="sr-only">Baum aus Auswahl löschen</span>
              </Button>
            </ListCardActions>
          </ListCard>
        ))}
      </div>
    )
  },
}

export const CompactTreeClusterWithActions: Story = {
  render: () => {
    const selectedClusters = [
      { id: 1, status: 'green-light', name: 'Alsterbogen' },
      { id: 2, status: 'yellow', name: 'Nordpark' },
    ]

    return (
      <div className="flex flex-col gap-y-3">
        <p className="text-sm font-medium text-dark-800">Ausgewählte Bewässerungsgruppen</p>
        {selectedClusters.map((cluster) => (
          <ListCard key={cluster.id} size="compact" hoverable={false}>
            <ListCardStatus status={cluster.status} />
            <ListCardContent>
              <span className="font-medium">
                <strong className="font-semibold">Bewässerungsgruppe:</strong> {cluster.name} ·{' '}
                {cluster.id}
              </span>
            </ListCardContent>
            <ListCardActions>
              <Button
                variant="ghost"
                size="icon"
                className="text-dark-600 hover:text-red"
                onClick={() => alert(`Gruppe ${cluster.id} entfernen`)}
              >
                <Trash2 className="w-5 h-5" />
                <span className="sr-only">Gruppe aus Auswahl löschen</span>
              </Button>
            </ListCardActions>
          </ListCard>
        ))}
      </div>
    )
  },
}

export const CompactSingleCard: Story = {
  render: () => (
    <ListCard size="compact" hoverable={false}>
      <ListCardStatus status="green-dark" />
      <ListCardContent>
        <span className="font-medium">
          <strong className="font-semibold">Baum:</strong> Quercus robur · 1001 · 2020
        </span>
      </ListCardContent>
      <ListCardActions>
        <Button variant="ghost" size="icon" className="text-dark-600 hover:text-red">
          <Trash2 className="w-5 h-5" />
        </Button>
      </ListCardActions>
    </ListCard>
  ),
}
