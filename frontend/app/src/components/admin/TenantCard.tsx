import { ArrowLeftRight, ArrowRight, Building2, Car, Users } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Badge, Button, ListCard } from '@green-ecolution/ui'
import SensorIcon from '@/components/icons/Sensor'
import { moduleManifests } from '@/modules/registry'
import type { TenantRow } from './tenantRows'

interface TenantCardProps {
  tenant: TenantRow
  onOpen: () => void
}

interface FactProps {
  icon: React.ReactNode
  label: string
  value: number | null
  pending: string
}

const Fact = ({ icon, label, value, pending }: FactProps) => (
  <span className="flex items-center gap-2">
    {icon}
    <span className="sr-only">{label}</span>
    <span className="tabular-nums">{value ?? pending}</span>
    <span aria-hidden className="text-dark-500">
      {label}
    </span>
  </span>
)

const TenantCard = ({ tenant, onOpen }: TenantCardProps) => {
  const { t, i18n } = useTranslation('admin')
  const pending = t('tenants.card.pending')
  const moduleLabel = (key: string) => {
    const manifest = moduleManifests.find((entry) => entry.key === key)
    return manifest ? i18n.t(manifest.labelKey) : key
  }

  return (
    <ListCard hoverable={false} className="@container">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <div className="min-w-0">
          <h2 className="font-lato text-lg font-bold">{tenant.name}</h2>
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <span className="sr-only">{t('tenants.card.modules')}</span>
            {tenant.activeModules.length === 0 ? (
              <Badge variant="muted">{t('tenants.card.noModules')}</Badge>
            ) : (
              tenant.activeModules.map((key) => (
                <Badge key={key} variant="outline-dark">
                  {moduleLabel(key)}
                </Badge>
              ))
            )}
          </div>
        </div>
        <Button
          type="button"
          variant="outline"
          onClick={onOpen}
          aria-label={t('tenants.openSrLabel', { name: tenant.name })}
        >
          {t('tenants.open')}
          <ArrowRight aria-hidden className="size-4" />
        </Button>
      </div>

      <div className="flex flex-wrap items-center gap-x-6 gap-y-2 text-sm text-dark-600">
        <Fact
          icon={<Building2 aria-hidden className="size-4" />}
          label={t('tenants.card.subOrganizations')}
          value={tenant.subOrganizationCount}
          pending={pending}
        />
        <Fact
          icon={<Users aria-hidden className="size-4" />}
          label={t('tenants.card.members')}
          value={tenant.memberCount}
          pending={pending}
        />
        <Fact
          icon={<SensorIcon aria-hidden className="size-4" />}
          label={t('tenants.card.sensors')}
          value={tenant.sensorCount}
          pending={pending}
        />
        <Fact
          icon={<Car aria-hidden className="size-4" />}
          label={t('tenants.card.vehicles')}
          value={tenant.vehicleCount}
          pending={pending}
        />
        <Fact
          icon={<ArrowLeftRight aria-hidden className="size-4" />}
          label={t('tenants.card.plans')}
          value={tenant.planCount}
          pending={pending}
        />
      </div>
    </ListCard>
  )
}

export default TenantCard
