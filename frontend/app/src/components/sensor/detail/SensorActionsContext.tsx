import { createContext, Suspense, use, useState, type PropsWithChildren } from 'react'
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  toast,
} from '@green-ecolution/ui'
import { Link2Off } from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import { mapActivateError, mapDeactivateError, mapReassignError } from '@/api/sensorErrors'
import {
  useActivateSensor,
  useDeactivateSensor,
  useReassignSensor,
} from '@/hooks/useSensorTargetMutations'
import { sensorTargetKinds, targetDescriptor, type SensorTargetKind } from '@/lib/targets'
import type { ModuleTarget, SelectedTarget, TargetAssignMode } from '@/modules/types'
import { useActiveModules } from '@/modules/useActiveModules'
import TargetKindChoice from '../TargetKindChoice'

interface SensorActionsApi {
  /** False when no active module offers a target to link the sensor to. */
  canAssign: boolean
  /** The descriptor of the sensor's current target, if its module is active. */
  currentTarget: ModuleTarget | undefined
  requestActivate: () => void
  requestReassign: () => void
  requestRemove: () => void
}

type AssignState =
  | { step: 'kind'; mode: TargetAssignMode }
  | { step: 'pick'; mode: TargetAssignMode; kind: ModuleTarget }

const SensorActionsContext = createContext<SensorActionsApi | null>(null)

/* eslint-disable-next-line react-refresh/only-export-components */
export const useSensorActions = (): SensorActionsApi => {
  const ctx = use(SensorActionsContext)
  if (!ctx) throw new Error('useSensorActions must be used within SensorActionsProvider')
  return ctx
}

const SensorActionsProvider = ({ sensor, children }: PropsWithChildren<{ sensor: Sensor }>) => {
  const { t } = useTranslation(['sensor', 'enums', 'common'])
  const { manifests } = useActiveModules()
  const kinds = sensorTargetKinds(manifests)
  const currentTarget = targetDescriptor(manifests, sensor.target)
  const [assign, setAssign] = useState<AssignState | null>(null)
  const [removeOpen, setRemoveOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const activate = useActivateSensor(sensor.id)
  const reassign = useReassignSensor(sensor.id)
  const deactivate = useDeactivateSensor(sensor.id)

  const closeAssign = () => {
    setAssign(null)
    setErrorMessage(null)
  }

  // Without a current target, or one whose module is off, the user picks the
  // kind first, exactly as in the wizard.
  const open = (mode: TargetAssignMode, kind: ModuleTarget | undefined) => {
    setErrorMessage(null)
    if (kind) setAssign({ step: 'pick', mode, kind })
    else if (kinds.length === 1) setAssign({ step: 'pick', mode, kind: kinds[0].target })
    else if (kinds.length > 1) setAssign({ step: 'kind', mode })
  }

  const handleConfirm = (mode: TargetAssignMode, kind: ModuleTarget, selected: SelectedTarget) => {
    setErrorMessage(null)
    // The old target loses the sensor, the new one gains it.
    const invalidate = [...(currentTarget?.invalidate ?? []), ...kind.invalidate]
    const variables = { target: selected.target, invalidate }
    if (mode === 'activate') {
      activate.mutate(variables, {
        onSuccess: () => {
          toast.success(t('actions.activateSuccessToast'))
          closeAssign()
        },
        onError: (err) => setErrorMessage(mapActivateError(err)),
      })
    } else {
      reassign.mutate(variables, {
        onSuccess: () => {
          toast.success(t('actions.reassignSuccessToast'))
          closeAssign()
        },
        onError: (err) => setErrorMessage(mapReassignError(err)),
      })
    }
  }

  const api: SensorActionsApi = {
    canAssign: kinds.length > 0,
    currentTarget,
    requestActivate: () => open('activate', undefined),
    requestReassign: () => open('reassign', currentTarget),
    requestRemove: () => setRemoveOpen(true),
  }

  const isPending = activate.isPending || reassign.isPending
  const picking = assign?.step === 'pick' ? assign : null
  const AssignDialog = picking?.kind.AssignDialog

  return (
    <SensorActionsContext value={api}>
      {children}

      {assign?.step === 'kind' && (
        <Dialog
          open
          onOpenChange={(next) => {
            if (!next) closeAssign()
          }}
        >
          <DialogContent>
            <DialogHeader>
              <DialogTitle>{t('wizard.target.chooseKind.title')}</DialogTitle>
              <DialogDescription>{t('wizard.target.chooseKind.description')}</DialogDescription>
            </DialogHeader>
            <TargetKindChoice
              kinds={kinds}
              onChoose={(chosen: SensorTargetKind) =>
                setAssign({ step: 'pick', mode: assign.mode, kind: chosen.target })
              }
            />
          </DialogContent>
        </Dialog>
      )}

      {picking && AssignDialog && (
        <Suspense fallback={null}>
          <AssignDialog
            open
            onOpenChange={(next) => {
              if (!next) closeAssign()
            }}
            mode={picking.mode}
            sensor={sensor}
            pending={isPending}
            error={errorMessage}
            onConfirm={(selected) => handleConfirm(picking.mode, picking.kind, selected)}
          />
        </Suspense>
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
                deactivate.mutate(
                  { invalidate: currentTarget?.invalidate ?? [] },
                  {
                    onSuccess: () => {
                      toast.success(t('actions.removeSuccessToast'))
                      setRemoveOpen(false)
                    },
                    onError: (err) => toast.error(mapDeactivateError(err)),
                  },
                )
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
