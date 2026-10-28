import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { modulesQuery, organizationQueries } from '@/api/queries'
import EntityList from '@/components/general/EntityList'
import ListPageHeader from '@/components/general/ListPageHeader'
import createToast from '@/hooks/createToast'
import { getOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { useSwitchOrganizationScope } from '@/lib/auth/scope/useSwitchOrganizationScope'
import TenantCard from './TenantCard'
import { tenantRows } from './tenantRows'

const TenantOverview = () => {
  const { t } = useTranslation('admin')
  const { data: organizations } = useSuspenseQuery(organizationQueries.list())
  const { data: modules } = useQuery(modulesQuery())
  const { data: counts } = useQuery(organizationQueries.resourceCounts())
  const switchScope = useSwitchOrganizationScope()
  const navigate = useNavigate()
  const showToast = createToast()
  const [pendingId, setPendingId] = useState<string | null>(null)

  const rows = tenantRows(organizations, modules, counts)

  // The scope switch resets the queries; the dashboard then loads as the tenant.
  const openTenant = async (id: string) => {
    if (pendingId) return
    setPendingId(id)
    try {
      await switchScope(id)
      // The switch is a silent no-op while the user is still unknown.
      if (getOrganizationScope() !== id) throw new Error('organization scope was not applied')
      await navigate({ to: '/dashboard' })
    } catch {
      showToast(t('tenants.openError'), 'error')
    } finally {
      setPendingId(null)
    }
  }

  return (
    <div className="container mt-6">
      <ListPageHeader title={t('tenants.title')} description={t('tenants.description')} />
      <EntityList
        items={rows}
        getKey={(row) => row.id}
        emptyMessage={t('tenants.empty')}
        renderItem={(row) => (
          <TenantCard
            tenant={row}
            onOpen={() => void openTenant(row.id)}
            pending={pendingId !== null}
          />
        )}
      />
      {rows.length === 0 && (
        <p className="mt-4 text-center">
          <Link to="/settings/organization" className="text-green-dark underline">
            {t('tenants.emptyLink')}
          </Link>
        </p>
      )}
    </div>
  )
}

export default TenantOverview
