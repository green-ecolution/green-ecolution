import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Checkbox } from '@green-ecolution/ui'
import { ACTIONS, type Permission } from '@/lib/auth/permissions'
import { permissionAreasFor } from '@/lib/auth/permissionAreas'

interface PluginPermissionMatrixProps {
  heading: string
  hint?: string
  permissions: ReadonlySet<string>
  disabled?: boolean
  onToggle: (permission: Permission) => void
}

/** Area name plus four action columns, wide enough for the longest short label. */
const ROW_COLUMNS = 'sm:grid sm:grid-cols-[minmax(0,1fr)_repeat(4,4.5rem)] sm:items-center'

/**
 * A `<fieldset>` with a `<legend>` gives the browser a role="group" whose
 * accessible name is the legend text — so the plugin matrix and the access
 * matrix are two distinctly-named groups a screen reader (and a test) can
 * address independently, without repeating a prefix on every cell.
 *
 * `min-w-0` is load-bearing: browsers give a fieldset `min-inline-size:
 * min-content`, so without it the widest row sets the floor and drags the whole
 * settings page sideways out of a phone viewport.
 *
 * One markup serves both layouts. Below `sm` a row is the area name over two
 * columns of labelled checkboxes; from `sm` the action wrapper turns into
 * `display: contents` and its children drop straight into the row grid, so the
 * checkboxes line up under the header without a second set of controls in the
 * DOM.
 */
const PluginPermissionMatrix = ({
  heading,
  hint,
  permissions,
  disabled = false,
  onToggle,
}: PluginPermissionMatrixProps) => {
  const { t } = useTranslation('settings')
  const hintId = useId()
  const fieldId = useId()
  const areas = permissionAreasFor(t)
  const actionShort: Record<(typeof ACTIONS)[number], string> = {
    read: t('plugin.permissionMatrix.actionShort.read'),
    create: t('plugin.permissionMatrix.actionShort.create'),
    update: t('plugin.permissionMatrix.actionShort.update'),
    delete: t('plugin.permissionMatrix.actionShort.delete'),
  }

  return (
    <fieldset aria-describedby={hint ? hintId : undefined} className="flex min-w-0 flex-col gap-2">
      <legend className="font-lato text-sm font-semibold text-dark">{heading}</legend>
      {hint && (
        <p id={hintId} className="-mt-1 text-sm text-dark-600">
          {hint}
        </p>
      )}

      <div className="overflow-hidden rounded-xl border border-dark-100">
        <div
          className={`hidden border-b border-dark-100 bg-dark-50 px-4 py-2.5 text-sm text-dark-600 ${ROW_COLUMNS}`}
        >
          <span>{t('plugin.permissionMatrix.areaColumnLabel')}</span>
          {ACTIONS.map((action) => (
            <span key={action} className="text-center">
              {actionShort[action]}
            </span>
          ))}
        </div>

        <ul className="divide-y divide-dark-100">
          {areas.map((area) => (
            <li key={area.resource} className={`px-4 py-3 sm:py-2 ${ROW_COLUMNS}`}>
              <span className="text-sm font-medium text-dark">{area.label}</span>
              <div className="mt-2.5 grid grid-cols-2 gap-x-4 gap-y-2.5 sm:contents">
                {area.actions.map((action) => {
                  const checkboxId = `${fieldId}-${action.permission.replace(':', '-')}`
                  return (
                    <div
                      key={action.permission}
                      className="flex items-center gap-2.5 sm:justify-center"
                    >
                      <Checkbox
                        id={checkboxId}
                        aria-label={`${area.label} ${actionShort[action.action]}`}
                        checked={permissions.has(action.permission)}
                        disabled={disabled}
                        onCheckedChange={() => onToggle(action.permission)}
                      />
                      <label htmlFor={checkboxId} className="text-sm text-dark-600 sm:hidden">
                        {actionShort[action.action]}
                      </label>
                    </div>
                  )
                })}
              </div>
            </li>
          ))}
        </ul>
      </div>
    </fieldset>
  )
}

export default PluginPermissionMatrix
