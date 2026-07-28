import { useId, useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { Badge, SegmentedControl } from '@green-ecolution/ui'
import type { Permission, Permissions } from '@/lib/auth/permissions'
import {
  ACCESS_LEVELS,
  activeActionCount,
  isGrantable,
  levelOf,
  LEVEL_LABELS,
  type AccessLevel,
  type PermissionArea,
} from '@/lib/auth/permissionAreas'
import PermissionToggle from './PermissionToggle'

const LEVEL_OPTIONS = ACCESS_LEVELS.map((level) => ({ value: level, label: LEVEL_LABELS[level] }))

const NOT_GRANTABLE = 'Du kannst nur Rechte vergeben, die du selbst besitzt.'

interface RoleAreaCardProps {
  area: PermissionArea
  permissions: ReadonlySet<string>
  grantable: Permissions
  readOnly?: boolean
  onLevelChange: (level: AccessLevel) => void
  onActionToggle: (permission: Permission) => void
}

const RoleAreaCard = ({
  area,
  permissions,
  grantable,
  readOnly = false,
  onLevelChange,
  onActionToggle,
}: RoleAreaCardProps) => {
  const [expanded, setExpanded] = useState(false)
  const panelId = useId()

  const level = levelOf(area.resource, permissions)
  const count = activeActionCount(area.resource, permissions)

  return (
    <div className="relative overflow-hidden rounded-xl border border-dark-50 bg-white shadow-cards">
      {/* Permission spine: fills with the active action count so a role's shape
          is readable down the column before any label is read. */}
      <span
        aria-hidden
        className="absolute bottom-3 left-0 top-3 w-[3px] overflow-hidden rounded-full bg-dark-100"
      >
        <span
          className={
            level === 'custom'
              ? 'absolute bottom-0 left-0 w-full bg-[repeating-linear-gradient(to_top,var(--green-dark)_0_4px,transparent_4px_8px)] transition-[height] duration-200 motion-reduce:transition-none'
              : 'absolute bottom-0 left-0 w-full bg-green-dark transition-[height] duration-200 motion-reduce:transition-none'
          }
          style={{ height: `${(count / area.actions.length) * 100}%` }}
        />
      </span>

      <div className="flex flex-wrap items-center gap-4 px-5 py-4">
        <div className="min-w-0 flex-1">
          <p className="font-lato text-base font-semibold text-dark">{area.label}</p>
          <p className="mt-0.5 text-sm text-dark-600">{area.description}</p>
          <p className="mt-1 flex items-center gap-2 text-xs text-dark-500">
            <span>
              {count} von {area.actions.length} Aktionen aktiv
            </span>
            {level === 'custom' && (
              <Badge variant="muted" className="font-normal">
                Individuell
              </Badge>
            )}
          </p>
        </div>

        <SegmentedControl
          options={LEVEL_OPTIONS}
          value={level === 'custom' ? null : level}
          onChange={onLevelChange}
          ariaLabel={`Zugriffsstufe für ${area.label}`}
          disabled={readOnly}
        />

        <button
          type="button"
          aria-expanded={expanded}
          aria-controls={panelId}
          onClick={() => setExpanded((open) => !open)}
          className="rounded-lg p-1.5 text-dark-500 transition-colors hover:bg-dark-50 hover:text-dark focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          <span className="sr-only">
            {expanded ? `Aktionen für ${area.label} verbergen` : `Aktionen für ${area.label} anzeigen`}
          </span>
          <ChevronDown
            aria-hidden
            className={`size-5 transition-transform duration-200 motion-reduce:transition-none ${expanded ? 'rotate-180' : ''}`}
          />
        </button>
      </div>

      <div
        id={panelId}
        className={`grid transition-[grid-template-rows] duration-200 motion-reduce:transition-none ${expanded ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'}`}
      >
        <div className="overflow-hidden">
          <div className="border-t border-dark-50">
            {expanded &&
              area.actions.map((action) => {
                const grantableAction = isGrantable(action.permission, grantable)
                return (
                  <PermissionToggle
                    key={action.permission}
                    label={action.label}
                    hint={action.hint}
                    checked={permissions.has(action.permission)}
                    disabled={readOnly || !grantableAction}
                    disabledReason={grantableAction ? undefined : NOT_GRANTABLE}
                    onCheckedChange={() => onActionToggle(action.permission)}
                  />
                )
              })}
          </div>
        </div>
      </div>
    </div>
  )
}

export default RoleAreaCard
