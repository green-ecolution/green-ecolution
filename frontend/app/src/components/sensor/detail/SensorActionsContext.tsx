import { createContext, use, useState, type PropsWithChildren } from 'react'
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
  const [assignMode, setAssignMode] = useState<AssignMode | null>(null)
  const [removeOpen, setRemoveOpen] = useState(false)
  const [errorMessage, setErrorMessage] = useState<string | null>(null)
  const previousTreeId = sensor.linkedTreeId ?? null

  const activate = useActivateSensor(sensor.id)
  const reassign = useReassignSensorTree(sensor.id, previousTreeId)
  const deactivate = useDeactivateSensor(sensor.id, previousTreeId)

  const closeAssign = () => {
    setAssignMode(null)
    setErrorMessage(null)
  }

  const handleConfirm = (treeId: string) => {
    setErrorMessage(null)
    if (assignMode === 'activate') {
      activate.mutate(treeId, {
        onSuccess: () => {
          toast.success('Sensor aktiviert und Baum zugewiesen.')
          closeAssign()
        },
        onError: (err) => setErrorMessage(mapActivateError(err)),
      })
    } else if (assignMode === 'reassign') {
      reassign.mutate(treeId, {
        onSuccess: () => {
          toast.success('Baum gewechselt.')
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
            <AlertDialogTitle>Baumverknüpfung aufheben und Sensor zurücksetzen?</AlertDialogTitle>
            <AlertDialogDescription>
              Die Verknüpfung zum Baum wird aufgehoben und der Sensor auf den Zustand „Vorbereitet"
              zurückgesetzt. Anschließend kann er wie ein neuer Sensor an einem anderen Baum
              aktiviert werden.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={deactivate.isPending}>Abbrechen</AlertDialogCancel>
            <AlertDialogAction
              disabled={deactivate.isPending}
              onClick={(e) => {
                e.preventDefault()
                deactivate.mutate(undefined, {
                  onSuccess: () => {
                    toast.success('Sensor wurde zurückgesetzt.')
                    setRemoveOpen(false)
                  },
                  onError: (err) => toast.error(mapDeactivateError(err)),
                })
              }}
            >
              {deactivate.isPending ? 'Wird aufgehoben …' : 'Verknüpfung aufheben'}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </SensorActionsContext>
  )
}

export default SensorActionsProvider
