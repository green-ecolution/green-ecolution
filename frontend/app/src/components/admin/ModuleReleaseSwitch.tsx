import { useTranslation } from 'react-i18next'
import { Switch } from '@green-ecolution/ui'
import type { TenantModuleCell } from './tenantRows'

interface ModuleReleaseSwitchProps {
  moduleLabel: string
  tenantName: string
  cell: TenantModuleCell
  disabled: boolean
  onChange: (released: boolean) => void
}

const ModuleReleaseSwitch = ({
  moduleLabel,
  tenantName,
  cell,
  disabled,
  onChange,
}: ModuleReleaseSwitchProps) => {
  const { t } = useTranslation('admin')

  const state = !cell.released
    ? t('tenants.release.hidden', { name: tenantName })
    : cell.enabled
      ? t('tenants.release.enabledByTenant')
      : t('tenants.release.disabledByTenant')

  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center gap-2.5">
        <Switch
          checked={cell.released}
          disabled={disabled}
          onCheckedChange={onChange}
          aria-label={t('tenants.release.switchLabel', { module: moduleLabel, name: tenantName })}
        />
        <span className="font-bold">
          {cell.released ? t('tenants.release.released') : t('tenants.release.locked')}
        </span>
      </div>
      <span className="text-sm text-dark-600">{state}</span>
    </div>
  )
}

export default ModuleReleaseSwitch
