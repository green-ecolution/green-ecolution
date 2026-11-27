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
import { PlanStatus } from '@green-ecolution/backend-client'
import type { PlanInListResponse } from '@green-ecolution/backend-client'
import type { ModulePlanning } from '@/modules/types'
import { CancelPlan, FinishPlan } from '../PlanStatusUpdate'
import { usePlanBoardMutations } from '@/hooks/usePlanBoardMutations'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { usePlanStatusDetails } from '@/hooks/details/useDetailsForPlanStatus'
import { formatBoardDate } from './format'

type CompleteMode = 'finished' | 'canceled'

interface CompletePlanDialogProps {
  plan: PlanInListResponse | null
  planning: ModulePlanning
  onClose: () => void
}

const CompletePlanDialog = ({ plan, planning, onClose }: CompletePlanDialogProps) => {
  const [mode, setMode] = useState<CompleteMode>('finished')
  const { finishPlan, cancelPlan } = usePlanBoardMutations(planning)
  const { t } = useTranslation('plan')
  const dateLocale = useDateLocale()
  const getStatusDetails = usePlanStatusDetails()

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
            {getStatusDetails(PlanStatus.Finished).label}
          </Button>
          <Button
            type="button"
            size="sm"
            variant={mode === 'canceled' ? 'default' : 'outline'}
            role="radio"
            aria-checked={mode === 'canceled'}
            onClick={() => setMode('canceled')}
          >
            {getStatusDetails(PlanStatus.Canceled).label}
          </Button>
        </div>
        {plan && mode === 'finished' && (
          <FinishPlan
            plan={plan}
            planning={planning}
            submitLabel={t('board.completeDialog.title')}
            onSubmit={(outcomes) => finishPlan.mutate({ plan, outcomes }, { onSuccess: onClose })}
          />
        )}
        {plan && mode === 'canceled' && (
          <CancelPlan
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
