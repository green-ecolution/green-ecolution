import { Copy, Lock, Shield, Trash2 } from 'lucide-react'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Avatar,
  AvatarFallback,
  Badge,
  Button,
  Input,
  Spinner,
} from '@green-ecolution/ui'
import type { Role } from '@/api/backendApi'
import type { Permission, Permissions, Resource } from '@/lib/auth/permissions'
import {
  AREA_GROUP_LABELS,
  AREA_GROUP_ORDER,
  PERMISSION_AREAS,
  unknownPermissions,
  type AccessLevel,
} from '@/lib/auth/permissionAreas'
import RoleAreaCard from './RoleAreaCard'
import type { RoleDraft } from './useRoleDraft'

interface RoleDetailProps {
  /** null while a brand new draft is being written. */
  role: Role | null
  draft: RoleDraft
  dirty: boolean
  grantable: Permissions
  canUpdate: boolean
  canDelete: boolean
  canCreate: boolean
  assignees: string[]
  nameError: string | null
  saving: boolean
  onNameChange: (name: string) => void
  onDescriptionChange: (description: string) => void
  onLevelChange: (resource: Resource, level: AccessLevel) => void
  onActionToggle: (permission: Permission) => void
  onCopy: () => void
  onSave: () => void
  onCancel: () => void
  onDelete: () => void
}

const SECTION_LABEL = 'text-xs font-semibold uppercase tracking-wider text-dark-500'

const RoleDetail = ({
  role,
  draft,
  dirty,
  grantable,
  canUpdate,
  canDelete,
  canCreate,
  assignees,
  nameError,
  saving,
  onNameChange,
  onDescriptionChange,
  onLevelChange,
  onActionToggle,
  onCopy,
  onSave,
  onCancel,
  onDelete,
}: RoleDetailProps) => {
  const isSystemRole = role?.isTemplate === true
  const isNew = draft.kind === 'new'
  const readOnly = isSystemRole || (!isNew && !canUpdate)
  const unknown = unknownPermissions(draft.permissions)
  const nameEmpty = draft.name.trim().length === 0

  return (
    <div className="flex flex-col gap-6">
      <header className="flex flex-wrap items-start gap-4">
        <span className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-green-dark text-white">
          <Shield className="size-6" aria-hidden />
        </span>

        <div className="min-w-0 flex-1">
          {readOnly ? (
            <h2 className="font-lato text-2xl font-bold text-dark">{draft.name}</h2>
          ) : (
            <>
              <Input
                aria-label="Name der Rolle"
                value={draft.name}
                onChange={(event) => onNameChange(event.target.value)}
                aria-invalid={nameError !== null || nameEmpty}
                className="max-w-md font-lato text-2xl font-bold"
              />
              {nameError && <p className="mt-1 text-sm text-red">{nameError}</p>}
              {!nameError && nameEmpty && (
                <p className="mt-1 text-sm text-dark-600">Gib der Rolle einen Namen.</p>
              )}
              <Input
                aria-label="Beschreibung der Rolle"
                value={draft.description}
                onChange={(event) => onDescriptionChange(event.target.value)}
                placeholder="Kurze Beschreibung"
                className="mt-2 max-w-md text-sm"
              />
            </>
          )}

          {readOnly && draft.description && (
            <p className="mt-1 max-w-prose text-sm text-dark-600">{draft.description}</p>
          )}
        </div>

        <div className="flex shrink-0 items-center gap-3">
          <Badge variant={isSystemRole ? 'muted' : 'outline-green-dark'}>
            {isSystemRole ? (
              <>
                <Lock className="mr-1 size-3" aria-hidden />
                Systemrolle
              </>
            ) : (
              'Eigene Rolle'
            )}
          </Badge>

          {assignees.length > 0 && (
            <span className="flex -space-x-2">
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

          {!isNew && canCreate && (
            <Button type="button" variant="outline" size="sm" onClick={onCopy}>
              <Copy className="size-4" aria-hidden />
              Kopieren & bearbeiten
            </Button>
          )}

          {!isNew && !isSystemRole && canDelete && (
            <Button type="button" variant="outline" size="sm" onClick={onDelete}>
              <Trash2 className="size-4" aria-hidden />
              Löschen
            </Button>
          )}
        </div>
      </header>

      {isSystemRole && (
        <Alert variant="warning" size="default" className="flex w-full gap-4">
          <AlertIcon variant="warning" icon={Lock} />
          <AlertContent>
            <AlertDescription>
              Systemrollen sind schreibgeschützt.{' '}
              <button
                type="button"
                onClick={onCopy}
                className="font-semibold text-green-dark underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
              >
                Kopiere
              </button>{' '}
              die Rolle, um Berechtigungen anzupassen oder eine eigene Rolle zu erstellen.
            </AlertDescription>
          </AlertContent>
        </Alert>
      )}

      {draft.clampedAway.length > 0 && (
        <Alert variant="warning" size="default" className="flex w-full gap-4">
          <AlertIcon variant="warning" />
          <AlertContent>
            <AlertDescription>
              {draft.clampedAway.length} Rechte wurden entfernt, weil du sie selbst nicht besitzt.
            </AlertDescription>
          </AlertContent>
        </Alert>
      )}

      {AREA_GROUP_ORDER.map((group) => (
        <section key={group} className="flex flex-col gap-3">
          <p className={SECTION_LABEL}>{AREA_GROUP_LABELS[group]}</p>
          {PERMISSION_AREAS.filter((area) => area.group === group).map((area) => (
            <RoleAreaCard
              key={area.resource}
              area={area}
              permissions={draft.permissions}
              grantable={grantable}
              readOnly={readOnly}
              onLevelChange={(level) => onLevelChange(area.resource, level)}
              onActionToggle={onActionToggle}
            />
          ))}
        </section>
      ))}

      {unknown.length > 0 && (
        <section className="rounded-xl border border-dark-100 bg-dark-50 p-4">
          <p className="text-xs font-semibold uppercase tracking-wider text-dark-500">
            Weitere Rechte ({unknown.length})
          </p>
          <p className="mt-1 text-sm text-dark-600">
            Diese Rechte kennt diese Ansicht nicht. Sie bleiben beim Speichern erhalten.
          </p>
          <ul className="mt-2 flex flex-wrap gap-2">
            {unknown.map((permission) => (
              <li key={permission}>
                <Badge variant="muted" className="font-mono font-normal">
                  {permission}
                </Badge>
              </li>
            ))}
          </ul>
        </section>
      )}

      {!readOnly && dirty && (
        <div className="sticky bottom-0 -mx-4 flex items-center justify-end gap-3 border-t border-dark-200 bg-white/95 px-4 py-3 backdrop-blur lg:-mx-6 lg:px-6">
          <Button type="button" variant="outline" onClick={onCancel} disabled={saving}>
            Abbrechen
          </Button>
          <Button type="button" onClick={onSave} disabled={saving || nameEmpty}>
            {saving && <Spinner className="size-4" />}
            {isNew ? 'Rolle anlegen' : 'Speichern'}
          </Button>
        </div>
      )}
    </div>
  )
}

export default RoleDetail
