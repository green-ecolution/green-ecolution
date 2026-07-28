import type { ComponentProps } from 'react'
import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent } from '@/test/utils'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import type { Role } from '@/api/backendApi'
import RoleDetail from './RoleDetail'

const template: Role = {
  id: 't1',
  name: 'Routenplanung',
  description: 'Planung von Bewässerungsrouten',
  isTemplate: true,
  permissions: ['watering_plan:read', 'tree:read'],
}

const ownRole: Role = {
  id: 'r1',
  name: 'Bezirksleiter Nord',
  description: 'Kopie von Routenplanung',
  isTemplate: false,
  permissions: ['watering_plan:read'],
}

const baseProps: Omit<ComponentProps<typeof RoleDetail>, 'role' | 'draft' | 'dirty'> = {
  grantable: UNRESTRICTED,
  canUpdate: true,
  canDelete: true,
  canCreate: true,
  assignees: [],
  nameError: null,
  saving: false,
  onNameChange: vi.fn(),
  onDescriptionChange: vi.fn(),
  onLevelChange: vi.fn(),
  onActionToggle: vi.fn(),
  onCopy: vi.fn(),
  onSave: vi.fn(),
  onCancel: vi.fn(),
  onDelete: vi.fn(),
}

const draftFor = (role: Role, kind: 'new' | 'existing' = 'existing') => ({
  kind,
  id: kind === 'existing' ? role.id : undefined,
  name: role.name,
  description: role.description ?? '',
  permissions: new Set(role.permissions),
  clampedAway: [],
})

describe('RoleDetail', () => {
  it('shows the lock notice for a system role and no save bar', () => {
    render(
      <RoleDetail {...baseProps} role={template} draft={draftFor(template)} dirty={false} />,
    )
    expect(screen.getByText(/Systemrollen sind schreibgeschützt/)).toBeInTheDocument()
    expect(screen.getByText('Systemrolle')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument()
  })

  it('offers copying from both the header and the notice', async () => {
    const onCopy = vi.fn()
    render(
      <RoleDetail
        {...baseProps}
        onCopy={onCopy}
        role={template}
        draft={draftFor(template)}
        dirty={false}
      />,
    )
    await userEvent.click(screen.getByRole('button', { name: 'Kopieren & bearbeiten' }))
    await userEvent.click(screen.getByRole('button', { name: 'Kopiere' }))
    expect(onCopy).toHaveBeenCalledTimes(2)
  })

  it('offers no actionable copy affordance on a system role without create rights', () => {
    const onCopy = vi.fn()
    render(
      <RoleDetail
        {...baseProps}
        canCreate={false}
        onCopy={onCopy}
        role={template}
        draft={draftFor(template)}
        dirty={false}
      />,
    )
    expect(screen.queryByRole('button', { name: 'Kopieren & bearbeiten' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Kopiere' })).not.toBeInTheDocument()
    expect(screen.getByText(/Kopiere die Rolle/)).toBeInTheDocument()
    expect(onCopy).not.toHaveBeenCalled()
  })

  it('renders the name as read-only text for a system role', () => {
    render(<RoleDetail {...baseProps} role={template} draft={draftFor(template)} dirty={false} />)
    expect(screen.queryByRole('textbox', { name: 'Name der Rolle' })).not.toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'Routenplanung' })).toBeInTheDocument()
  })

  it('lets an own role be renamed and shows the save bar when dirty', async () => {
    const onNameChange = vi.fn()
    render(
      <RoleDetail
        {...baseProps}
        onNameChange={onNameChange}
        role={ownRole}
        draft={draftFor(ownRole)}
        dirty
      />,
    )
    await userEvent.type(screen.getByRole('textbox', { name: 'Name der Rolle' }), '!')
    expect(onNameChange).toHaveBeenCalled()
    expect(screen.getByRole('button', { name: 'Speichern' })).toBeInTheDocument()
  })

  it('labels the primary action Rolle anlegen for a new draft', () => {
    render(
      <RoleDetail {...baseProps} role={null} draft={draftFor(ownRole, 'new')} dirty />,
    )
    expect(screen.getByRole('button', { name: 'Rolle anlegen' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Speichern' })).not.toBeInTheDocument()
  })

  it('blocks saving on an empty name', () => {
    render(
      <RoleDetail
        {...baseProps}
        role={null}
        draft={{ ...draftFor(ownRole, 'new'), name: '' }}
        dirty
      />,
    )
    expect(screen.getByRole('button', { name: 'Rolle anlegen' })).toBeDisabled()
  })

  it('shows the name error at the field', () => {
    render(
      <RoleDetail
        {...baseProps}
        nameError="Eine Rolle mit diesem Namen existiert bereits."
        role={ownRole}
        draft={draftFor(ownRole)}
        dirty
      />,
    )
    expect(
      screen.getByText('Eine Rolle mit diesem Namen existiert bereits.'),
    ).toBeInTheDocument()
  })

  it('reports how many permissions the copy dropped', () => {
    render(
      <RoleDetail
        {...baseProps}
        role={null}
        draft={{ ...draftFor(ownRole, 'new'), clampedAway: ['role:delete', 'user:delete'] }}
        dirty
      />,
    )
    expect(
      screen.getByText('2 Rechte wurden entfernt, weil du sie selbst nicht besitzt.'),
    ).toBeInTheDocument()
  })

  it('lists unknown permissions instead of dropping them silently', () => {
    render(
      <RoleDetail
        {...baseProps}
        role={ownRole}
        draft={{
          ...draftFor(ownRole),
          permissions: new Set(['watering_plan:read', 'report:export']),
        }}
        dirty={false}
      />,
    )
    expect(screen.getByText(/Weitere Rechte \(1\)/)).toBeInTheDocument()
    expect(screen.getByText('report:export')).toBeInTheDocument()
  })

  it('groups the nine areas under three headings', () => {
    render(<RoleDetail {...baseProps} role={ownRole} draft={draftFor(ownRole)} dirty={false} />)
    expect(screen.getByText('Grünflächen')).toBeInTheDocument()
    expect(screen.getByText('Einsatzplanung')).toBeInTheDocument()
    expect(screen.getByText('Verwaltung')).toBeInTheDocument()
  })
})
