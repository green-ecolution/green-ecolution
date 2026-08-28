import { Badge } from '@green-ecolution/ui'
import EntityDetailHeader from '../general/EntityDetailHeader'
import TreeKpiRow from './TreeKpiRow'
import TreeLocationCard from './TreeLocationCard'
import TreeClusterCard from './TreeClusterCard'
import TreeSensorCard from './TreeSensorCard'
import TreeMasterDataCard from './TreeMasterDataCard'
import { getWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import type { Tree, TreeCluster } from '@/api/backendApi'
import { useHasPermission } from '@/lib/auth/useHasPermission'

interface TreeDashboardProps {
  tree: Tree
  treeCluster?: TreeCluster
}

const TreeDashboard = ({ tree, treeCluster }: TreeDashboardProps) => {
  const canEdit = useHasPermission(['tree:update'])
  const wateringStatus = getWateringStatusDetails(tree.wateringStatus)

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/trees' }, label: 'Zu allen Bäumen' }}
        title={<>Baum: {tree.number}</>}
        badge={<Badge variant={wateringStatus.color}>{wateringStatus.label}</Badge>}
        editLink={
          canEdit
            ? {
                label: 'Baum bearbeiten',
                link: {
                  to: `/map/tree/edit/$treeId`,
                  params: { treeId: String(tree.id) },
                },
              }
            : undefined
        }
      >
        <p className="mb-4 text-dark-600">
          {tree.species} ·{' '}
          {treeCluster
            ? `${treeCluster.name} · ${treeCluster.address}`
            : 'Keiner Bewässerungsgruppe zugeordnet'}
        </p>
        {tree.description && <p>{tree.description}</p>}
      </EntityDetailHeader>

      <div className="mt-10 flex flex-col gap-6">
        <TreeKpiRow tree={tree} />
        <div className="grid gap-6 md:grid-cols-2">
          <div className="flex flex-col gap-6">
            <TreeLocationCard tree={tree} />
            <TreeClusterCard tree={tree} treeCluster={treeCluster} />
          </div>
          <div className="flex flex-col gap-6">
            <TreeSensorCard tree={tree} />
            <TreeMasterDataCard tree={tree} />
          </div>
        </div>
      </div>
    </>
  )
}

export default TreeDashboard
