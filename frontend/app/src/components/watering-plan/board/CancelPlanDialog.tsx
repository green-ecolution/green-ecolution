import { Ban } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogIcon,
  DialogTitle,
} from '@green-ecolution/ui'
import type { WateringPlanInList } from '@/api/backendApi'
import { CancelWateringPlan } from '../WateringPlanStatusUpdate'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'
import { formatBoardDate } from './format'

interface CancelPlanDialogProps {
  plan: WateringPlanInList | null
  onClose: () => void
}

const CancelPlanDialog = ({ plan, onClose }: CancelPlanDialogProps) => {
  const { cancelPlan } = useWateringPlanBoardMutations()

  return (
    <Dialog open={plan !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogIcon variant="destructive">
          <Ban />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>Einsatz abbrechen</DialogTitle>
          <DialogDescription>
            {plan && (
              <>
                Der Einsatz vom {formatBoardDate(plan.date)} wird abgebrochen. Geben Sie dafür einen
                Grund an.
              </>
            )}
          </DialogDescription>
        </DialogHeader>
        {plan && (
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

export default CancelPlanDialog
