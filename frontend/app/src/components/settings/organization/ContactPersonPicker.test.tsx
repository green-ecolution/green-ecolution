import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { UserResponse } from '@/api/backendApi'
import ContactPersonPicker from './ContactPersonPicker'

const members = [
  { id: 'u1', firstName: 'Anke', lastName: 'Kruse', email: 'anke.kruse@flensburg.de' },
  { id: 'u2', firstName: 'Malte', lastName: 'Boysen', email: 'malte.boysen@flensburg.de' },
] as UserResponse[]

describe('ContactPersonPicker', () => {
  it('lists the members of the organization', () => {
    render(
      <ContactPersonPicker
        open
        members={members}
        selectedId={null}
        onOpenChange={vi.fn()}
        onSelect={vi.fn()}
      />,
    )
    expect(screen.getByText('Anke Kruse')).toBeInTheDocument()
    expect(screen.getByText('Malte Boysen')).toBeInTheDocument()
  })

  it('reports the picked member', async () => {
    const onSelect = vi.fn()
    render(
      <ContactPersonPicker
        open
        members={members}
        selectedId={null}
        onOpenChange={vi.fn()}
        onSelect={onSelect}
      />,
    )
    await userEvent.click(screen.getByText('Malte Boysen'))
    expect(onSelect).toHaveBeenCalledWith('u2')
  })

  it('offers removing the current contact person', async () => {
    const onSelect = vi.fn()
    render(
      <ContactPersonPicker
        open
        members={members}
        selectedId="u1"
        onOpenChange={vi.fn()}
        onSelect={onSelect}
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: /entfernen/i }))
    expect(onSelect).toHaveBeenCalledWith(null)
  })

  it('explains the empty case', () => {
    render(
      <ContactPersonPicker
        open
        members={[]}
        selectedId={null}
        onOpenChange={vi.fn()}
        onSelect={vi.fn()}
      />,
    )
    expect(screen.getByText(/keine Personen/i)).toBeInTheDocument()
  })
})
