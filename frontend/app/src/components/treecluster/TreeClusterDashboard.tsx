import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronDown, Pencil, Trash2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import ClusterKpiRow from './ClusterKpiRow'
import ClusterWaterSupplyChart from './ClusterWaterSupplyChart'
import ClusterWateringHistory from './ClusterWateringHistory'
import ClusterLocationCard from './ClusterLocationCard'
import ClusterSensorCard from './ClusterSensorCard'
import ClusterTreeList from './ClusterTreeList'
import ClusterMasterDataCard from './ClusterMasterDataCard'
import CommentsSection from '@/components/comments/CommentsSection'
import EntityDetailHeader from '@/components/general/EntityDetailHeader'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import { unknownStatusReasons } from './clusterStatusReason'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import createToast from '@/hooks/createToast'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import {
  Alert,
  AlertIcon,
  AlertContent,
  AlertTitle,
  AlertDescription,
  Badge,
  Button,
  ButtonGroup,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'
import { clusterApi, type TreeCluster } from '@/api/backendApi'
import { useInvalidateAggregates } from '@/lib/queryInvalidation'

interface TreeClusterDashboardProps {
  treecluster: TreeCluster
}

const TreeClusterDashboard = ({ treecluster }: TreeClusterDashboardProps) => {
  const { t } = useTranslation('treecluster')
  const navigate = useNavigate()
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const [confirmDelete, setConfirmDelete] = useState(false)
  const canEdit = useHasPermission(['tree_cluster:update'])
  const canDelete = useHasPermission(['tree_cluster:delete'])
  const getWateringStatusDetails = useWateringStatusDetails()
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const trees = treecluster.trees ?? []
  const hasSensors = trees.some((tree) => tree.sensor)
  // Only the calibration window depends on it, so a stale year cannot mislead.
  // eslint-disable-next-line react-x/purity
  const statusReasons = unknownStatusReasons(treecluster, new Date().getFullYear(), t)

  const handleDelete = () => {
    clusterApi
      .deleteCluster({ clusterId: treecluster.id.toString() })
      // Invalidate only after leaving: this page holds a live query on the
      // cluster, which would refetch into a 404 while still mounted. Trees too,
      // because deleting a cluster walks its trees out of it.
      .then(() => navigate({ to: '/treecluster', search: { page: 1 } }))
      .then(() => invalidate(['cluster', 'tree']))
      .then(() => showToast(t('dashboard.deleteSuccessToast')))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast(t('dashboard.deleteErrorToast'), 'error')
      })
  }

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/treecluster' }, label: t('dashboard.backToList') }}
        title={<>{t('dashboard.title', { name: treecluster.name })}</>}
        badge={<Badge variant={wateringStatus.color}>{wateringStatus.label}</Badge>}
        actions={
          (canEdit || canDelete) && (
            <ButtonGroup>
              {canEdit && (
                <Button variant="outline" asChild>
                  <Link
                    to="/map/treecluster/edit/$treeclusterId"
                    params={{ treeclusterId: treecluster.id.toString() }}
                  >
                    {t('dashboard.editLink')}
                    <Pencil className="stroke-1" />
                  </Link>
                </Button>
              )}
              <Can permission={['tree_cluster:delete']}>
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      aria-label={t('dashboard.moreActionsAriaLabel')}
                      className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-base data-[state=open]:[&_svg]:rotate-180 motion-reduce:[&_svg]:transition-none"
                    >
                      <ChevronDown />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="min-w-[12rem]">
                    <DropdownMenuItem
                      className="gap-2 px-3 py-2 text-destructive focus:bg-destructive/10 focus:text-destructive"
                      onSelect={() => setConfirmDelete(true)}
                    >
                      <Trash2 />
                      {t('dashboard.deleteMenuItem')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </Can>
            </ButtonGroup>
          )
        }
      >
        <p className="mb-4 text-dark-600">
          {t('dashboard.infoLine', {
            address: treecluster.address,
            region: treecluster.region?.name ?? '—',
            treeCount: t('dashboard.treeCount', { count: trees.length }),
          })}
        </p>
        {treecluster.description && <p className="mb-4">{treecluster.description}</p>}
        {trees.length === 0 && (
          <Alert variant="destructive" className="flex gap-4">
            <AlertIcon variant="destructive" />
            <AlertContent>
              <AlertTitle>{t('dashboard.noTreesTitle')}</AlertTitle>
              <AlertDescription>{t('dashboard.noTreesDescription')}</AlertDescription>
            </AlertContent>
          </Alert>
        )}
      </EntityDetailHeader>

      {statusReasons.length > 0 && (
        <Alert variant="info" className="mt-6 flex w-full gap-4">
          <AlertIcon variant="info" />
          <AlertContent>
            <AlertTitle>{t('unknownStatus.title')}</AlertTitle>
            {statusReasons.length === 1 ? (
              <AlertDescription>{statusReasons[0].text}</AlertDescription>
            ) : (
              <>
                <AlertDescription>{t('unknownStatus.multipleReasonsNote')}</AlertDescription>
                <ul className="list-disc space-y-1 pl-5 text-sm leading-relaxed text-muted-foreground">
                  {statusReasons.map((reason) => (
                    <li key={reason.key}>{reason.text}</li>
                  ))}
                </ul>
              </>
            )}
          </AlertContent>
        </Alert>
      )}

      {/* min-w-0: grid items must shrink below the chart svg's explicit width,
          otherwise Recharts locks the page wider than small viewports. */}
      <div className="mt-10 grid gap-6 xl:grid-cols-[2fr_1fr]">
        <div className="flex min-w-0 flex-col gap-6">
          <ClusterKpiRow treecluster={treecluster} />
          <ClusterWaterSupplyChart clusterId={treecluster.id} hasSensors={hasSensors} />
          <ClusterWateringHistory clusterId={treecluster.id} />
          <CommentsSection subject="cluster" parentId={treecluster.id.toString()} />
        </div>
        <div className="flex min-w-0 flex-col gap-6">
          <ClusterLocationCard treecluster={treecluster} />
          <ClusterSensorCard trees={trees} />
          <ClusterTreeList trees={trees} />
          <ClusterMasterDataCard treecluster={treecluster} />
        </div>
      </div>

      <DeleteConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title={t('dashboard.deleteDialogTitle')}
        description={t('dashboard.deleteDialogDescription')}
        onConfirm={handleDelete}
      />
    </>
  )
}

export default TreeClusterDashboard
