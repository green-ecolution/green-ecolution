import type { Meta, StoryObj } from '@storybook/react-vite'
import { CalendarClock, CheckCircle2, Sprout, Truck } from 'lucide-react'
import {
  KanbanBoard,
  KanbanColumn,
  KanbanColumnHeader,
  KanbanCard,
  KanbanDropHint,
  KanbanColumnEmpty,
} from '../src/components/ui/kanban'
import { Badge } from '../src/components/ui/badge'

const meta: Meta<typeof KanbanBoard> = {
  title: 'UI/Kanban',
  component: KanbanBoard,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

const PlanCard = ({ title, groups, liters }: { title: string; groups: number; liters: string }) => (
  <KanbanCard>
    <p className="font-lato font-semibold text-dark">{title}</p>
    <div className="mt-2 flex flex-wrap gap-1.5">
      <Badge variant="muted" className="tabular-nums">{groups} Gruppen</Badge>
      <Badge variant="muted" className="tabular-nums">{liters}</Badge>
      <Badge variant="muted">FL-GE-A01</Badge>
    </div>
  </KanbanCard>
)

export const Board: Story = {
  render: () => (
    <KanbanBoard>
      <KanbanColumn tone="suggestion">
        <KanbanColumnHeader icon={<Sprout />} title="Vorschläge" count={2} />
        <KanbanCard>
          <p className="font-lato font-semibold text-dark">Hafenspitze</p>
          <p className="mt-1 text-sm text-dark-600">12 Bäume · Sehr trocken</p>
        </KanbanCard>
        <KanbanCard>
          <p className="font-lato font-semibold text-dark">Volkspark West</p>
          <p className="mt-1 text-sm text-dark-600">31 Bäume · Sehr trocken</p>
        </KanbanCard>
      </KanbanColumn>
      <KanbanColumn>
        <KanbanColumnHeader icon={<CalendarClock />} title="Geplant" count={1} />
        <PlanCard title="Di, 8. Juli" groups={5} liters="3.800 L" />
      </KanbanColumn>
      <KanbanColumn tone="active">
        <KanbanColumnHeader icon={<Truck />} title="Unterwegs" count={1} />
        <PlanCard title="Di, 8. Juli" groups={6} liters="4.200 L" />
      </KanbanColumn>
      <KanbanColumn>
        <KanbanColumnHeader icon={<CheckCircle2 />} title="Erledigt" count={0} />
        <KanbanColumnEmpty>Noch keine erledigten Einsätze.</KanbanColumnEmpty>
      </KanbanColumn>
    </KanbanBoard>
  ),
}

export const DragStates: Story = {
  render: () => (
    <KanbanBoard>
      <KanbanColumn state="dimmed">
        <KanbanColumnHeader icon={<CalendarClock />} title="Geplant" count={1} />
        <PlanCard title="Di, 8. Juli" groups={5} liters="3.800 L" />
      </KanbanColumn>
      <KanbanColumn tone="active" state="target">
        <KanbanColumnHeader icon={<Truck />} title="Unterwegs" count={0} />
        <KanbanDropHint label="Einsatz starten" />
      </KanbanColumn>
      <KanbanColumn>
        <KanbanColumnHeader icon={<CheckCircle2 />} title="Erledigt" count={0} />
        <KanbanCard state="dragging">
          <p className="font-lato font-semibold text-dark">Wird gezogen…</p>
        </KanbanCard>
        <KanbanCard state="ghost">
          <p className="font-lato font-semibold text-dark">Ursprungsposition</p>
        </KanbanCard>
      </KanbanColumn>
    </KanbanBoard>
  ),
}
