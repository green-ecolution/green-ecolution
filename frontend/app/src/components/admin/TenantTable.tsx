import { ArrowRight } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@green-ecolution/ui'
import { moduleManifests } from '@/modules/registry'
import ModuleReleaseSwitch from './ModuleReleaseSwitch'
import type { TenantModuleCell, TenantRow } from './tenantRows'

interface TenantTableProps {
  rows: TenantRow[]
  moduleKeys: string[] | null
  canRelease: boolean
  releasePending: boolean
  pendingOpen: boolean
  onOpen: (id: string) => void
  onRelease: (row: TenantRow, cell: TenantModuleCell, released: boolean) => void
}

const TenantTable = ({
  rows,
  moduleKeys,
  canRelease,
  releasePending,
  pendingOpen,
  onOpen,
  onRelease,
}: TenantTableProps) => {
  const { t, i18n } = useTranslation('admin')

  const moduleLabel = (key: string) => {
    const manifest = moduleManifests.find((entry) => entry.key === key)
    return manifest ? i18n.t(manifest.labelKey) : null
  }

  const stock = (row: TenantRow) => {
    const parts = [
      row.sensorCount !== null && t('tenants.summary.sensors', { count: row.sensorCount }),
      row.vehicleCount !== null && t('tenants.summary.vehicles', { count: row.vehicleCount }),
      row.planCount !== null && t('tenants.summary.plans', { count: row.planCount }),
    ].filter(Boolean)
    return parts.length > 0 ? parts.join(', ') : t('tenants.summary.pending')
  }

  return (
    <div className="overflow-hidden rounded-xl border border-dark-100 bg-white shadow-sm">
      <Table className="min-w-[900px]">
        <TableHeader>
          <TableRow className="hover:bg-transparent">
            <TableHead scope="col" className="px-6 font-bold">
              {t('tenants.columns.tenant')}
            </TableHead>
            <TableHead scope="col" className="font-bold">
              {t('tenants.columns.stock')}
            </TableHead>
            {moduleKeys?.map((key) => {
              const label = moduleLabel(key)
              return (
                <TableHead key={key} scope="col" className="min-w-52 py-3">
                  <span className="flex flex-col">
                    {label && <span className="font-bold text-dark">{label}</span>}
                    <span className="font-mono text-xs font-normal">{key}</span>
                  </span>
                </TableHead>
              )
            })}
            <TableHead scope="col" className="pr-6 text-right font-bold">
              {t('tenants.columns.actions')}
            </TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {rows.map((row) => (
            <TableRow key={row.id} className="hover:bg-transparent">
              <TableCell className="px-6 py-5">
                <div className="flex items-center gap-3">
                  <span
                    aria-hidden
                    className="flex size-10 shrink-0 items-center justify-center rounded-xl bg-green-dark font-bold text-white"
                  >
                    {row.name.charAt(0).toUpperCase()}
                  </span>
                  <div className="min-w-0">
                    <p className="font-lato text-base font-bold">{row.name}</p>
                    <p className="text-sm text-dark-600">
                      {t('tenants.summary.subOrganizations', { count: row.subOrganizationCount })}
                      {', '}
                      {t('tenants.summary.members', { count: row.memberCount })}
                    </p>
                  </div>
                </div>
              </TableCell>
              <TableCell className="whitespace-normal text-sm text-dark-600">
                {stock(row)}
              </TableCell>
              {moduleKeys?.map((key) => {
                const cell = row.modules?.find((entry) => entry.key === key)
                return (
                  <TableCell key={key}>
                    {cell && (
                      <ModuleReleaseSwitch
                        moduleLabel={moduleLabel(key) ?? key}
                        tenantName={row.name}
                        cell={cell}
                        disabled={!canRelease || releasePending}
                        onChange={(released) => onRelease(row, cell, released)}
                      />
                    )}
                  </TableCell>
                )
              })}
              <TableCell className="pr-6 text-right">
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => onOpen(row.id)}
                  disabled={pendingOpen}
                  aria-label={t('tenants.openSrLabel', { name: row.name })}
                >
                  {t('tenants.open')}
                  <ArrowRight aria-hidden className="size-4" />
                </Button>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  )
}

export default TenantTable
