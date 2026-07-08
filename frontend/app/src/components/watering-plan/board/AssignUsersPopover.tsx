import { useState } from 'react'
import {
  Button,
  MultiSelectCombobox,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from '@green-ecolution/ui'
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
          {plan.userIds.length > 0 ? 'Ändern' : 'Zuweisen'}
        </Button>
      </PopoverTrigger>
      <PopoverContent align="end" className="w-72 space-y-3">
        <p className="font-lato text-sm font-semibold text-dark">Mitarbeitende zuweisen</p>
        <MultiSelectCombobox
          options={options}
          value={selected}
          onChange={setSelected}
          placeholder="Mitarbeitende auswählen…"
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
          Zuweisung speichern
        </Button>
      </PopoverContent>
    </Popover>
  )
}

export default AssignUsersPopover
