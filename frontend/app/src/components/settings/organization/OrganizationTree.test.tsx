import { describe, it, expect, vi } from 'vitest'
import { render, screen, userEvent, within } from '@/test/utils'
import type { OrganizationResponse } from '@/api/backendApi'
import OrganizationTree from './OrganizationTree'
import { buildTree } from './organizationTree'

const org = (
  id: string,
  name: string,
  memberCount: number,
  parentId?: string,
): OrganizationResponse => ({ id, name, memberCount, parentId })

const rootOf = (orgs: OrganizationResponse[]) => {
  const root = buildTree(orgs, 'amt')
  if (!root) throw new Error('tree fixture is broken')
  return root
}

const renderTree = (orgs: OrganizationResponse[], expanded: string[] = ['amt']) => {
  const onSelect = vi.fn()
  const onToggle = vi.fn()
  render(
    <OrganizationTree
      root={rootOf(orgs)}
      selectedId="amt"
      expanded={new Set(expanded)}
      canCreate
      onSelect={onSelect}
      onToggle={onToggle}
      onCreate={vi.fn()}
    />,
  )
  return { onSelect, onToggle }
}

const list = () => within(screen.getByRole('list', { name: 'Organisationsstruktur' }))

describe('OrganizationTree', () => {
  it('pluralises people and teams in a node subtitle', () => {
    renderTree([
      org('amt', 'Grünflächenamt', 2),
      org('nord', 'Nord', 3, 'amt'),
      org('sued', 'Süd', 4, 'amt'),
    ])

    // 2 own + 3 + 4, below it two child organizations
    expect(list().getByText('9 Personen · 2 Teams')).toBeInTheDocument()
  })

  it('uses the singular for a single person and a single team', () => {
    renderTree([org('amt', 'Grünflächenamt', 0), org('nord', 'Nord', 1, 'amt')])

    expect(list().getByText('1 Person · 1 Team')).toBeInTheDocument()
  })

  it('omits the subtitle for a node whose subtree has no members', () => {
    renderTree([org('amt', 'Grünflächenamt', 0)])

    expect(list().queryByText(/Person/)).not.toBeInTheDocument()
  })

  it('toggles a branch without selecting it', async () => {
    const { onSelect, onToggle } = renderTree([
      org('amt', 'Grünflächenamt', 1),
      org('nord', 'Nord', 1, 'amt'),
      org('duburg', 'Duburg', 1, 'nord'),
    ])

    await userEvent.click(list().getByRole('button', { name: 'Aufklappen' }))

    expect(onToggle).toHaveBeenCalledWith('nord')
    expect(onSelect).not.toHaveBeenCalled()
  })
})
