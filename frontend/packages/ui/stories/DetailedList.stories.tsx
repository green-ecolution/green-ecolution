import type { Meta, StoryObj } from '@storybook/react'
import { DetailedList } from '../src/components/ui/detailed-list'
import { Badge } from '../src/components/ui/badge'

const meta: Meta<typeof DetailedList> = {
  title: 'UI/DetailedList',
  component: DetailedList,
  tags: ['autodocs'],
  argTypes: {
    columns: {
      control: 'select',
      options: [1, 2],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    details: [
      { label: 'Name', value: 'Eiche #1234' },
      { label: 'Art', value: 'Quercus robur' },
      { label: 'Standort', value: 'Hauptstraße 42' },
      { label: 'Pflanzjahr', value: '2019' },
    ],
  },
}

export const WithHeadline: Story = {
  args: {
    headline: 'Bauminformationen',
    details: [
      { label: 'Name', value: 'Eiche #1234' },
      { label: 'Art', value: 'Quercus robur' },
      { label: 'Standort', value: 'Hauptstraße 42' },
      { label: 'Pflanzjahr', value: '2019' },
    ],
  },
}

export const SingleColumn: Story = {
  args: {
    columns: 1,
    headline: 'Sensordaten',
    details: [
      { label: 'Temperatur', value: '24°C' },
      { label: 'Bodenfeuchtigkeit', value: '45%' },
      { label: 'Akkustand', value: '87%' },
      { label: 'Letzte Messung', value: 'vor 2 Minuten' },
    ],
  },
}

export const WithCustomValues: Story = {
  render: () => (
    <DetailedList
      headline="Baumstatus"
      details={[
        { label: 'Status', value: <Badge variant="success">Gesund</Badge> },
        { label: 'Bewässerungsbedarf', value: <Badge variant="warning">Mittel</Badge> },
        { label: 'Zuletzt gegossen', value: 'vor 3 Tagen' },
        { label: 'Nächste Bewässerung', value: 'Morgen, 8:00 Uhr' },
      ]}
    />
  ),
}

export const VehicleDetails: Story = {
  args: {
    headline: 'Fahrzeuginformationen',
    details: [
      { label: 'Fahrzeug-ID', value: 'WTR-001' },
      { label: 'Typ', value: 'Wassertankwagen' },
      { label: 'Kapazität', value: '5000 L' },
      { label: 'Aktuelle Ladung', value: '3200 L' },
      { label: 'Fahrer', value: 'Max Mustermann' },
      { label: 'Status', value: 'Unterwegs' },
    ],
  },
}

export const SensorDetails: Story = {
  args: {
    headline: 'Sensor #SN-2024-0042',
    columns: 1,
    details: [
      { label: 'Geräte-ID', value: 'SN-2024-0042' },
      { label: 'Modell', value: 'LoRaWAN Bodensensor v2' },
      { label: 'Zugewiesener Baum', value: 'Eiche #1234' },
      { label: 'Akku', value: '92%' },
      { label: 'Signalstärke', value: 'Ausgezeichnet (-65 dBm)' },
      { label: 'Firmware', value: 'v2.1.3' },
      { label: 'Letzte Kommunikation', value: 'vor 5 Minuten' },
    ],
  },
}
