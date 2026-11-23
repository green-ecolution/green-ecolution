import type { Meta, StoryObj } from '@storybook/react'
import { Eye, Lock, Plus, Users } from 'lucide-react'
import { EmptyState } from '../src/components/ui/empty-state'
import { Button } from '../src/components/ui/button'

const meta: Meta<typeof EmptyState> = {
  title: 'UI/EmptyState',
  component: EmptyState,
  tags: ['autodocs'],
  parameters: {
    layout: 'padded',
  },
}

export default meta
type Story = StoryObj<typeof meta>

function ExampleNode({
  initial,
  name,
  parent,
}: {
  initial: string
  name: string
  parent?: boolean
}) {
  return (
    <div className={parent ? '' : 'flex items-center gap-2.5 pl-5'}>
      {!parent && (
        <span className="-mt-3 h-3.5 w-3.5 shrink-0 rounded-bl-md border-b-2 border-l-2 border-dark-300" />
      )}
      <div
        className={
          parent
            ? 'flex min-h-[52px] items-center gap-3 rounded-lg border border-dark-300 bg-white px-3.5'
            : 'flex min-h-[52px] flex-1 items-center gap-3 rounded-lg border border-dashed border-dark-400 bg-dark-50/60 px-3.5'
        }
      >
        <span
          className={
            parent
              ? 'flex h-8 w-8 items-center justify-center rounded-lg bg-green-dark font-bold text-white'
              : 'flex h-8 w-8 items-center justify-center rounded-lg bg-green-light-100 font-bold text-green-dark'
          }
        >
          {initial}
        </span>
        <span className="font-lato font-bold">{name}</span>
      </div>
    </div>
  )
}

const benefits = {
  title: 'Wofür Unterorganisationen gut sind',
  items: [
    {
      icon: <Users className="size-5" />,
      title: 'Eigenes Team',
      text: 'Jede Unterorganisation hat eigene Mitarbeitende und eigene Rollen.',
    },
    {
      icon: <Lock className="size-5" />,
      title: 'Getrennte Daten',
      text: 'Ihre Mitarbeitenden sehen nur, was zur eigenen Unterorganisation gehört, zum Beispiel Fahrzeuge, Sensoren und Einsatzpläne.',
    },
    {
      icon: <Eye className="size-5" />,
      title: 'Alles im Blick',
      text: 'TBZ behält den Überblick. Mit den passenden Rechten siehst du weiterhin alles aus allen Unterorganisationen.',
    },
  ],
}

const note =
  'Gut zu wissen: Eine Unterorganisation lässt sich später umbenennen, aber nicht verschieben. Löschen geht nur, solange sie keine Mitarbeitenden und keine eigenen Unterorganisationen hat.'

export const StrukturOhneUnterorganisationen: Story = {
  name: 'Struktur ohne Unterorganisationen',
  render: () => (
    <EmptyState
      title="TBZ hat noch keine Unterorganisationen"
      description="Eine Unterorganisation ist ein eigener Bereich innerhalb von TBZ, zum Beispiel ein Betriebshof, eine Abteilung oder eine beauftragte Firma."
      action={
        <Button>
          <Plus className="size-4" aria-hidden="true" />
          Unterorganisation anlegen
        </Button>
      }
      example={{
        caption: 'So könnte es aussehen',
        label: 'Beispiel: TBZ mit den Unterorganisationen Betriebshof und Beauftragte Firma',
        content: (
          <div className="flex flex-col gap-2">
            <ExampleNode parent initial="T" name="TBZ" />
            <ExampleNode initial="B" name="Betriebshof" />
            <ExampleNode initial="F" name="Beauftragte Firma" />
          </div>
        ),
      }}
      benefits={benefits}
      note={note}
    />
  ),
}

export const OhneRechtZumAnlegen: Story = {
  name: 'Ohne Recht zum Anlegen',
  render: () => (
    <EmptyState
      title="TBZ hat noch keine Unterorganisationen"
      description="Eine Unterorganisation ist ein eigener Bereich innerhalb von TBZ, zum Beispiel ein Betriebshof, eine Abteilung oder eine beauftragte Firma."
      example={{
        caption: 'So könnte es aussehen',
        label: 'Beispiel: TBZ mit den Unterorganisationen Betriebshof und Beauftragte Firma',
        content: (
          <div className="flex flex-col gap-2">
            <ExampleNode parent initial="T" name="TBZ" />
            <ExampleNode initial="B" name="Betriebshof" />
            <ExampleNode initial="F" name="Beauftragte Firma" />
          </div>
        ),
      }}
      benefits={benefits}
      note={note}
    />
  ),
}
