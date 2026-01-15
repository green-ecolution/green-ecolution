import type { Meta, StoryObj } from '@storybook/react-vite'
import { Search, Mail, Eye, EyeOff, X, Check, MapPin } from 'lucide-react'
import { useState } from 'react'
import {
  InputGroup,
  InputGroupAddon,
  InputGroupButton,
  InputGroupText,
  InputGroupInput,
  InputGroupTextarea,
} from '../src/components/ui/input-group'

const meta: Meta<typeof InputGroup> = {
  title: 'UI/InputGroup',
  component: InputGroup,
  tags: ['autodocs'],
}

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="Eingabe..." />
    </InputGroup>
  ),
}

export const SearchInput: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="Suchen..." />
      <InputGroupAddon>
        <Search />
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const SearchWithButton: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="Bäume suchen..." />
      <InputGroupAddon>
        <Search />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">
        <InputGroupButton variant="default" size="xs">
          Suchen
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const EmailInput: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput type="email" placeholder="E-Mail eingeben" />
      <InputGroupAddon>
        <Mail />
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const PasswordToggle: Story = {
  render: function PasswordToggleStory() {
    const [showPassword, setShowPassword] = useState(false)
    return (
      <InputGroup className="max-w-sm">
        <InputGroupInput
          type={showPassword ? 'text' : 'password'}
          placeholder="Passwort eingeben"
        />
        <InputGroupAddon align="inline-end">
          <InputGroupButton
            size="icon-xs"
            onClick={() => setShowPassword(!showPassword)}
            aria-label={showPassword ? 'Passwort verbergen' : 'Passwort anzeigen'}
          >
            {showPassword ? <EyeOff /> : <Eye />}
          </InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
    )
  },
}

export const WithPrefix: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="beispiel.de" className="!pl-1" />
      <InputGroupAddon>
        <InputGroupText>https://</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const WithSuffix: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput type="number" placeholder="0" className="!pr-1" />
      <InputGroupAddon align="inline-end">
        <InputGroupText>kg</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const WithClearButton: Story = {
  render: function ClearButtonStory() {
    const [value, setValue] = useState('Eiche #42')
    return (
      <InputGroup className="max-w-sm">
        <InputGroupInput
          placeholder="Suchen..."
          value={value}
          onChange={(e) => setValue(e.target.value)}
        />
        <InputGroupAddon>
          <Search />
        </InputGroupAddon>
        {value && (
          <InputGroupAddon align="inline-end">
            <InputGroupButton size="icon-xs" onClick={() => setValue('')} aria-label="Löschen">
              <X />
            </InputGroupButton>
          </InputGroupAddon>
        )}
      </InputGroup>
    )
  },
}

export const LocationSearch: Story = {
  render: () => (
    <InputGroup className="max-w-md">
      <InputGroupInput placeholder="Standort suchen..." />
      <InputGroupAddon>
        <MapPin />
      </InputGroupAddon>
      <InputGroupAddon align="inline-end">
        <InputGroupText className="text-xs">12 Ergebnisse</InputGroupText>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const Validated: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput defaultValue="valid@example.de" />
      <InputGroupAddon align="inline-end">
        <div className="bg-green-dark text-white flex size-4 items-center justify-center rounded-full">
          <Check className="size-3" />
        </div>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const WithTextarea: Story = {
  render: () => (
    <InputGroup className="max-w-md">
      <InputGroupTextarea placeholder="Notizen zum Baum eingeben..." rows={4} />
      <InputGroupAddon align="block-end" className="border-t border-dark-100">
        <InputGroupText className="text-xs">0/500 Zeichen</InputGroupText>
        <InputGroupButton variant="default" size="sm" className="ml-auto">
          Speichern
        </InputGroupButton>
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const Error: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="E-Mail" aria-invalid="true" defaultValue="ungueltig" />
      <InputGroupAddon>
        <Mail />
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const Disabled: Story = {
  render: () => (
    <InputGroup className="max-w-sm">
      <InputGroupInput placeholder="Deaktiviert" disabled />
      <InputGroupAddon>
        <Search />
      </InputGroupAddon>
    </InputGroup>
  ),
}

export const AllVariants: Story = {
  render: () => (
    <div className="grid gap-4 max-w-md">
      <InputGroup>
        <InputGroupInput placeholder="Icon links (Standard)" />
        <InputGroupAddon>
          <Search />
        </InputGroupAddon>
      </InputGroup>

      <InputGroup>
        <InputGroupInput placeholder="Icon rechts" />
        <InputGroupAddon align="inline-end">
          <Search />
        </InputGroupAddon>
      </InputGroup>

      <InputGroup>
        <InputGroupInput placeholder="Icons beidseitig" />
        <InputGroupAddon>
          <MapPin />
        </InputGroupAddon>
        <InputGroupAddon align="inline-end">
          <Search />
        </InputGroupAddon>
      </InputGroup>

      <InputGroup>
        <InputGroupInput placeholder="Mit Text und Button" />
        <InputGroupAddon>
          <InputGroupText>https://</InputGroupText>
        </InputGroupAddon>
        <InputGroupAddon align="inline-end">
          <InputGroupButton>Prüfen</InputGroupButton>
        </InputGroupAddon>
      </InputGroup>
    </div>
  ),
}
