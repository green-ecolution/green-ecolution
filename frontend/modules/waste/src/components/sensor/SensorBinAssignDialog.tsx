import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  InlineAlert,
} from '@green-ecolution/ui'
import type { SelectedTarget, TargetAssignDialogProps } from '@/modules/types'
import { wasteBinQueries } from '../../api/queries'
import BinPickList from './BinPickList'
import { binSelection } from './useBinSelection'

const SensorBinAssignDialog = ({
  open,
  onOpenChange,
  mode,
  sensor,
  pending,
  error,
  onConfirm,
}: TargetAssignDialogProps) => {
  const { t } = useTranslation(['waste', 'common'])
  const [selected, setSelected] = useState<SelectedTarget | null>(null)
  const { data, isLoading, isError, refetch } = useQuery({
    ...wasteBinQueries.markers(),
    enabled: open,
  })

  const copy =
    mode === 'activate'
      ? {
          title: t('sensor.binAssignDialog.activate.title'),
          description: t('sensor.binAssignDialog.activate.description'),
          confirm: t('sensor.binAssignDialog.activate.confirm'),
        }
      : {
          title: t('sensor.binTexts.reassignAction'),
          description: t('sensor.binAssignDialog.reassign.description'),
          confirm: t('sensor.binAssignDialog.reassign.confirm'),
        }

  return (
    <Dialog
      open={open}
      onOpenChange={(next) => {
        if (!next) setSelected(null)
        onOpenChange(next)
      }}
    >
      <DialogContent className="flex h-[88vh] max-w-md flex-col gap-0 p-0 sm:h-[70vh] sm:max-w-xl">
        <DialogHeader className="px-6 pb-3 pt-6">
          <DialogTitle>{copy.title}</DialogTitle>
          <DialogDescription>{copy.description}</DialogDescription>
        </DialogHeader>

        <div className="flex min-h-0 flex-1 flex-col px-6 pb-2">
          <BinPickList
            bins={data ?? []}
            loading={isLoading}
            failed={isError}
            onRetry={() => void refetch()}
            selectedId={selected?.target.id ?? null}
            sensorId={sensor.id}
            onSelect={(bin) => setSelected(binSelection(bin))}
          />
        </div>

        <DialogFooter className="flex-col gap-3 px-6 py-4 sm:flex-row sm:items-center sm:justify-end">
          {error && <InlineAlert variant="destructive" description={error} />}
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={pending}>
            {t('common:actions.cancel')}
          </Button>
          <Button disabled={!selected || pending} onClick={() => selected && onConfirm(selected)}>
            {pending ? t('sensor.binAssignDialog.saving') : copy.confirm}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

export default SensorBinAssignDialog
