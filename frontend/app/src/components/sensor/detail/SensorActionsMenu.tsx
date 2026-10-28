import { queryRoots } from '@/api/queries'
import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { ArrowRightLeft, ChevronDown, Link2, Link2Off, Replace, Trash2 } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogIcon,
  AlertDialogTitle,
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  toast,
} from '@green-ecolution/ui'
import type { Sensor } from '@/api/backendApi'
import { sensorApi } from '@/api/backendApi'
import TransferOrganizationDialog from '@/components/general/transfer/TransferOrganizationDialog'
import { useTransferTargets } from '@/components/general/transfer/useTransferTargets'
import { useInvalidateQueryRoots } from '@/lib/queryInvalidation'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { targetTextKey } from '@/lib/targets'
import { useSensorActions } from './SensorActionsContext'

interface SensorActionsMenuProps {
  sensor: Sensor
}

const SensorActionsMenu = ({ sensor }: SensorActionsMenuProps) => {
  const { t, i18n } = useTranslation(['sensor', 'common'])
  const sensorId = sensor.id
  const actions = useSensorActions()
  const isPrepared = sensor.status === 'prepared'
  const [dialogOpen, setDialogOpen] = useState(false)
  const [transferOpen, setTransferOpen] = useState(false)
  const navigate = useNavigate()
  const invalidate = useInvalidateQueryRoots()
  const canUpdate = useHasPermission(['sensor:update'])
  const canDelete = useHasPermission(['sensor:delete'])
  const transferTargets = useTransferTargets('sensor:update', sensor.organizationId)
  const canTransfer = canUpdate && transferTargets.length > 0
  // A bound sensor moves with its target, the backend refuses it alone.
  const transferBlocked = sensor.target != null

  const transferMutation = useMutation({
    mutationFn: (organizationId: string) =>
      sensorApi.transferSensor({ sensorId, transferRequest: { organizationId } }),
    onSuccess: async () => {
      setTransferOpen(false)
      toast.success(t('actions.transferSuccessToast'))
      await invalidate(queryRoots.sensor, { reloadRoutes: true })
    },
    onError: () => {
      toast.error(t('actions.transferFailedToast'))
    },
  })

  const deleteMutation = useMutation({
    mutationFn: () => sensorApi.deleteSensor({ sensorId }),
    onSuccess: async () => {
      toast.success(t('actions.deleteSuccessToast'))
      await navigate({ to: '/sensors', search: { page: 1 } })
      // After leaving: this page holds a live query on the sensor. The target's
      // module too, because deleting a sensor detaches it from its target.
      await invalidate([...queryRoots.sensor, ...(actions.currentTarget?.invalidate ?? [])])
    },
    onError: () => {
      toast.error(t('actions.deleteFailedToast'))
    },
  })

  if (!canUpdate && !canDelete) return null

  return (
    <AlertDialog open={dialogOpen} onOpenChange={setDialogOpen}>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="outline" size="sm" className="gap-2 [&_svg]:size-4">
            {t('actions.menuButton')}
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end" className="min-w-56">
          {canUpdate && actions.canAssign && isPrepared && (
            <DropdownMenuItem className="cursor-pointer" onSelect={() => actions.requestActivate()}>
              <Link2 className="mr-2 size-4" />
              {i18n.t(
                targetTextKey(
                  actions.textTarget,
                  'activateAction',
                  'sensor:actions.activateAssign',
                ),
              )}
            </DropdownMenuItem>
          )}
          {canUpdate && actions.canAssign && !isPrepared && (
            <DropdownMenuItem className="cursor-pointer" onSelect={() => actions.requestReassign()}>
              <Replace className="mr-2 size-4" />
              {i18n.t(
                targetTextKey(actions.textTarget, 'reassignAction', 'sensor:actions.reassign'),
              )}
            </DropdownMenuItem>
          )}
          {canUpdate && !isPrepared && (
            <DropdownMenuItem className="cursor-pointer" onSelect={() => actions.requestRemove()}>
              <Link2Off className="mr-2 size-4" />
              {i18n.t(
                targetTextKey(actions.textTarget, 'removeLinkAction', 'sensor:actions.removeLink'),
              )}
            </DropdownMenuItem>
          )}
          {canTransfer && (
            <DropdownMenuItem
              className="cursor-pointer items-start"
              disabled={transferBlocked}
              onSelect={() => setTransferOpen(true)}
            >
              <ArrowRightLeft className="mr-2 mt-0.5 size-4" />
              <span className="flex flex-col gap-0.5">
                {t('common:transfer.menuItem')}
                {transferBlocked && (
                  <span className="text-xs text-muted-foreground">
                    {t('actions.transferBlockedBound')}
                  </span>
                )}
              </span>
            </DropdownMenuItem>
          )}
          {canDelete && (
            <DropdownMenuItem
              className="text-destructive focus:text-destructive cursor-pointer"
              onSelect={(e) => {
                e.preventDefault()
                setDialogOpen(true)
              }}
            >
              <Trash2 className="mr-2 size-4" />
              {t('actions.deleteSensor')}
            </DropdownMenuItem>
          )}
        </DropdownMenuContent>
      </DropdownMenu>

      <TransferOrganizationDialog
        open={transferOpen}
        onOpenChange={setTransferOpen}
        title={t('actions.transferDialogTitle', { id: sensorId })}
        description={t('actions.transferDialogDescription')}
        organizations={transferTargets}
        pending={transferMutation.isPending}
        onConfirm={(organizationId) => transferMutation.mutate(organizationId)}
      />

      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogIcon variant="destructive">
            <Trash2 />
          </AlertDialogIcon>
          <AlertDialogTitle>{t('actions.deleteConfirmTitle')}</AlertDialogTitle>
          <AlertDialogDescription>
            {t('actions.deleteConfirmDescription', { id: sensorId })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={deleteMutation.isPending}>
            {t('common:actions.cancel')}
          </AlertDialogCancel>
          <AlertDialogAction
            disabled={deleteMutation.isPending}
            onClick={(e) => {
              e.preventDefault()
              deleteMutation.mutate()
            }}
          >
            {deleteMutation.isPending ? t('actions.deletePending') : t('actions.deleteFinal')}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

export default SensorActionsMenu
