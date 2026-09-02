import { useId, useState } from 'react'
import { ChevronDown } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, SegmentedControl } from '@green-ecolution/ui'
import type { Permission, Permissions } from '@/lib/auth/permissions'
import {
  ACCESS_LEVELS,
  activeActionCount,
  isGrantable,
  levelOf,
  levelLabels,
  type AccessLevel,
  type PermissionArea,
} from '@/lib/auth/permissionAreas'
import PermissionToggle from './PermissionToggle'

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
  const { t } = useTranslation('settings')
  const [expanded, setExpanded] = useState(false)
  const panelId = useId()

  const level = levelOf(area.resource, permissions)
  const count = activeActionCount(area.resource, permissions)
  const levelOptions = ACCESS_LEVELS.map((option) => ({
    value: option,
    label: levelLabels(t)[option],
  }))
  const notGrantable = t('roles.notGrantable')

  return (
    <div className="@container overflow-hidden rounded-xl border border-dark-50 bg-white shadow-cards">
      <div className="flex flex-col gap-3 px-5 py-4 @min-[30rem]:flex-row @min-[30rem]:flex-wrap @min-[30rem]:items-center @min-[30rem]:gap-4">
        <div className="min-w-0 @min-[30rem]:flex-1">
          <p className="font-lato text-base font-semibold text-dark">{area.label}</p>
          <p className="mt-0.5 text-sm text-dark-600">{area.description}</p>
          <p className="mt-1 flex flex-wrap items-center gap-2 text-xs text-dark-500">
            <span className="whitespace-nowrap">
              {t('roles.areaActiveCount', { count, total: area.actions.length })}
            </span>
            {level === 'custom' && (
              <Badge variant="muted" className="font-normal">
                {t('roles.customBadge')}
              </Badge>
            )}
          </p>
        </div>

        <div className="flex items-center justify-between gap-2 @min-[30rem]:justify-end">
          <SegmentedControl
            options={levelOptions}
            value={level === 'custom' ? null : level}
            onChange={onLevelChange}
            ariaLabel={t('roles.accessLevelAriaLabel', { area: area.label })}
            disabled={readOnly}
          />

          <button
            type="button"
            aria-expanded={expanded}
            aria-controls={panelId}
            onClick={() => setExpanded((open) => !open)}
            className="shrink-0 rounded-lg p-1.5 text-dark-500 transition-colors hover:bg-dark-50 hover:text-dark focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
          >
            <span className="sr-only">
              {t(expanded ? 'roles.hideActionsAriaLabel' : 'roles.showActionsAriaLabel', {
                area: area.label,
              })}
            </span>
            <ChevronDown
              aria-hidden
              className={`size-5 transition-transform duration-200 motion-reduce:transition-none ${expanded ? 'rotate-180' : ''}`}
            />
          </button>
        </div>
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
                    disabledReason={grantableAction ? undefined : notGrantable}
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
