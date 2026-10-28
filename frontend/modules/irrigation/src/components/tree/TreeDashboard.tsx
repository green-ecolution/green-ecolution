import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Link } from '@tanstack/react-router'
import { useMutation } from '@tanstack/react-query'
import { ArrowRightLeft, ChevronDown, Pencil } from 'lucide-react'
import {
  Badge,
  Button,
  ButtonGroup,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@green-ecolution/ui'
import EntityDetailHeader from '@/components/general/EntityDetailHeader'
import TransferOrganizationDialog from '@/components/general/transfer/TransferOrganizationDialog'
import { useTransferTargets } from '@/components/general/transfer/useTransferTargets'
import TreeKpiRow from './TreeKpiRow'
import TreeLocationCard from './TreeLocationCard'
import TreeClusterCard from './TreeClusterCard'
import TreeSensorCard from './TreeSensorCard'
import TreeMasterDataCard from './TreeMasterDataCard'
import { useWateringStatusDetails } from '../../hooks/details/useDetailsForWateringStatus'
import { treeApi, type Tree, type TreeCluster } from '../../api/client'
import { irrigationRoots } from '../../api/queries'
import { queryRoots } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'

interface TreeDashboardProps {
  tree: Tree
  treeCluster?: TreeCluster
}

const TreeDashboard = ({ tree, treeCluster }: TreeDashboardProps) => {
  const { t } = useTranslation(['tree', 'common'])
  const invalidate = useInvalidateQueryRoots()
  const showToast = createToast()
  const [transferOpen, setTransferOpen] = useState(false)
  const canEdit = useHasPermission(['tree:update'])
  const transferTargets = useTransferTargets('tree:update', tree.organizationId)
  const canTransfer = canEdit && transferTargets.length > 0
  // A tree inside a group moves with its group, the backend refuses it alone.
  const transferBlockedReason = tree.treeClusterId
    ? treeCluster
      ? t('detail.transferBlockedInCluster', { name: treeCluster.name })
      : t('detail.transferBlockedInUnknownCluster')
    : undefined
  const getWateringStatusDetails = useWateringStatusDetails()
  const wateringStatus = getWateringStatusDetails(tree.wateringStatus)

  const transfer = useMutation({
    mutationFn: (organizationId: string) =>
      treeApi.transferTree({ treeId: tree.id, transferRequest: { organizationId } }),
    onSuccess: async () => {
      setTransferOpen(false)
      showToast(t('detail.transferSuccessToast'))
      // An attached sensor moves along with the tree.
      await invalidate([...irrigationRoots.tree, ...queryRoots.sensor], { reloadRoutes: true })
    },
    onError: () => showToast(t('detail.transferErrorToast'), 'error'),
  })

  return (
    <>
      <EntityDetailHeader
        backLink={{ link: { to: '/trees' }, label: t('detail.backToList') }}
        title={<>{t('detail.title', { number: tree.number })}</>}
        badge={<Badge variant={wateringStatus.color}>{wateringStatus.label}</Badge>}
        actions={
          canEdit && (
            <ButtonGroup>
              <Button variant="outline" asChild>
                <Link to="/map/tree/edit/$treeId" params={{ treeId: String(tree.id) }}>
                  {t('detail.editLink')}
                  <Pencil className="stroke-1" />
                </Link>
              </Button>
              {canTransfer && (
                <DropdownMenu>
                  <DropdownMenuTrigger asChild>
                    <Button
                      variant="outline"
                      size="icon"
                      aria-label={t('detail.moreActionsAriaLabel')}
                      className="[&_svg]:size-4 [&_svg]:transition-transform [&_svg]:duration-base data-[state=open]:[&_svg]:rotate-180 motion-reduce:[&_svg]:transition-none"
                    >
                      <ChevronDown />
                    </Button>
                  </DropdownMenuTrigger>
                  <DropdownMenuContent align="end" className="min-w-[12rem]">
                    <DropdownMenuItem
                      className="items-start gap-2 px-3 py-2"
                      disabled={transferBlockedReason !== undefined}
                      onSelect={() => setTransferOpen(true)}
                    >
                      <ArrowRightLeft className="mt-0.5" />
                      <span className="flex flex-col gap-0.5">
                        {t('common:transfer.menuItem')}
                        {transferBlockedReason && (
                          <span className="text-xs text-muted-foreground">
                            {transferBlockedReason}
                          </span>
                        )}
                      </span>
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              )}
            </ButtonGroup>
          )
        }
      >
        <p className="mb-4 text-dark-600">
          {tree.species} ·{' '}
          {treeCluster
            ? `${treeCluster.name} · ${treeCluster.address}`
            : t('detail.noClusterAssigned')}
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

      <TransferOrganizationDialog
        open={transferOpen}
        onOpenChange={setTransferOpen}
        title={t('detail.transferDialogTitle', { number: tree.number })}
        description={t('detail.transferDialogDescription')}
        organizations={transferTargets}
        pending={transfer.isPending}
        onConfirm={(organizationId) => transfer.mutate(organizationId)}
      />
    </>
  )
}

export default TreeDashboard
