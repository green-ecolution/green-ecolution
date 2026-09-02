import type { Meta, StoryObj } from '@storybook/react-vite'
import * as React from 'react'
import { CommentList } from '../src/components/ui/comment-list'
import { CommentItem } from '../src/components/ui/comment-item'
import { CommentComposer, type CommentAuthor } from '../src/components/ui/comment-composer'

const meta: Meta<typeof CommentList> = {
  title: 'UI/CommentList',
  component: CommentList,
  tags: ['autodocs'],
  argTypes: {
    isLoading: {
      control: 'boolean',
      description: 'Zeigt Platzhalter-Balken anstelle der Kommentare',
    },
    emptyLabel: {
      control: 'text',
      description: 'Text für den Leerzustand, überschreibt den Katalog-Standard',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Empty: Story = {
  args: {},
}

export const Loading: Story = {
  name: 'Lädt',
  args: { isLoading: true },
}

export const WithItems: Story = {
  name: 'Mit Kommentaren',
  render: () => (
    <CommentList>
      <CommentItem
        author={{ name: 'Anna Krüger' }}
        body="Boden war noch feucht, Gießen verschoben."
        timestamp="Heute, 09:12"
        canEdit
        canDelete
      />
      <CommentItem
        author={{ name: 'Tom Bergmann' }}
        body={'Sensor an Baum 12 zeigt weiter niedrige Werte.\nBitte bei nächster Tour prüfen.'}
        timestamp="Gestern, 16:40"
        editedLabel="bearbeitet"
      />
      <CommentItem
        author={{ name: 'Julia Nolte' }}
        body="Gruppe wurde vollständig bewässert, keine Auffälligkeiten."
        timestamp="Montag, 08:03"
      />
    </CommentList>
  ),
}

const avatarDataUri =
  'data:image/svg+xml;utf8,' +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64">' +
      '<rect width="64" height="64" fill="#2f6b52"/>' +
      '<text x="32" y="41" font-size="24" fill="white" text-anchor="middle" font-family="sans-serif">TB</text>' +
      '</svg>',
  )

interface DemoComment {
  id: number
  author: CommentAuthor
  body: string
  timestamp: string
}

const initialComments: DemoComment[] = [
  {
    id: 1,
    author: { name: 'Anna Krüger' },
    body: 'Boden war noch feucht, Gießen verschoben.',
    timestamp: 'Heute, 09:12',
  },
  {
    id: 2,
    author: { name: 'Tom Bergmann', avatarUrl: avatarDataUri },
    body: 'Sensor an Baum 12 zeigt weiter niedrige Werte.',
    timestamp: 'Gestern, 16:40',
  },
]

function GesamtansichtDemo() {
  const [comments, setComments] = React.useState<DemoComment[]>(initialComments)
  const [isSubmitting, setIsSubmitting] = React.useState(false)
  const nextId = React.useRef(initialComments.length + 1)

  const handleSubmit = async (body: string) => {
    setIsSubmitting(true)
    await new Promise((resolve) => setTimeout(resolve, 400))
    setComments((prev) => [
      ...prev,
      { id: nextId.current++, author: { name: 'Anna Krüger' }, body, timestamp: 'Gerade eben' },
    ])
    setIsSubmitting(false)
  }

  const handleDelete = (id: number) => {
    setComments((prev) => prev.filter((comment) => comment.id !== id))
  }

  const handleEdit = (id: number, body: string) => {
    setComments((prev) =>
      prev.map((comment) => (comment.id === id ? { ...comment, body } : comment)),
    )
  }

  return (
    <div className="flex max-w-xl flex-col gap-6">
      <CommentList emptyLabel="Noch keine Beobachtungen zu dieser Gruppe.">
        {comments.map((comment) => (
          <CommentItem
            key={comment.id}
            author={comment.author}
            body={comment.body}
            timestamp={comment.timestamp}
            canEdit
            canDelete
            onEdit={(body) => handleEdit(comment.id, body)}
            onDelete={() => handleDelete(comment.id)}
          />
        ))}
      </CommentList>
      <CommentComposer
        author={{ name: 'Anna Krüger' }}
        isSubmitting={isSubmitting}
        onSubmit={handleSubmit}
      />
    </div>
  )
}

export const Gesamtansicht: Story = {
  render: () => <GesamtansichtDemo />,
}
