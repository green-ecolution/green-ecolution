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
      { label: 'Name', value: 'Oak Tree #1234' },
      { label: 'Species', value: 'Quercus robur' },
      { label: 'Location', value: 'Main Street 42' },
      { label: 'Planting Year', value: '2019' },
    ],
  },
}

export const WithHeadline: Story = {
  args: {
    headline: 'Tree Information',
    details: [
      { label: 'Name', value: 'Oak Tree #1234' },
      { label: 'Species', value: 'Quercus robur' },
      { label: 'Location', value: 'Main Street 42' },
      { label: 'Planting Year', value: '2019' },
    ],
  },
}

export const SingleColumn: Story = {
  args: {
    columns: 1,
    headline: 'Sensor Data',
    details: [
      { label: 'Temperature', value: '24°C' },
      { label: 'Soil Moisture', value: '45%' },
      { label: 'Battery Level', value: '87%' },
      { label: 'Last Reading', value: '2 minutes ago' },
    ],
  },
}

export const WithCustomValues: Story = {
  render: () => (
    <DetailedList
      headline="Tree Status"
      details={[
        { label: 'Status', value: <Badge variant="success">Healthy</Badge> },
        { label: 'Watering Need', value: <Badge variant="warning">Medium</Badge> },
        { label: 'Last Watered', value: '3 days ago' },
        { label: 'Next Scheduled', value: 'Tomorrow, 8:00 AM' },
      ]}
    />
  ),
}

export const VehicleDetails: Story = {
  args: {
    headline: 'Vehicle Information',
    details: [
      { label: 'Vehicle ID', value: 'WTR-001' },
      { label: 'Type', value: 'Water Truck' },
      { label: 'Capacity', value: '5000 L' },
      { label: 'Current Load', value: '3200 L' },
      { label: 'Driver', value: 'Max Mustermann' },
      { label: 'Status', value: 'On Route' },
    ],
  },
}

export const SensorDetails: Story = {
  args: {
    headline: 'Sensor #SN-2024-0042',
    columns: 1,
    details: [
      { label: 'Device ID', value: 'SN-2024-0042' },
      { label: 'Model', value: 'LoRaWAN Soil Sensor v2' },
      { label: 'Assigned Tree', value: 'Oak Tree #1234' },
      { label: 'Battery', value: '92%' },
      { label: 'Signal Strength', value: 'Excellent (-65 dBm)' },
      { label: 'Firmware', value: 'v2.1.3' },
      { label: 'Last Communication', value: '5 minutes ago' },
    ],
  },
}
