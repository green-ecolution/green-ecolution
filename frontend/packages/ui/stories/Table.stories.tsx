import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableFooter,
  TableHead,
  TableHeader,
  TableRow,
} from '../src/components/ui/table'
import { Badge } from '../src/components/ui/badge'

const meta: Meta<typeof Table> = {
  title: 'UI/Table',
  component: Table,
  tags: ['autodocs'],
  parameters: {
    layout: 'padded',
  },
}

export default meta
type Story = StoryObj<typeof meta>

const treeData = [
  { id: 1, status: 'In Ordnung', species: 'Quercus robur', number: '1001', cluster: 'Alsterbogen' },
  {
    id: 2,
    status: 'Leicht trocken',
    species: 'Tilia cordata',
    number: '1002',
    cluster: 'Alte Zob-Brücke',
  },
  {
    id: 3,
    status: 'Unbekannt',
    species: 'Acer platanoides',
    number: '1003',
    cluster: 'Nicht zugeordnet',
  },
  { id: 4, status: 'Kritisch', species: 'Fagus sylvatica', number: '1004', cluster: 'Nordpark' },
]

const clusterData = [
  {
    id: 1,
    status: 'green-dark',
    statusLabel: 'In Ordnung',
    name: 'Alsterbogen',
    address: 'Alsterbogen 60, Fruerlund',
    trees: 1,
  },
  {
    id: 2,
    status: 'green-light',
    statusLabel: 'Leicht trocken',
    name: 'Alte Zob-Brücke',
    address: 'Alte Zob-Brücke, Weiche',
    trees: 8,
  },
  {
    id: 3,
    status: 'yellow',
    statusLabel: 'Mäßig trocken',
    name: 'Nordpark',
    address: 'Nordstraße 12, Zentrum',
    trees: 15,
  },
]

const invoiceData = [
  { invoice: 'INV001', status: 'Bezahlt', method: 'Überweisung', amount: '€250.00' },
  { invoice: 'INV002', status: 'Ausstehend', method: 'PayPal', amount: '€150.00' },
  { invoice: 'INV003', status: 'Unbezahlt', method: 'Rechnung', amount: '€350.00' },
  { invoice: 'INV004', status: 'Bezahlt', method: 'Überweisung', amount: '€450.00' },
  { invoice: 'INV005', status: 'Bezahlt', method: 'Kreditkarte', amount: '€550.00' },
]

export const Default: Story = {
  render: () => (
    <Table>
      <TableCaption>Eine Liste der letzten Rechnungen.</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead className="w-[100px]">Rechnung</TableHead>
          <TableHead>Status</TableHead>
          <TableHead>Zahlungsmethode</TableHead>
          <TableHead className="text-right">Betrag</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {invoiceData.map((invoice) => (
          <TableRow key={invoice.invoice}>
            <TableCell className="font-medium">{invoice.invoice}</TableCell>
            <TableCell>{invoice.status}</TableCell>
            <TableCell>{invoice.method}</TableCell>
            <TableCell className="text-right">{invoice.amount}</TableCell>
          </TableRow>
        ))}
      </TableBody>
      <TableFooter>
        <TableRow>
          <TableCell colSpan={3}>Gesamt</TableCell>
          <TableCell className="text-right">€1,750.00</TableCell>
        </TableRow>
      </TableFooter>
    </Table>
  ),
}

export const WithTreeData: Story = {
  render: () => (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Status</TableHead>
          <TableHead>Baumart</TableHead>
          <TableHead>Baumnummer</TableHead>
          <TableHead>Bewässerungsgruppe</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {treeData.map((tree) => (
          <TableRow key={tree.id}>
            <TableCell>{tree.status}</TableCell>
            <TableCell className="font-medium">{tree.species}</TableCell>
            <TableCell>{tree.number}</TableCell>
            <TableCell>{tree.cluster}</TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  ),
}

export const WithBadges: Story = {
  render: () => (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Status</TableHead>
          <TableHead>Name</TableHead>
          <TableHead>Standort</TableHead>
          <TableHead className="text-right">Anzahl d. Bäume</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {clusterData.map((cluster) => (
          <TableRow key={cluster.id}>
            <TableCell>
              <Badge variant={`outline-${cluster.status}` as const} size="lg">
                {cluster.statusLabel}
              </Badge>
            </TableCell>
            <TableCell className="font-bold">{cluster.name}</TableCell>
            <TableCell>{cluster.address}</TableCell>
            <TableCell className="text-right">
              {cluster.trees} {cluster.trees === 1 ? 'Baum' : 'Bäume'}
            </TableCell>
          </TableRow>
        ))}
      </TableBody>
    </Table>
  ),
}

export const Simple: Story = {
  render: () => (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Name</TableHead>
          <TableHead>Email</TableHead>
          <TableHead>Rolle</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow>
          <TableCell className="font-medium">Max Mustermann</TableCell>
          <TableCell>max@example.de</TableCell>
          <TableCell>Administrator</TableCell>
        </TableRow>
        <TableRow>
          <TableCell className="font-medium">Erika Musterfrau</TableCell>
          <TableCell>erika@example.de</TableCell>
          <TableCell>Benutzer</TableCell>
        </TableRow>
      </TableBody>
    </Table>
  ),
}

export const Striped: Story = {
  render: () => (
    <Table>
      <TableHeader>
        <TableRow>
          <TableHead>Fahrzeug</TableHead>
          <TableHead>Kennzeichen</TableHead>
          <TableHead>Wasserkapazität</TableHead>
          <TableHead>Status</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow className="bg-muted/50">
          <TableCell className="font-medium">MAN TGS</TableCell>
          <TableCell>FL-GE 123</TableCell>
          <TableCell>10.000 Liter</TableCell>
          <TableCell>
            <Badge variant="outline-green-dark">Verfügbar</Badge>
          </TableCell>
        </TableRow>
        <TableRow>
          <TableCell className="font-medium">Mercedes Sprinter</TableCell>
          <TableCell>FL-GE 456</TableCell>
          <TableCell>3.000 Liter</TableCell>
          <TableCell>
            <Badge variant="outline-yellow">Im Einsatz</Badge>
          </TableCell>
        </TableRow>
        <TableRow className="bg-muted/50">
          <TableCell className="font-medium">Anhänger</TableCell>
          <TableCell>FL-GE 789</TableCell>
          <TableCell>5.000 Liter</TableCell>
          <TableCell>
            <Badge variant="outline-red">Wartung</Badge>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  ),
}
