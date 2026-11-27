import { Ban } from 'lucide-react'
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogIcon,
  DialogTitle,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import type { PlanInListResponse } from '@green-ecolution/backend-client'
import type { ModulePlanning } from '@/modules/types'
import { CancelPlan } from '../PlanStatusUpdate'
import { usePlanBoardMutations } from '@/hooks/usePlanBoardMutations'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatBoardDate } from './format'

interface CancelPlanDialogProps {
  plan: PlanInListResponse | null
  planning: ModulePlanning
  onClose: () => void
}

const CancelPlanDialog = ({ plan, planning, onClose }: CancelPlanDialogProps) => {
  const { cancelPlan } = usePlanBoardMutations(planning)
  const { t } = useTranslation('wateringPlan')
  const dateLocale = useDateLocale()

  return (
    <Dialog open={plan !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogIcon variant="destructive">
          <Ban />
        </DialogIcon>
        <DialogHeader>
          <DialogTitle>{t('board.cancelDialog.title')}</DialogTitle>
          <DialogDescription>
            {plan && (
              <>
                {t('board.cancelDialog.description', {
                  date: formatBoardDate(plan.date, dateLocale),
                })}
              </>
            )}
          </DialogDescription>
        </DialogHeader>
        {plan && (
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

export default CancelPlanDialog
