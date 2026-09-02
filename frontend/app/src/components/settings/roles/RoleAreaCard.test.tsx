import { describe, it, expect, vi, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { render, screen, userEvent } from '@/test/utils'
import { getI18n } from '@/lib/i18n'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { permissionAreasFor } from '@/lib/auth/permissionAreas'
import RoleAreaCard from './RoleAreaCard'

let t: TFunction<'settings'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'settings')
})

const treeArea = () => permissionAreasFor(t).find((area) => area.resource === 'tree')!

const renderCard = (overrides: Partial<React.ComponentProps<typeof RoleAreaCard>> = {}) => {
  const props: React.ComponentProps<typeof RoleAreaCard> = {
    area: treeArea(),
    permissions: new Set(['tree:read', 'tree:create', 'tree:update']),
    grantable: UNRESTRICTED,
    onLevelChange: vi.fn(),
    onActionToggle: vi.fn(),
    ...overrides,
  }
  render(<RoleAreaCard {...props} />)
  return props
}

describe('RoleAreaCard', () => {
  it('shows the area label, description and active count', () => {
    renderCard()
    expect(screen.getByText('Bäume')).toBeInTheDocument()
    expect(screen.getByText('Bäume & Standorte')).toBeInTheDocument()
    expect(screen.getByText('3 von 4 Aktionen aktiv')).toBeInTheDocument()
  })

  it('preselects the level matching the permission set', () => {
    renderCard()
    expect(screen.getByRole('radio', { name: 'Bearbeiten' })).toBeChecked()
  })

  it('marks a set matching no preset as individuell', () => {
    renderCard({ permissions: new Set(['tree:read', 'tree:delete']) })
    expect(screen.getByText('Individuell')).toBeInTheDocument()
    for (const radio of screen.getAllByRole('radio')) {
      expect(radio).not.toBeChecked()
    }
  })

  it('reports the chosen level', async () => {
    const props = renderCard()
    await userEvent.click(screen.getByRole('radio', { name: 'Verwalten' }))
    expect(props.onLevelChange).toHaveBeenCalledWith('manage')
  })

  it('keeps the actions collapsed until asked', async () => {
    renderCard()
    expect(screen.queryByRole('switch', { name: /Bäume ansehen/ })).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('button', { name: 'Aktionen für Bäume anzeigen' }))
    expect(screen.getByRole('switch', { name: /Bäume ansehen/ })).toBeInTheDocument()
  })

  it('reports a toggled action', async () => {
    const props = renderCard()
    await userEvent.click(screen.getByRole('button', { name: 'Aktionen für Bäume anzeigen' }))
    await userEvent.click(screen.getByRole('switch', { name: /Baum löschen/ }))
    expect(props.onActionToggle).toHaveBeenCalledWith('tree:delete')
  })

  it('disables actions the caller cannot grant', async () => {
    renderCard({ grantable: new Set(['tree:read']) })
    await userEvent.click(screen.getByRole('button', { name: 'Aktionen für Bäume anzeigen' }))
    expect(screen.getByRole('switch', { name: /Bäume ansehen/ })).toBeEnabled()
    expect(screen.getByRole('switch', { name: /Baum löschen/ })).toBeDisabled()
  })

  it('disables everything in read-only mode', async () => {
    renderCard({ readOnly: true })
    expect(screen.getByRole('radio', { name: 'Verwalten' })).toBeDisabled()
    await userEvent.click(screen.getByRole('button', { name: 'Aktionen für Bäume anzeigen' }))
    expect(screen.getByRole('switch', { name: /Bäume ansehen/ })).toBeDisabled()
  })
})
