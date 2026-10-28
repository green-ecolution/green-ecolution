import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { ArrowRightLeft, Pencil, Star, Trash2 } from 'lucide-react'
import { Badge, Button, ListCard, ListCardDescription, ListCardTitle } from '@green-ecolution/ui'
import type { Organization, StartPointResponse } from '@/api/backendApi'
import { Can } from '@/lib/auth/Can'
import { capacityDisplay } from '@/lib/capacity'
import { useOrganizationsWithPermission } from '@/lib/auth/useOrganizationsWithPermission'
import EntityList from '@/components/general/EntityList'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import TransferOrganizationDialog from '@/components/general/transfer/TransferOrganizationDialog'

interface StationListProps {
  stations: StartPointResponse[]
  organizations: Organization[]
  onEdit: (station: StartPointResponse) => void
  onSetDefault: (station: StartPointResponse) => void
  onDelete: (station: StartPointResponse) => void
  onTransfer: (station: StartPointResponse, organizationId: string) => void
}

const StationList = ({
  stations,
  organizations,
  onEdit,
  onSetDefault,
  onDelete,
  onTransfer,
}: StationListProps) => {
  const { t } = useTranslation('settings')
  const [pendingDelete, setPendingDelete] = useState<StartPointResponse | null>(null)
  const [pendingTransfer, setPendingTransfer] = useState<StartPointResponse | null>(null)
  const writableOrganizations = useOrganizationsWithPermission('start_point:update')

  const organizationName = (id: string) => organizations.find((org) => org.id === id)?.name ?? id
  const transferTargetsOf = (station: StartPointResponse) =>
    writableOrganizations.filter((org) => org.id !== station.organizationId)
  // The default is unique per organization, so moving it could collide with
  // the target's own default. Only non-default stations are offered.
  const canTransfer = (station: StartPointResponse) =>
    !station.isDefault && transferTargetsOf(station).length > 0

  return (
    <>
      <EntityList
        items={stations}
        getKey={(station) => station.id}
        emptyMessage={t('stations.empty')}
        renderItem={(station) => (
          <ListCard size="compact" hoverable={false} className="flex-wrap">
            <div className="min-w-0 flex-1 basis-48">
              <ListCardTitle className="mb-0.5 truncate">{station.name}</ListCardTitle>
              <ListCardDescription className="truncate">
                {organizationName(station.organizationId)}
              </ListCardDescription>
            </div>

            <div className="flex flex-wrap items-center gap-2">
              {station.isDefault && <Badge variant="success">{t('stations.defaultBadge')}</Badge>}
              {station.replenishes.length === 0 ? (
                <span className="text-sm text-dark-600">{t('stations.noReplenishes')}</span>
              ) : (
                station.replenishes.map((kind) => {
                  const { icon: Icon } = capacityDisplay[kind]
                  return (
                    <Badge key={kind} variant="muted">
                      <Icon className="size-3.5" aria-hidden />
                      {t(`stations.kind.${kind}`)}
                    </Badge>
                  )
                })
              )}
            </div>

            <div className="flex items-center gap-1">
              <Can permission={['start_point:update']}>
                <Button
                  type="button"
                  variant="ghost"
                  size="icon"
                  aria-label={t('stations.action.edit', { name: station.name })}
                  title={t('stations.actionLabel.edit')}
                  onClick={() => onEdit(station)}
                >
                  <Pencil className="size-4" aria-hidden />
                </Button>
                {!station.isDefault && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={t('stations.action.setDefault', { name: station.name })}
                    title={t('stations.actionLabel.setDefault')}
                    onClick={() => onSetDefault(station)}
                  >
                    <Star className="size-4" aria-hidden />
                  </Button>
                )}
                {canTransfer(station) && (
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    aria-label={t('stations.action.transfer', { name: station.name })}
                    title={t('stations.actionLabel.transfer')}
                    onClick={() => setPendingTransfer(station)}
                  >
                    <ArrowRightLeft className="size-4" aria-hidden />
                  </Button>
                )}
              </Can>
              <Can permission={['start_point:delete']}>
                <Button
                  type="button"
                  variant="ghost-destructive"
                  size="icon"
                  aria-label={t('stations.action.delete', { name: station.name })}
                  title={t('stations.actionLabel.delete')}
                  onClick={() => setPendingDelete(station)}
                >
                  <Trash2 className="size-4" aria-hidden />
                </Button>
              </Can>
            </div>
          </ListCard>
        )}
      />

      <DeleteConfirmDialog
        open={pendingDelete !== null}
        onOpenChange={(open) => {
          if (!open) setPendingDelete(null)
        }}
        title={t('stations.deleteConfirm.title')}
        description={t('stations.deleteConfirm.description', { name: pendingDelete?.name ?? '' })}
        onConfirm={() => {
          if (pendingDelete) onDelete(pendingDelete)
          setPendingDelete(null)
        }}
      />
      <TransferOrganizationDialog
        open={pendingTransfer !== null}
        onOpenChange={(open) => {
          if (!open) setPendingTransfer(null)
        }}
        title={t('stations.transferDialog.title', { name: pendingTransfer?.name ?? '' })}
        description={t('stations.transferDialog.description')}
        organizations={pendingTransfer ? transferTargetsOf(pendingTransfer) : []}
        onConfirm={(organizationId) => {
          if (pendingTransfer) onTransfer(pendingTransfer, organizationId)
          setPendingTransfer(null)
        }}
      />
    </>
  )
}

export default StationList
