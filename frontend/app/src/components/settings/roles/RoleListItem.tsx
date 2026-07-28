import { Lock, Pencil, Shield } from 'lucide-react'
import { Avatar, AvatarFallback, ListCard } from '@green-ecolution/ui'
import type { Role } from '@/api/backendApi'

interface RoleListItemProps {
  role: Role
  selected: boolean
  locked: boolean
  /** Initials of the people holding this role; empty when user:read is missing. */
  assignees: string[]
  onSelect: () => void
}

const RoleListItem = ({ role, selected, locked, assignees, onSelect }: RoleListItemProps) => (
  <ListCard
    size="compact"
    hoverable={!selected}
    asChild
    className={selected ? 'border-green-dark bg-green-dark-50' : undefined}
  >
    <button type="button" onClick={onSelect} aria-current={selected} className="w-full text-left">
      <span
        className={`flex size-9 shrink-0 items-center justify-center rounded-xl ${
          selected ? 'bg-green-dark text-white' : 'bg-green-light-100 text-green-dark'
        }`}
      >
        <Shield className="size-5" aria-hidden />
      </span>

      <span className="min-w-0 flex-1">
        <span className="block truncate font-lato text-sm font-semibold text-dark">
          {role.name}
        </span>
        <span className="block truncate text-xs text-dark-600">
          {role.description ? `${role.description} · ` : ''}
          {role.permissions.length}
        </span>
      </span>

      {assignees.length > 0 && (
        <span className="flex shrink-0 -space-x-2">
          {assignees.slice(0, 3).map((initials) => (
            <Avatar key={initials} size="sm" className="ring-2 ring-white">
              <AvatarFallback variant="user">{initials}</AvatarFallback>
            </Avatar>
          ))}
          {assignees.length > 3 && (
            <span className="flex size-8 items-center justify-center rounded-full bg-dark-100 text-xs font-semibold text-dark-600 ring-2 ring-white">
              +{assignees.length - 3}
            </span>
          )}
        </span>
      )}

      {locked ? (
        <Lock className="size-4 shrink-0 text-dark-400" aria-label="Schreibgeschützt" />
      ) : (
        <Pencil className="size-4 shrink-0 text-dark-400" aria-hidden />
      )}
    </button>
  </ListCard>
)

export default RoleListItem
