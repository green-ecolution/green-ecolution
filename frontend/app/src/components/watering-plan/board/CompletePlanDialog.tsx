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
import { useTranslation } from 'react-i18next'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import type { WateringPlanInList } from '@/api/backendApi'
import { CancelWateringPlan, FinishedWateringPlan } from '../WateringPlanStatusUpdate'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { useWateringPlanStatusDetails } from '@/hooks/details/useDetailsForWateringPlanStatus'
import { formatBoardDate } from './format'

type CompleteMode = 'finished' | 'canceled'

interface CompletePlanDialogProps {
  plan: WateringPlanInList | null
  onClose: () => void
}

const CompletePlanDialog = ({ plan, onClose }: CompletePlanDialogProps) => {
  const [mode, setMode] = useState<CompleteMode>('finished')
  const { finishPlan, cancelPlan } = useWateringPlanBoardMutations()
  const { t } = useTranslation('wateringPlan')
  const dateLocale = useDateLocale()
  const getStatusDetails = useWateringPlanStatusDetails()

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
          <DialogTitle>{t('board.completeDialog.title')}</DialogTitle>
          <DialogDescription>
            {plan && (
              <>
                {t('board.completeDialog.description', {
                  date: formatBoardDate(plan.date, dateLocale),
                })}
              </>
            )}
          </DialogDescription>
        </DialogHeader>
        <div
          className="flex gap-2"
          role="radiogroup"
          aria-label={t('board.completeDialog.outcomeAriaLabel')}
        >
          <Button
            type="button"
            size="sm"
            variant={mode === 'finished' ? 'default' : 'outline'}
            role="radio"
            aria-checked={mode === 'finished'}
            onClick={() => setMode('finished')}
          >
            {getStatusDetails(WateringPlanStatus.Finished).label}
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === 'canceled' ? 'default' : 'outline'}
            role="radio"
            aria-checked={mode === 'canceled'}
            onClick={() => setMode('canceled')}
          >
            {getStatusDetails(WateringPlanStatus.Canceled).label}
          </Button>
        </div>
        {plan && mode === 'finished' && (
          <FinishedWateringPlan
            wateringPlanId={plan.id.toString()}
            loadedData={{ treeclusters: plan.treeclusters }}
            submitLabel={t('board.completeDialog.title')}
            onSubmit={(data) =>
              finishPlan.mutate({ plan, evaluation: data.evaluation }, { onSuccess: onClose })
            }
          />
        )}
        {plan && mode === 'canceled' && (
          <CancelWateringPlan
            className="w-full"
            submitLabel={t('board.cancelDialog.title')}
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
