import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '../src/components/ui/select'
import { Label } from '../src/components/ui/label'

const meta: Meta<typeof Select> = {
  title: 'UI/Select',
  component: Select,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <Select>
      <SelectTrigger className="w-[280px]">
        <SelectValue placeholder="Baumart auswählen" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>Baumarten</SelectLabel>
          <SelectItem value="oak">Eiche</SelectItem>
          <SelectItem value="beech">Buche</SelectItem>
          <SelectItem value="maple">Ahorn</SelectItem>
          <SelectItem value="linden">Linde</SelectItem>
          <SelectItem value="birch">Birke</SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
  ),
}

export const WithLabel: Story = {
  render: () => (
    <div className="grid w-full max-w-sm items-center gap-1.5">
      <Label htmlFor="tree-status">Baumstatus</Label>
      <Select>
        <SelectTrigger id="tree-status">
          <SelectValue placeholder="Status auswählen" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="healthy">Gesund</SelectItem>
          <SelectItem value="needs-water">Bewässerung nötig</SelectItem>
          <SelectItem value="critical">Kritisch</SelectItem>
          <SelectItem value="unknown">Unbekannt</SelectItem>
        </SelectContent>
      </Select>
    </div>
  ),
}

export const WithGroups: Story = {
  render: () => (
    <Select>
      <SelectTrigger className="w-[280px]">
        <SelectValue placeholder="Fahrzeug auswählen" />
      </SelectTrigger>
      <SelectContent>
        <SelectGroup>
          <SelectLabel>LKW</SelectLabel>
          <SelectItem value="truck-1">Wassertank-LKW 1</SelectItem>
          <SelectItem value="truck-2">Wassertank-LKW 2</SelectItem>
        </SelectGroup>
        <SelectGroup>
          <SelectLabel>Transporter</SelectLabel>
          <SelectItem value="van-1">Service-Transporter 1</SelectItem>
          <SelectItem value="van-2">Service-Transporter 2</SelectItem>
        </SelectGroup>
      </SelectContent>
    </Select>
  ),
}

export const Disabled: Story = {
  render: () => (
    <Select disabled>
      <SelectTrigger className="w-[280px]">
        <SelectValue placeholder="Option auswählen" />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="1">Option 1</SelectItem>
        <SelectItem value="2">Option 2</SelectItem>
      </SelectContent>
    </Select>
  ),
}

export const WithDefaultValue: Story = {
  render: () => (
    <Select defaultValue="healthy">
      <SelectTrigger className="w-[280px]">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="healthy">Gesund</SelectItem>
        <SelectItem value="needs-water">Bewässerung nötig</SelectItem>
        <SelectItem value="critical">Kritisch</SelectItem>
      </SelectContent>
    </Select>
  ),
}
