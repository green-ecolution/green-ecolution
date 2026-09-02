import { createContext, use, useState, type PropsWithChildren } from 'react'
import { useTranslation } from 'react-i18next'
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
  toast,
} from '@green-ecolution/ui'
import { Link2Off } from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import { mapActivateError, mapDeactivateError, mapReassignError } from '@/api/sensorErrors'
import {
  useActivateSensor,
  useDeactivateSensor,
  useReassignSensorTree,
} from '@/hooks/useSensorTreeMutations'
import SensorTreeAssignDialog, { type AssignMode } from '../SensorTreeAssignDialog'

interface SensorActionsApi {
  requestActivate: () => void
  requestReassign: () => void
  requestRemove: () => void
}

const SensorActionsContext = createContext<SensorActionsApi | null>(null)

/* eslint-disable-next-line react-refresh/only-export-components */
export const useSensorActions = (): SensorActionsApi => {
  const ctx = use(SensorActionsContext)
  if (!ctx) throw new Error('useSensorActions must be used within SensorActionsProvider')
  return ctx
}

const SensorActionsProvider = ({ sensor, children }: PropsWithChildren<{ sensor: Sensor }>) => {
  const { t } = useTranslation(['sensor', 'enums', 'common'])
  const [assignMode, setAssignMode] = useState<AssignMode | null>(null)
  const [removeOpen, setRemoveOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const activate = useActivateSensor(sensor.id)
  const reassign = useReassignSensorTree(sensor.id)
  const deactivate = useDeactivateSensor(sensor.id)

  const closeAssign = () => {
    setAssignMode(null)
    setErrorMessage(null)
  }

  const handleConfirm = (treeId: string) => {
    setErrorMessage(null)
    if (assignMode === 'activate') {
      activate.mutate(treeId, {
        onSuccess: () => {
          toast.success(t('actions.activateSuccessToast'))
          closeAssign()
        },
        onError: (err) => setErrorMessage(mapActivateError(err)),
      })
    } else if (assignMode === 'reassign') {
      reassign.mutate(treeId, {
        onSuccess: () => {
          toast.success(t('actions.reassignSuccessToast'))
          closeAssign()
        },
        onError: (err) => setErrorMessage(mapReassignError(err)),
      })
    }
  }

  const api: SensorActionsApi = {
    requestActivate: () => {
      setErrorMessage(null)
      setAssignMode('activate')
    },
    requestReassign: () => {
      setErrorMessage(null)
      setAssignMode('reassign')
    },
    requestRemove: () => setRemoveOpen(true),
  }

  const isPending = activate.isPending || reassign.isPending

  return (
    <SensorActionsContext value={api}>
      {children}

      {assignMode && (
        <SensorTreeAssignDialog
          open
          onOpenChange={(next) => {
            if (!next) closeAssign()
          }}
          mode={assignMode}
          sensor={sensor}
          isPending={isPending}
          errorMessage={errorMessage}
          onConfirm={handleConfirm}
        />
      )}

      <AlertDialog open={removeOpen} onOpenChange={setRemoveOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogIcon variant="destructive">
              <Link2Off />
            </AlertDialogIcon>
            <AlertDialogTitle>{t('actions.removeConfirmTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('actions.removeConfirmDescription', {
                status: t('enums:sensorStatus.prepared.label'),
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deactivate.isPending}>
              {t('common:actions.cancel')}
            </AlertDialogCancel>
            <AlertDialogAction
              disabled={deactivate.isPending}
              onClick={(e) => {
                e.preventDefault()
                deactivate.mutate(undefined, {
                  onSuccess: () => {
                    toast.success(t('actions.removeSuccessToast'))
                    setRemoveOpen(false)
                  },
                  onError: (err) => toast.error(mapDeactivateError(err)),
                })
              }}
            >
              {deactivate.isPending
                ? t('actions.removeConfirmSubmitPending')
                : t('actions.removeConfirmSubmit')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SensorActionsContext>
  )
}

export default SensorActionsProvider
