import type { Meta, StoryObj } from '@storybook/react-vite'
import { FilterChip } from '../src/components/ui/filter-chip'

const meta: Meta<typeof FilterChip> = {
  title: 'UI/FilterChip',
  component: FilterChip,
  tags: ['autodocs'],
  args: {
    label: 'Kritisch',
    onRemove: () => {},
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {}

export const TreeListFilters: Story = {
  render: () => (
    <div className="flex flex-wrap gap-2">
      <FilterChip label="Kritisch" onRemove={() => {}} />
      <FilterChip label="Unbekannt" onRemove={() => {}} />
      <FilterChip label="Solitüde Strand" onRemove={() => {}} />
      <FilterChip label="Ohne Sensor" onRemove={() => {}} />
      <FilterChip label="Pflanzjahr 2019" onRemove={() => {}} />
    </div>
  ),
}
