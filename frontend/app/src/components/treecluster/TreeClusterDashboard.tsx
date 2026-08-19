import { useState } from 'react'
import { Link, useNavigate } from '@tanstack/react-router'
import { ChevronDown, Pencil, Trash2 } from 'lucide-react'
import ClusterKpiRow from './ClusterKpiRow'
import ClusterWaterSupplyChart from './ClusterWaterSupplyChart'
import ClusterWateringHistory from './ClusterWateringHistory'
import ClusterLocationCard from './ClusterLocationCard'
import ClusterSensorCard from './ClusterSensorCard'
import ClusterTreeList from './ClusterTreeList'
import ClusterMasterDataCard from './ClusterMasterDataCard'
import EntityDetailHeader from '@/components/general/EntityDetailHeader'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
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
  const navigate = useNavigate()
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const [confirmDelete, setConfirmDelete] = useState(false)
  const canEdit = useHasPermission(['tree_cluster:update'])
  const canDelete = useHasPermission(['tree_cluster:delete'])
  const wateringStatus = getWateringStatusDetails(treecluster.wateringStatus)
  const trees = treecluster.trees ?? []
  const hasSensors = trees.some((tree) => tree.sensor)

  const handleDelete = () => {
    clusterApi
      .deleteCluster({ clusterId: treecluster.id.toString() })
      // Invalidate only after leaving: this page holds a live query on the
      // cluster, which would refetch into a 404 while still mounted. Trees too,
      // because deleting a cluster walks its trees out of it.
      .then(() => navigate({ to: '/treecluster', search: { page: 1 } }))
      .then(() => invalidate(['cluster', 'tree']))
      .then(() => showToast('Die Bewässerungsgruppe wurde gelöscht.'))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast('Die Bewässerungsgruppe konnte nicht gelöscht werden.', 'error')
      })
  }

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/treecluster' }, label: 'Zu allen Bewässerungsgruppen' }}
        title={<>Bewässerungsgruppe: {treecluster.name}</>}
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
                    Gruppe bearbeiten
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
                      aria-label="Weitere Aktionen"
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
                      Gruppe löschen
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </Can>
            </ButtonGroup>
          )
        }
      >
        <p className="mb-4 text-dark-600">
          {treecluster.address} · {treecluster.region?.name ?? '—'} · {trees.length}{' '}
          {trees.length === 1 ? 'Baum' : 'Bäume'}
        </p>
        {treecluster.description && <p className="mb-4">{treecluster.description}</p>}
        {trees.length === 0 && (
          <Alert variant="destructive" className="flex gap-4">
            <AlertIcon variant="destructive" />
            <AlertContent>
              <AlertTitle>Keine Bäume zugewiesen</AlertTitle>
              <AlertDescription>
                Diese Baumgruppe enthält keine Bäume und hat daher keinen Standort.
              </AlertDescription>
            </AlertContent>
          </Alert>
        )}
      </EntityDetailHeader>

      {/* min-w-0: grid items must shrink below the chart svg's explicit width,
          otherwise Recharts locks the page wider than small viewports. */}
      <div className="mt-10 grid gap-6 xl:grid-cols-[2fr_1fr]">
        <div className="flex min-w-0 flex-col gap-6">
          <ClusterKpiRow treecluster={treecluster} />
          <ClusterWaterSupplyChart clusterId={treecluster.id} hasSensors={hasSensors} />
          <ClusterWateringHistory clusterId={treecluster.id} />
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
        title="Bewässerungsgruppe löschen?"
        description="Möchtest du die Bewässerungsgruppe wirklich löschen? Die zugehörigen Bäume bleiben erhalten."
        onConfirm={handleDelete}
      />
    </>
  )
}

export default TreeClusterDashboard
