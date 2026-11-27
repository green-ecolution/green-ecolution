import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Plus } from 'lucide-react'
import {
  Badge,
  Button,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@green-ecolution/ui'
import { modulesQuery, userQueries } from '@/api/queries'
import { useOrganizationMutations } from '@/hooks/useOrganizationMutations'
import { statusOf } from '@/lib/httpError'
import { initialsOfName } from '@/lib/initials'
import { moduleManifests } from '@/modules/registry'
import { CARD } from '../cardStyles'
import CreateOrganizationDialog from './CreateOrganizationDialog'
import StructureEmptyState from './StructureEmptyState'
import { structureRows } from './structureRows'
import { useSettingsOrganization } from './useSettingsOrganization'

const CONTACTS_PER_PAGE = 100

const OrganizationStructurePage = () => {
  const { t, i18n } = useTranslation('settings')
  const navigate = useNavigate()
  const { orgId, organizations, readableIds, can } = useSettingsOrganization()
  const { createOrganization } = useOrganizationMutations()
  const [createOpen, setCreateOpen] = useState(false)

  const { data: moduleList } = useQuery(modulesQuery())
  // Only resolves contact names; a person beyond the first page stays unnamed.
  const { data: people } = useQuery({
    ...userQueries.list({ page: 1, perPage: CONTACTS_PER_PAGE }),
    enabled: can(['user:read']),
  })

  const activeByOrg = new Map(
    (moduleList?.organizations ?? []).map((entry) => [entry.organizationId, entry.active]),
  )
  // Organizations only reachable carry no member count or contact person to show.
  const rows = structureRows(
    organizations.filter((org) => readableIds.has(org.id)),
    activeByOrg,
  )
  const nameOfPerson = (id: string | null): string | null => {
    const person = (people?.data ?? []).find((user) => user.id === id)
    return person ? `${person.firstName} ${person.lastName}` : null
  }
  const labelOfModule = (key: string): string => {
    const manifest = moduleManifests.find((entry) => entry.key === key)
    return manifest ? i18n.t(manifest.labelKey) : key
  }
  const empty = rows.length <= 1
  const selectedName = organizations.find((org) => org.id === orgId)?.name ?? ''

  const openCreate = () => {
    createOrganization.reset()
    setCreateOpen(true)
  }

  const create = (name: string) => {
    createOrganization.mutate(
      { parentId: orgId, name },
      {
        onSuccess: (created) => {
          setCreateOpen(false)
          void navigate({ to: '/settings/organization/$orgId', params: { orgId: created.id } })
        },
      },
    )
  }

  return (
    <div className="flex flex-col gap-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <header className="min-w-0">
          <h2 className="font-lato text-2xl font-bold text-dark">
            {t('organization.structure.title')}
          </h2>
          {!empty && (
            <p className="mt-1 max-w-[62ch] text-sm text-dark-600">
              {t('organization.structure.intro')}
            </p>
          )}
        </header>
        {!empty && can(['organization:create']) && (
          <Button
            type="button"
            size="sm"
            className="w-full sm:w-auto sm:shrink-0"
            onClick={openCreate}
          >
            <Plus className="size-4" aria-hidden />
            {t('organization.createSubOrganization')}
          </Button>
        )}
      </div>

      {empty ? (
        <StructureEmptyState
          organizationName={selectedName}
          onCreate={can(['organization:create']) ? openCreate : undefined}
        />
      ) : (
        <div className={`${CARD} overflow-x-auto p-0`}>
          <Table className="min-w-[44rem]">
            <TableHeader>
              <TableRow>
                <TableHead>{t('organization.structure.columnOrganization')}</TableHead>
                <TableHead>{t('organization.structure.columnMembers')}</TableHead>
                <TableHead>{t('organization.structure.columnContact')}</TableHead>
                <TableHead>{t('organization.structure.columnModules')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {rows.map((row) => (
                <TableRow key={row.id} aria-current={row.id === orgId ? 'true' : undefined}>
                  <TableCell>
                    <Link
                      to="/settings/organization/$orgId"
                      params={{ orgId: row.id }}
                      className="flex items-center gap-3 rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                      style={{ paddingLeft: `${row.depth * 1.5}rem` }}
                    >
                      <span
                        className="flex size-8 shrink-0 items-center justify-center rounded-lg bg-green-light-100 text-sm font-semibold text-green-dark"
                        aria-hidden
                      >
                        {initialsOfName(row.name)}
                      </span>
                      <span className="min-w-0">
                        <span className="block truncate font-lato text-sm font-semibold text-dark">
                          {row.name}
                        </span>
                        {/* The space keeps name and marker apart in the link's accessible name. */}
                        {row.id === orgId && (
                          <>
                            {' '}
                            <span className="block text-xs text-dark-600">
                              {t('organization.structure.current')}
                            </span>
                          </>
                        )}
                      </span>
                    </Link>
                  </TableCell>
                  <TableCell>
                    {row.memberTotal === row.memberDirect
                      ? row.memberTotal
                      : t('organization.structure.membersWithDirect', {
                          total: row.memberTotal,
                          direct: row.memberDirect,
                        })}
                  </TableCell>
                  <TableCell>
                    {nameOfPerson(row.contactPersonId) ??
                      (row.contactPersonId ? (
                        <span className="text-dark-600">
                          <span aria-hidden="true">–</span>
                          <span className="sr-only">
                            {t('organization.structure.contactUnresolved')}
                          </span>
                        </span>
                      ) : (
                        <span className="text-dark-600">
                          {t('organization.structure.noContact')}
                        </span>
                      ))}
                  </TableCell>
                  <TableCell>
                    {row.moduleKeys.length === 0 ? (
                      <span className="text-dark-600">{t('organization.structure.noModules')}</span>
                    ) : (
                      <span className="flex flex-wrap gap-1.5">
                        {row.moduleKeys.map((key) => (
                          <Badge key={key} variant="muted">
                            {labelOfModule(key)}
                          </Badge>
                        ))}
                      </span>
                    )}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </div>
      )}

      <CreateOrganizationDialog
        open={createOpen}
        parentName={selectedName}
        saving={createOrganization.isPending}
        nameError={
          statusOf(createOrganization.error) === 409 ? t('organization.nameConflict') : null
        }
        onOpenChange={setCreateOpen}
        onSubmit={create}
      />
    </div>
  )
}

export default OrganizationStructurePage
