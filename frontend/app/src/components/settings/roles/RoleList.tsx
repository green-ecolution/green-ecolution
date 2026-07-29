import { Plus } from 'lucide-react'
import { Button } from '@green-ecolution/ui'
import type { Role } from '@/api/backendApi'
import RoleListItem from './RoleListItem'

interface RoleListProps {
  templates: Role[]
  ownRoles: Role[]
  selectedId: string | null
  canCreate: boolean
  assigneesOf: (roleId: string) => string[]
  onSelect: (role: Role) => void
  onCreate: () => void
}

const SECTION_LABEL = 'text-xs font-semibold uppercase tracking-wider text-dark-500'

const RoleList = ({
  templates,
  ownRoles,
  selectedId,
  canCreate,
  assigneesOf,
  onSelect,
  onCreate,
}: RoleListProps) => (
  <div className="flex flex-col gap-6">
    <div className="flex items-center justify-between gap-4">
      <h2 className="font-lato text-base font-semibold">Rollen</h2>
      {canCreate && (
        <Button type="button" size="sm" onClick={onCreate}>
          <Plus className="size-4" aria-hidden />
          Neu
        </Button>
      )}
    </div>

    <section className="flex flex-col gap-2">
      <p className={SECTION_LABEL}>System · nicht editierbar</p>
      {templates.map((role) => (
        <RoleListItem
          key={role.id}
          role={role}
          selected={role.id === selectedId}
          locked
          assignees={[]}
          onSelect={() => onSelect(role)}
        />
      ))}
    </section>

    <section className="flex flex-col gap-2">
      <p className={SECTION_LABEL}>Eigene Rollen</p>
      {ownRoles.length === 0 ? (
        <p className="text-sm text-dark-600">
          Noch keine eigene Rolle. Kopiere eine Systemrolle oder lege eine neue an.
        </p>
      ) : (
        ownRoles.map((role) => (
          <RoleListItem
            key={role.id}
            role={role}
            selected={role.id === selectedId}
            locked={false}
            assignees={assigneesOf(role.id)}
            onSelect={() => onSelect(role)}
          />
        ))
      )}
    </section>
  </div>
)

export default RoleList
