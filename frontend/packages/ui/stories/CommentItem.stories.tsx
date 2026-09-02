import type { Meta, StoryObj } from '@storybook/react-vite'
import * as React from 'react'
import { CommentItem } from '../src/components/ui/comment-item'

const meta: Meta<typeof CommentItem> = {
  title: 'UI/CommentItem',
  component: CommentItem,
  tags: ['autodocs'],
  argTypes: {
    canEdit: {
      control: 'boolean',
      description: 'Zeigt die Bearbeiten-Aktion, z. B. für den eigenen Kommentar',
    },
    canDelete: {
      control: 'boolean',
      description: 'Zeigt die Löschen-Aktion, z. B. mit Löschrecht auf der Bewässerungsgruppe',
    },
    isSaving: {
      control: 'boolean',
      description: 'Sperrt Speichern und Abbrechen während eine Bearbeitung läuft',
    },
    editedLabel: {
      control: 'text',
      description: 'Hinweistext, wenn der Kommentar nachträglich geändert wurde',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

const avatarDataUri =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">' +
      '<rect width="64" height="64" fill="#2f6b52"/>' +
      '<text x="32" y="41" font-size="24" fill="white" text-anchor="middle" font-family="sans-serif">TB</text>' +
      '</svg>',
  )

export const Default: Story = {
  args: {
    author: { name: 'Julia Nolte' },
    body: 'Gruppe wurde vollständig bewässert, keine Auffälligkeiten.',
    timestamp: 'Montag, 08:03',
  },
}

export const WithPermissions: Story = {
  name: 'Mit Berechtigungen',
  args: {
    author: { name: 'Anna Krüger' },
    body: 'Boden war noch feucht, Gießen verschoben.',
    timestamp: 'Heute, 09:12',
    canEdit: true,
    canDelete: true,
    onEdit: (body: string) => console.log('gespeichert:', body),
    onDelete: () => console.log('gelöscht'),
  },
}

export const Edited: Story = {
  name: 'Bearbeitet',
  args: {
    author: { name: 'Tom Bergmann', avatarUrl: avatarDataUri },
    body: 'Sensor an Baum 12 zeigt weiter niedrige Werte.\nBitte bei nächster Tour prüfen.',
    timestamp: 'Gestern, 16:40',
    editedLabel: 'bearbeitet',
  },
}

function EditModePreview(args: React.ComponentProps<typeof CommentItem>) {
  const containerRef = React.useRef<HTMLDivElement>(null)

  React.useEffect(() => {
    containerRef.current?.querySelector<HTMLButtonElement>('button[title="Bearbeiten"]')?.click()
  }, [])

  return (
    <div ref={containerRef}>
      <CommentItem {...args} />
    </div>
  )
}

export const EditMode: Story = {
  name: 'Im Bearbeitungsmodus',
  render: (args) => <EditModePreview {...args} />,
  args: {
    author: { name: 'Anna Krüger' },
    body: 'Boden war noch feucht, Gießen verschoben.',
    timestamp: 'Heute, 09:12',
    canEdit: true,
    canDelete: true,
    onEdit: (body: string) => console.log('gespeichert:', body),
    onDelete: () => console.log('gelöscht'),
  },
}
