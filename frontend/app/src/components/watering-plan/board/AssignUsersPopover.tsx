import { useState } from 'react'
import {
  Button,
  MultiSelectCombobox,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import type { User, WateringPlanInList } from '@/api/backendApi'
import { useWateringPlanBoardMutations } from '@/hooks/useWateringPlanBoardMutations'

interface AssignUsersPopoverProps {
  plan: WateringPlanInList
  users: User[]
}

const AssignUsersPopover = ({ plan, users }: AssignUsersPopoverProps) => {
  const [open, setOpen] = useState(false)
  const [selected, setSelected] = useState<string[]>(plan.userIds)
  const { assignUsers } = useWateringPlanBoardMutations()
  const { t } = useTranslation('wateringPlan')

  const options = users.map((user) => ({
    value: user.id,
    label: `${user.firstName} ${user.lastName}`,
  }))

  const handleOpenChange = (next: boolean) => {
    setOpen(next)
    if (next) setSelected(plan.userIds)
  }

  return (
    <Popover open={open} onOpenChange={handleOpenChange}>
      <PopoverTrigger asChild>
        <Button type="button" size="sm" variant="outline">
          {plan.userIds.length > 0
            ? t('board.assignUsersPopover.changeLabel')
            : t('board.assignUsersPopover.assignLabel')}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-72 space-y-3">
        <p className="font-lato text-sm font-semibold text-dark">
          {t('board.assignUsersPopover.heading')}
        </p>
        <MultiSelectCombobox
          options={options}
          value={selected}
          onChange={setSelected}
          placeholder={t('board.assignUsersPopover.placeholder')}
        />
        <Button
          type="button"
          size="sm"
          className="w-full"
          disabled={assignUsers.isPending}
          onClick={() =>
            assignUsers.mutate({ plan, userIds: selected }, { onSuccess: () => setOpen(false) })
          }
        >
          {t('board.assignUsersPopover.saveLabel')}
        </Button>
      </PopoverContent>
    </Popover>
  )
}

export default AssignUsersPopover
