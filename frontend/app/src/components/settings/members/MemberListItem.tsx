import { Avatar, AvatarFallback, AvatarImage, Badge, ListCard } from '@green-ecolution/ui'
import type { UserResponse } from '@/api/backendApi'
import { useUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'
import { initialsOf } from '@/lib/initials'
import { fullNameOf } from './memberList'

interface MemberListItemProps {
  user: UserResponse
  selected: boolean
  onSelect: () => void
}

const MemberListItem = ({ user, selected, onSelect }: MemberListItemProps) => {
  const getUserStatusDetails = useUserStatusDetails()
  const status = getUserStatusDetails(user.status)

  return (
    <ListCard
      size="compact"
      hoverable={!selected}
      asChild
      className={selected ? 'border-green-dark bg-green-dark-50' : undefined}
    >
      <button type="button" onClick={onSelect} aria-current={selected} className="w-full text-left">
        <Avatar size="sm">
          {user.avatarUrl && <AvatarImage src={user.avatarUrl} alt="" />}
          <AvatarFallback variant="user">
            {initialsOf(user.firstName, user.lastName)}
          </AvatarFallback>
        </Avatar>

        <span className="min-w-0 flex-1">
          <span className="block truncate font-lato text-sm font-semibold text-dark">
            {fullNameOf(user)}
          </span>
          <span className="block truncate text-xs text-dark-600">
            {user.organization?.name ?? 'Keine Organisation'}
          </span>
        </span>

        <Badge variant={status.color}>{status.label}</Badge>
      </button>
    </ListCard>
  )
}

export default MemberListItem
