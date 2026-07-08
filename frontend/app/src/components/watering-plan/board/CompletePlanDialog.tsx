import { useState } from 'react'
import { CheckCircle2 } from 'lucide-react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogIcon,
  DialogTitle,
} from '@green-ecolution/ui'
import type { WateringPlanInList } from '@/api/backendApi'
import { CancelWateringPlan, FinishedWateringPlan } from '../WateringPlanStatusUpdate'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'
import { formatBoardDate } from './format'

type CompleteMode = 'finished' | 'canceled'

interface CompletePlanDialogProps {
  plan: WateringPlanInList | null
  onClose: () => void
}

const CompletePlanDialog = ({ plan, onClose }: CompletePlanDialogProps) => {
  const [mode, setMode] = useState<CompleteMode>('finished')
  const { finishPlan, cancelPlan } = useWateringPlanBoardMutations()

  const handleOpenChange = (open: boolean) => {
    if (!open) {
      setMode('finished')
      onClose()
    }
  }

  return (
    <Dialog open={plan !== null} onOpenChange={handleOpenChange}>
      <DialogContent className="max-h-[85vh] overflow-y-auto">
        <DialogIcon variant="success">
          <CheckCircle2 />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>Einsatz abschließen</DialogTitle>
          <DialogDescription>
            {plan && <>Wie ist der Einsatz vom {formatBoardDate(plan.date)} ausgegangen?</>}
          </DialogDescription>
        </DialogHeader>
        <div className="flex gap-2" role="radiogroup" aria-label="Ausgang des Einsatzes">
          <Button
            type="button"
            size="sm"
            variant={mode === 'finished' ? 'default' : 'outline'}
            role="radio"
            aria-checked={mode === 'finished'}
            onClick={() => setMode('finished')}
          >
            Beendet
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === 'canceled' ? 'default' : 'outline'}
            role="radio"
            aria-checked={mode === 'canceled'}
            onClick={() => setMode('canceled')}
          >
            Abgebrochen
          </Button>
        </div>
        {plan && mode === 'finished' && (
          <FinishedWateringPlan
            wateringPlanId={plan.id.toString()}
            loadedData={{ treeclusters: plan.treeclusters }}
            submitLabel="Einsatz abschließen"
            onSubmit={(data) =>
              finishPlan.mutate({ plan, evaluation: data.evaluation }, { onSuccess: onClose })
            }
          />
        )}
        {plan && mode === 'canceled' && (
          <CancelWateringPlan
            className="w-full"
            submitLabel="Einsatz abbrechen"
            onSubmit={(data) =>
              cancelPlan.mutate({ plan, note: data.cancellationNote }, { onSuccess: onClose })
            }
          />
        )}
      </DialogContent>
    </Dialog>
  )
}

export default CompletePlanDialog
