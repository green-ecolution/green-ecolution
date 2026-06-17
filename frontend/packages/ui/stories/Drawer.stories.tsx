import type { Meta, StoryObj } from '@storybook/react-vite'
import {
  Drawer,
  DrawerTrigger,
  DrawerContent,
  DrawerHeader,
  DrawerTitle,
  DrawerDescription,
  DrawerFooter,
  DrawerClose,
} from '../src/components/ui/drawer'
import { Button } from '../src/components/ui/button'

const meta: Meta = {
  title: 'UI/Drawer',
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj

export const Default: Story = {
  render: () => (
    <Drawer>
      <DrawerTrigger asChild>
        <Button variant="outline">Baumgruppe öffnen</Button>
      </DrawerTrigger>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>Baumgruppe Hafenspitze</DrawerTitle>
          <DrawerDescription>12 Bäume · Bodenfeuchte 14 %</DrawerDescription>
        </DrawerHeader>
        <div className="px-4 pb-4 text-sm text-dark-700">Weitere Details zur Baumgruppe …</div>
        <DrawerFooter>
          <Button>Zum Dashboard</Button>
          <DrawerClose asChild>
            <Button variant="outline">Schließen</Button>
          </DrawerClose>
        </DrawerFooter>
      </DrawerContent>
    </Drawer>
  ),
}

export const WithSnapPoints: Story = {
  render: () => (
    <Drawer snapPoints={['200px', 1]}>
      <DrawerTrigger asChild>
        <Button variant="outline">Mit Snap-Points öffnen</Button>
      </DrawerTrigger>
      <DrawerContent>
        <DrawerHeader>
          <DrawerTitle>Baumgruppe Hafenspitze</DrawerTitle>
          <DrawerDescription>Zuerst Kurzinfo — hochziehen für mehr.</DrawerDescription>
        </DrawerHeader>
        <div className="space-y-3 overflow-y-auto px-4 pb-8 text-sm text-dark-700">
          {Array.from({ length: 8 }).map((_, index) => (
            <p key={index}>Detailzeile {index + 1}</p>
          ))}
        </div>
      </DrawerContent>
    </Drawer>
  ),
}
