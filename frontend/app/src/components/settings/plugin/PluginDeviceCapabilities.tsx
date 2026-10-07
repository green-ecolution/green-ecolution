import { useId } from 'react'
import { useTranslation } from 'react-i18next'
import { Checkbox } from '@green-ecolution/ui'
import { CAPABILITIES } from './deviceCapabilities'

interface PluginDeviceCapabilitiesProps {
  capabilities: ReadonlySet<string>
  onToggle: (capability: string) => void
  disabled?: boolean
}

export function PluginDeviceCapabilities({
  capabilities,
  onToggle,
  disabled,
}: PluginDeviceCapabilitiesProps) {
  const { t } = useTranslation('settings')
  const baseId = useId()
  const hintId = `${baseId}-hint`

  return (
    <fieldset aria-describedby={hintId} className="flex min-w-0 flex-col gap-2">
      <legend className="font-lato text-sm font-semibold text-dark">
        {t('plugin.install.deviceCapabilitiesHeading')}
      </legend>
      <p id={hintId} className="-mt-1 text-sm text-dark-600">
        {t('plugin.install.deviceCapabilitiesHint')}
      </p>
      <div className="flex flex-wrap gap-6">
        {CAPABILITIES.map((capability) => {
          const id = `${baseId}-${capability}`
          return (
            <div key={capability} className="flex items-center gap-2">
              <Checkbox
                id={id}
                checked={capabilities.has(capability)}
                disabled={disabled}
                onCheckedChange={() => onToggle(capability)}
              />
              <label htmlFor={id} className="text-sm text-dark-600">
                {t(`plugin.install.deviceCapability.${capability}`)}
              </label>
            </div>
          )
        })}
      </div>
    </fieldset>
  )
}

export default PluginDeviceCapabilities
