import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import type { OrganizationResponse } from '@/api/backendApi'
import SettingsOrganizationPicker from './SettingsOrganizationPicker'

vi.mock('@tanstack/react-router', () => ({
  Link: ({ children }: { children: React.ReactNode }) => <a href="/structure">{children}</a>,
}))

const ORGS: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'sued', name: 'Betriebshof Süd', memberCount: 2, parentId: 'amt' },
]

describe('SettingsOrganizationPicker', () => {
  it('shows only the name when there is a single organization', () => {
    render(
      <SettingsOrganizationPicker organizations={[ORGS[0]]} selectedId="amt" onSelect={vi.fn()} />,
    )
    expect(screen.getByText('Grünflächenamt')).toBeInTheDocument()
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
  })

  it.each([
    ['no organization', []],
    ['only other organizations', [ORGS[1]]],
  ])('renders nothing while the selected one is unknown (%s)', (_, organizations) => {
    render(
      <SettingsOrganizationPicker
        organizations={organizations}
        selectedId="amt"
        onSelect={vi.fn()}
      />,
    )
    expect(screen.queryByRole('button')).not.toBeInTheDocument()
    expect(screen.queryByText(/Stadtgärtnerei|Grünflächenamt/)).not.toBeInTheDocument()
    expect(document.querySelector('[aria-hidden="true"]')).toBeNull()
  })

  it('names the selected organization and its number of sub-organizations', () => {
    render(<SettingsOrganizationPicker organizations={ORGS} selectedId="amt" onSelect={vi.fn()} />)
    expect(
      screen.getByRole('button', { name: 'Organisation wechseln, aktuell Grünflächenamt' }),
    ).toBeInTheDocument()
    expect(screen.getByText('mit 2 Unterorganisationen')).toBeInTheDocument()
  })

  it('lists the organizations as a tree and reports the chosen one', async () => {
    const onSelect = vi.fn()
    render(<SettingsOrganizationPicker organizations={ORGS} selectedId="amt" onSelect={onSelect} />)
    await userEvent.click(screen.getByRole('button', { name: /Organisation wechseln/ }))
    expect(screen.getAllByRole('option').map((option) => option.textContent)).toEqual([
      expect.stringContaining('Grünflächenamt'),
      expect.stringContaining('Betriebshof Süd'),
      expect.stringContaining('Stadtgärtnerei Nord'),
    ])
    await userEvent.click(screen.getByRole('option', { name: /Stadtgärtnerei Nord/ }))
    expect(onSelect).toHaveBeenCalledWith('nord')
  })

  it('never shows an organization above the visible ones', async () => {
    render(<SettingsOrganizationPicker organizations={ORGS} selectedId="amt" onSelect={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: /Organisation wechseln/ }))
    expect(screen.queryByText('root')).not.toBeInTheDocument()
    expect(screen.getAllByRole('option')).toHaveLength(3)
  })

  it('filters by name and keeps the ancestors for context', async () => {
    render(<SettingsOrganizationPicker organizations={ORGS} selectedId="amt" onSelect={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: /Organisation wechseln/ }))
    await userEvent.type(screen.getByPlaceholderText('Organisation suchen'), 'Nord')
    expect(screen.getAllByRole('option')).toHaveLength(2)
  })

  it('links to the structure page', async () => {
    render(<SettingsOrganizationPicker organizations={ORGS} selectedId="amt" onSelect={vi.fn()} />)
    await userEvent.click(screen.getByRole('button', { name: /Organisation wechseln/ }))
    expect(screen.getByRole('link', { name: 'Struktur verwalten' })).toBeInTheDocument()
  })
})
