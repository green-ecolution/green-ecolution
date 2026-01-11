import type { Meta, StoryObj } from '@storybook/react-vite'
import { FormField, TextareaField } from '../src/components/ui/form-field'

const meta: Meta<typeof FormField> = {
  title: 'UI/FormField',
  component: FormField,
  tags: ['autodocs'],
  argTypes: {
    type: {
      control: 'select',
      options: ['text', 'email', 'password', 'number', 'tel', 'url'],
    },
  },
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: {
    label: 'Name',
    placeholder: 'Name eingeben',
  },
}

export const WithError: Story = {
  args: {
    label: 'E-Mail',
    type: 'email',
    placeholder: 'E-Mail eingeben',
    error: 'Bitte gib eine gültige E-Mail-Adresse ein',
    defaultValue: 'ungueltige-email',
  },
}

export const WithDescription: Story = {
  args: {
    label: 'Benutzername',
    placeholder: 'Benutzername eingeben',
    description: 'Dies wird dein öffentlicher Anzeigename sein.',
  },
}

export const Required: Story = {
  args: {
    label: 'Vollständiger Name',
    placeholder: 'Vollständigen Namen eingeben',
    required: true,
  },
}

export const Disabled: Story = {
  args: {
    label: 'Schreibgeschütztes Feld',
    defaultValue: 'Dieser Wert kann nicht geändert werden',
    disabled: true,
  },
}

export const HiddenLabel: Story = {
  args: {
    label: 'Suche',
    placeholder: 'Suchen...',
    hideLabel: true,
  },
}

export const TextareaExample: Story = {
  render: () => (
    <TextareaField
      label="Beschreibung"
      placeholder="Beschreibung eingeben..."
      description="Maximal 500 Zeichen."
    />
  ),
}

export const TextareaWithError: Story = {
  render: () => (
    <TextareaField
      label="Notizen"
      placeholder="Notizen eingeben..."
      error="Notizen sind erforderlich"
      required
    />
  ),
}

export const FormExample: Story = {
  render: () => (
    <form className="space-y-4 max-w-md">
      <FormField
        label="Baumname"
        placeholder="z.B. Eiche #123"
        required
      />
      <FormField
        label="Standort"
        placeholder="Straßenadresse oder Koordinaten"
        description="Geben Sie den genauen Standort des Baumes ein."
      />
      <FormField
        label="Pflanzjahr"
        type="number"
        placeholder="z.B. 2020"
        min={1900}
        max={new Date().getFullYear()}
      />
      <TextareaField
        label="Notizen"
        placeholder="Zusätzliche Informationen zum Baum..."
        rows={4}
      />
    </form>
  ),
}
