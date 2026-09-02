import type { Meta, StoryObj } from '@storybook/react-vite'
import { CommentComposer } from '../src/components/ui/comment-composer'

const meta: Meta<typeof CommentComposer> = {
  title: 'UI/CommentComposer',
  component: CommentComposer,
  tags: ['autodocs'],
  argTypes: {
    isSubmitting: {
      control: 'boolean',
      description: 'Zeigt einen Spinner und sperrt das Textfeld während des Sendens',
    },
    disabled: {
      control: 'boolean',
      description: 'Sperrt das gesamte Formular, z. B. ohne Kommentar-Berechtigung',
    },
    maxLength: {
      control: { type: 'number', min: 20, max: 4000, step: 20 },
      description: 'Maximale Zeichenzahl, spiegelt das CommentBody-Limit der Domäne',
    },
    placeholder: {
      control: 'text',
      description: 'Platzhaltertext im Textfeld',
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

const author = { name: 'Anna Krüger' }

export const Default: Story = {
  args: {
    author,
    onSubmit: (body: string) => {
      console.log('Kommentar abgeschickt:', body)
    },
  },
}

export const Submitting: Story = {
  name: 'Wird gesendet',
  args: {
    author,
    isSubmitting: true,
    onSubmit: () => {},
  },
}

export const AtCharacterLimit: Story = {
  name: 'Am Zeichenlimit',
  args: {
    author,
    maxLength: 40,
    placeholder: 'Kurzer Vermerk zur Bewässerungsgruppe…',
    onSubmit: (body: string) => {
      console.log('Kommentar abgeschickt:', body)
    },
  },
}

export const Disabled: Story = {
  name: 'Gesperrt',
  args: {
    author,
    disabled: true,
    onSubmit: () => {},
  },
}
