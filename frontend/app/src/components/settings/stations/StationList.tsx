import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Pencil, Star, Trash2 } from 'lucide-react'
import { Badge, Button, ListCard, ListCardDescription, ListCardTitle } from '@green-ecolution/ui'
import type { Organization, StartPointResponse } from '@/api/backendApi'
import { Can } from '@/lib/auth/Can'
import { capacityDisplay } from '@/lib/capacity'
import EntityList from '@/components/general/EntityList'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'

interface StationListProps {
  stations: StartPointResponse[]
  organizations: Organization[]
  onEdit: (station: StartPointResponse) => void
  onSetDefault: (station: StartPointResponse) => void
  onDelete: (station: StartPointResponse) => void
}

const StationList = ({
  stations,
  organizations,
  onEdit,
  onSetDefault,
  onDelete,
}: StationListProps) => {
  const { t } = useTranslation('settings')
  const [pendingDelete, setPendingDelete] = useState<StartPointResponse | null>(null)

  const organizationName = (id: string) => organizations.find((org) => org.id === id)?.name ?? id

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
    </>
  )
}

export default StationList
