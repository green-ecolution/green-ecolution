import { Lock, Plus, X } from 'lucide-react'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Button,
  Combobox,
} from '@green-ecolution/ui'
import type { RoleResponse } from '@/api/backendApi'
import { CARD, CARD_TITLE } from './cardChrome'

interface MemberRolesCardProps {
  roles: RoleResponse[]
  assignable: RoleResponse[]
  editable: boolean
  lockedReason: string | null
  error: string | null
  onAssign: (roleId: string) => void
  onRevoke: (roleId: string) => void
}

const MemberRolesCard = ({
  roles,
  assignable,
  editable,
  lockedReason,
  error,
  onAssign,
  onRevoke,
}: MemberRolesCardProps) => (
  <section className={CARD}>
    <h3 className={CARD_TITLE}>Rollen</h3>
    <p className="mt-0.5 text-sm text-dark-600">
      Rollen wirken sofort. Eine Änderung braucht kein Speichern.
    </p>

    {lockedReason && (
      <Alert variant="info" size="default" className="mt-4 flex w-full gap-4">
        <AlertIcon variant="info" icon={Lock} />
        <AlertContent>
          <AlertDescription>{lockedReason}</AlertDescription>
        </AlertContent>
      </Alert>
    )}

    {roles.length === 0 ? (
      <p className="mt-4 text-sm text-dark-600">Keine Rolle zugewiesen.</p>
    ) : (
      <ul
        role="list"
        aria-label="Zugewiesene Rollen"
        className="mt-4 flex list-none flex-col gap-2"
      >
        {roles.map((role) => (
          <li
            key={role.id}
            className="flex items-center justify-between gap-3 rounded-lg border border-dark-50 px-3 py-2"
          >
            <span className="min-w-0 flex-1">
              <span className="block truncate text-sm font-semibold text-dark">{role.name}</span>
              {role.description && (
                <span className="block truncate text-xs text-dark-600">{role.description}</span>
              )}
            </span>
            {editable && (
              <Button
                type="button"
                variant="ghost-destructive"
                size="sm"
                onClick={() => onRevoke(role.id)}
                aria-label={`Rolle ${role.name} entziehen`}
              >
                <X className="size-4" aria-hidden />
              </Button>
            )}
          </li>
        ))}
      </ul>
    )}

    {editable && (
      <div className="mt-4 flex items-center gap-2">
        <Plus className="size-4 shrink-0 text-dark-500" aria-hidden />
        <Combobox
          options={assignable.map((role) => ({ value: role.id, label: role.name }))}
          onChange={onAssign}
          placeholder="Rolle zuweisen"
          searchPlaceholder="Rolle suchen"
          emptyText="Keine weitere Rolle verfügbar"
          aria-label="Rolle zuweisen"
          aria-describedby={error ? 'member-roles-error' : undefined}
          aria-invalid={!!error}
        />
      </div>
    )}

    {error && (
      <p
        id="member-roles-error"
        role="alert"
        aria-live="assertive"
        className="mt-2 text-sm text-red"
      >
        {error}
      </p>
    )}
  </section>
)

export default MemberRolesCard
