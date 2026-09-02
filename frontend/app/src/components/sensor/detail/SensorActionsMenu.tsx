import { useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { ChevronDown, Link2, Link2Off, Replace, Trash2 } from 'lucide-react'
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
import { useInvalidateAggregates } from '@/lib/queryInvalidation'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useSensorActions } from './SensorActionsContext'

interface SensorActionsMenuProps {
  sensor: Sensor
}

const SensorActionsMenu = ({ sensor }: SensorActionsMenuProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const sensorId = sensor.id
  const actions = useSensorActions()
  const isPrepared = sensor.status === 'prepared'
  const [dialogOpen, setDialogOpen] = useState(false)
  const navigate = useNavigate()
  const invalidate = useInvalidateAggregates()
  const canUpdate = useHasPermission(['sensor:update'])
  const canDelete = useHasPermission(['sensor:delete'])

  const deleteMutation = useMutation({
    mutationFn: () => sensorApi.deleteSensor({ sensorId }),
    onSuccess: async () => {
      toast.success(t('actions.deleteSuccessToast'))
      await navigate({ to: '/sensors', search: { page: 1 } })
      // After leaving: this page holds a live query on the sensor. Trees too,
      // because deleting a sensor detaches it from the tree it was on.
      await invalidate(['sensor', 'tree'])
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
          {canUpdate &&
            (isPrepared ? (
              <DropdownMenuItem
                className="cursor-pointer"
                onSelect={() => actions.requestActivate()}
              >
                <Link2 className="mr-2 size-4" />
                {t('actions.activateAssignTree')}
              </DropdownMenuItem>
            ) : (
              <>
                <DropdownMenuItem
                  className="cursor-pointer"
                  onSelect={() => actions.requestReassign()}
                >
                  <Replace className="mr-2 size-4" />
                  {t('actions.reassignTree')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  className="cursor-pointer"
                  onSelect={() => actions.requestRemove()}
                >
                  <Link2Off className="mr-2 size-4" />
                  {t('actions.removeLink')}
                </DropdownMenuItem>
              </>
            ))}
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
