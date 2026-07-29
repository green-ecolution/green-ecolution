import { ChevronDown, ChevronRight } from 'lucide-react'
import { ListCard } from '@green-ecolution/ui'
import type { OrganizationResponse } from '@/api/backendApi'
import type { OrgNode } from './organizationTree'
import { subtreeMemberCount } from './organizationTree'

interface OrganizationTreeItemProps {
  node: OrgNode
  depth: number
  selectedId: string | null
  expanded: ReadonlySet<string>
  onSelect: (org: OrganizationResponse) => void
  onToggle: (orgId: string) => void
}

// One step matches the disclosure control's own width, so a child's chevron
// lines up directly under its parent's tile.
const INDENT_STEP_REM = 1.25

const initialsOf = (name: string): string =>
  name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')

const subtitleOf = (node: OrgNode): string | null => {
  const memberCount = subtreeMemberCount(node)
  if (memberCount === 0) return null
  const teams = node.children.length > 0 ? ` · ${node.children.length} Teams` : ''
  return `${memberCount} Personen${teams}`
}

const OrganizationTreeItem = ({
  node,
  depth,
  selectedId,
  expanded,
  onSelect,
  onToggle,
}: OrganizationTreeItemProps) => {
  const hasChildren = node.children.length > 0
  const isExpanded = expanded.has(node.org.id)
  const isSelected = node.org.id === selectedId
  const subtitle = subtitleOf(node)

  return (
    <li>
      <div
        className="flex items-center gap-1"
        style={{ paddingLeft: `${depth * INDENT_STEP_REM}rem` }}
      >
        {hasChildren ? (
          <button
            type="button"
            onClick={(event) => {
              event.stopPropagation()
              onToggle(node.org.id)
            }}
            aria-label={isExpanded ? 'Zuklappen' : 'Aufklappen'}
            className="flex size-5 shrink-0 items-center justify-center rounded text-dark-500 transition-colors hover:bg-dark-50 hover:text-dark focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            {isExpanded ? (
              <ChevronDown className="size-4" aria-hidden />
            ) : (
              <ChevronRight className="size-4" aria-hidden />
            )}
          </button>
        ) : (
          <span aria-hidden className="size-5 shrink-0" />
        )}

        <ListCard
          size="compact"
          hoverable={!isSelected}
          asChild
          className={isSelected ? 'border-green-dark bg-green-dark-50' : undefined}
        >
          <button
            type="button"
            onClick={() => onSelect(node.org)}
            aria-current={isSelected}
            className="min-w-0 flex-1 text-left"
          >
            <span
              className={`flex size-8 shrink-0 items-center justify-center rounded-lg text-sm font-semibold ${
                isSelected ? 'bg-green-dark text-white' : 'bg-green-light-100 text-green-dark'
              }`}
            >
              {initialsOf(node.org.name)}
            </span>

            <span className="min-w-0 flex-1">
              <span className="block truncate font-lato text-sm font-semibold text-dark">
                {node.org.name}
              </span>
              {subtitle && <span className="block truncate text-xs text-dark-600">{subtitle}</span>}
            </span>
          </button>
        </ListCard>
      </div>

      {hasChildren && isExpanded && (
        <ul className="mt-1 flex list-none flex-col gap-1">
          {node.children.map((child) => (
            <OrganizationTreeItem
              key={child.org.id}
              node={child}
              depth={depth + 1}
              selectedId={selectedId}
              expanded={expanded}
              onSelect={onSelect}
              onToggle={onToggle}
            />
          ))}
        </ul>
      )}
    </li>
  )
}

export default OrganizationTreeItem
