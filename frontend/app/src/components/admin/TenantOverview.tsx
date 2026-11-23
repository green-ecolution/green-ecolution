import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { Info, MoveRight, TriangleAlert, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Loading,
} from '@green-ecolution/ui'
import { moduleReleasesQuery, organizationQueries } from '@/api/queries'
import ListPageHeader from '@/components/general/ListPageHeader'
import createToast from '@/hooks/createToast'
import { useUpdateModuleRelease } from '@/hooks/useModuleReleases'
import { statusOf } from '@/lib/httpError'
import { satisfies } from '@/lib/auth/permissions'
import { getOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { useSwitchOrganizationScope } from '@/lib/auth/scope/useSwitchOrganizationScope'
import { usePermissionsIn } from '@/lib/auth/usePermissionsIn'
import { moduleManifests } from '@/modules/registry'
import TenantTable from './TenantTable'
import { tenantRows, type TenantModuleCell, type TenantRow } from './tenantRows'

interface PendingLock {
  row: TenantRow
  cell: TenantModuleCell
}

const TenantOverview = () => {
  const { t, i18n } = useTranslation('admin')
  const { data: organizations } = useSuspenseQuery(organizationQueries.list())
  // A 403 (no right at the root) quietly leaves out the module columns; other failures say so.
  const { data: releases, error: releasesError } = useQuery(moduleReleasesQuery())
  const releasesFailed = !!releasesError && statusOf(releasesError) !== 403
  const { data: counts } = useQuery(organizationQueries.resourceCounts())
  const switchScope = useSwitchOrganizationScope()
  const navigate = useNavigate()
  const showToast = createToast()
  const updateRelease = useUpdateModuleRelease({
    onError: () => showToast(t('tenants.release.saveError'), 'error'),
  })
  const [pendingId, setPendingId] = useState<string | null>(null)
  const [pendingLock, setPendingLock] = useState<PendingLock | null>(null)

  const root = organizations.find((org) => org.parentId == null)
  const canRelease = satisfies(usePermissionsIn(root?.id ?? null), ['setting:update'])
  const rows = tenantRows(organizations, releases, counts)

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

  const changeRelease = (row: TenantRow, key: string, released: boolean) => {
    updateRelease.mutate({ orgId: row.id, key, released })
  }

  const requestRelease = (row: TenantRow, cell: TenantModuleCell, released: boolean) => {
    if (!released && cell.released && cell.enabled) {
      setPendingLock({ row, cell })
      return
    }
    changeRelease(row, cell.key, released)
  }

  const moduleLabel = (key: string) => {
    const manifest = moduleManifests.find((entry) => entry.key === key)
    return manifest ? i18n.t(manifest.labelKey) : key
  }

  // Under a tenant scope the list has no root until the router sends the user on.
  if (rows === null) {
    return <Loading className="mt-20 justify-center" label={t('tenants.loadingLabel')} />
  }

  return (
    <div className="container mt-6">
      <ListPageHeader title={t('tenants.title')} description={t('tenants.description')} />
      {rows.length === 0 ? (
        <>
          <p className="mt-8 text-center text-dark-600">{t('tenants.empty')}</p>
          <p className="mt-4 text-center">
            <Link to="/settings/organization" className="text-green-dark underline">
              {t('tenants.emptyLink')}
            </Link>
          </p>
        </>
      ) : (
        <>
          <TenantTable
            rows={rows}
            moduleKeys={releases?.modules ?? null}
            canRelease={canRelease}
            releasePending={updateRelease.isPending}
            pendingOpen={pendingId !== null}
            onOpen={(id) => void openTenant(id)}
            onRelease={requestRelease}
          />
          {releasesFailed && (
            <div role="alert" className="mt-6 flex max-w-prose gap-3 text-dark-600">
              <TriangleAlert aria-hidden className="mt-0.5 size-5 shrink-0" />
              <p>{t('tenants.release.loadError')}</p>
            </div>
          )}
          {releases && (
            <div className="mt-6 flex max-w-prose gap-3 text-dark-600">
              <Info aria-hidden className="mt-0.5 size-5 shrink-0" />
              <p>{t('tenants.release.note')}</p>
            </div>
          )}
        </>
      )}

      <AlertDialog
        open={pendingLock !== null}
        onOpenChange={(open) => !open && setPendingLock(null)}
      >
        <AlertDialogContent>
          {pendingLock && (
            <>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  {t('tenants.release.lockDialog.title', {
                    module: moduleLabel(pendingLock.cell.key),
                    name: pendingLock.row.name,
                  })}
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {t('tenants.release.lockDialog.description', {
                    module: moduleLabel(pendingLock.cell.key),
                    name: pendingLock.row.name,
                  })}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>
                  {t('tenants.release.lockDialog.cancel')}
                  <X />
                </AlertDialogCancel>
                <AlertDialogAction
                  onClick={() => changeRelease(pendingLock.row, pendingLock.cell.key, false)}
                >
                  {t('tenants.release.lockDialog.confirm')}
                  <MoveRight className="icon-arrow-animate" />
                </AlertDialogAction>
              </AlertDialogFooter>
            </>
          )}
        </AlertDialogContent>
      </AlertDialog>
    </div>
  )
}

export default TenantOverview
