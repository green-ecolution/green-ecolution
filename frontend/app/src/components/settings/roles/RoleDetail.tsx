import { Copy, Lock, Trash2 } from 'lucide-react'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AvatarStack,
  Badge,
  Button,
  Input,
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
import RoleActionButtons from './RoleActionButtons'
import RoleAreaCard from './RoleAreaCard'
import RoleIcon from './roleIcon'
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
  /** Desktop renders the save bar inline; the mobile drawer renders it in a footer instead. */
  renderActionBar?: boolean
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
  renderActionBar = true,
}: RoleDetailProps) => {
  const isSystemRole = role?.isTemplate === true
  const isNew = draft.kind === 'new'
  const readOnly = isSystemRole || (!isNew && !canUpdate)
  const unknown = unknownPermissions(draft.permissions)
  const nameEmpty = draft.name.trim().length === 0

  return (
    <div className="flex flex-col gap-6">
      <header className="@container">
        <div className="flex flex-col gap-4 @min-[40rem]:flex-row @min-[40rem]:items-start">
          <div className="flex min-w-0 flex-1 items-start gap-4">
            <span className="flex size-11 shrink-0 items-center justify-center rounded-xl bg-green-dark text-white">
              <RoleIcon role={role} className="size-6" />
            </span>

            <div className="min-w-0 flex-1">
              {readOnly ? (
                <h2 className="font-lato text-2xl font-bold text-dark break-words">{draft.name}</h2>
              ) : (
                <>
                  <Input
                    aria-label="Name der Rolle"
                    placeholder="Name der Rolle"
                    value={draft.name}
                    onChange={(event) => onNameChange(event.target.value)}
                    aria-invalid={nameError !== null || nameEmpty}
                    className="w-full max-w-md font-lato text-2xl font-bold"
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
          </div>

          <div className="flex flex-wrap items-center gap-3 @min-[40rem]:shrink-0 @min-[40rem]:justify-end">
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

            <AvatarStack items={assignees} />

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
        </div>
      </header>

      {isSystemRole && (
        <Alert variant="warning" size="default" className="flex w-full gap-4">
          <AlertIcon variant="warning" icon={Lock} />
          <AlertContent>
            <AlertDescription>
              Systemrollen sind schreibgeschützt.{' '}
              {canCreate ? (
                <button
                  type="button"
                  onClick={onCopy}
                  className="font-semibold text-green-dark underline focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
                >
                  Kopiere
                </button>
              ) : (
                'Kopiere'
              )}{' '}
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

      {renderActionBar && !readOnly && dirty && (
        <div className="sticky bottom-0 -mt-6 flex flex-col-reverse gap-2 border-t border-dark-200 bg-[var(--role-panel-bg)] pb-3 pt-6 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
          <RoleActionButtons
            isNew={isNew}
            saving={saving}
            nameEmpty={nameEmpty}
            onSave={onSave}
            onCancel={onCancel}
          />
        </div>
      )}
    </div>
  )
}

export default RoleDetail
