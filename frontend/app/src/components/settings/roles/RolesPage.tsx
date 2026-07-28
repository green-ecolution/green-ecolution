import { useEffect, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useBlocker } from '@tanstack/react-router'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Drawer,
  DrawerContent,
  Loading,
} from '@green-ecolution/ui'
import type { Role } from '@/api/backendApi'
import { currentUserQuery, orgRolesQuery, roleTemplatesQuery, userQuery } from '@/api/queries'
import { useRoleMutations } from '@/hooks/useRoleMutations'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { usePermissions } from '@/lib/auth/usePermissions'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import RoleDetail from './RoleDetail'
import RoleList from './RoleList'
import { ownRolesOf } from './roleList'
import { useRoleDraft } from './useRoleDraft'

const TEAM_USERS_PARAMS = { page: 1, perPage: 100 }

const initialsOf = (firstName?: string | null, lastName?: string | null): string =>
  `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase()

const nameConflictMessage = (error: unknown): string | null => {
  const status = (error as { response?: { status?: number } } | null)?.response?.status
  return status === 409 ? 'Eine Rolle mit diesem Namen existiert bereits.' : null
}

const RolesPage = () => {
  const grantable = usePermissions()
  const canCreate = useHasPermission(['role:create'])
  const canUpdate = useHasPermission(['role:update'])
  const canDelete = useHasPermission(['role:delete'])
  const canReadUsers = useHasPermission(['user:read'])
  const isDesktop = useMediaQuery('(min-width: 1024px)')

  const { data: me } = useQuery(currentUserQuery())
  const orgId = me?.organization?.id ?? null

  const { data: templates, isLoading: templatesLoading } = useQuery(roleTemplatesQuery())
  const { data: orgRoles, isLoading: orgRolesLoading } = useQuery({
    ...orgRolesQuery(orgId ?? ''),
    enabled: orgId !== null,
  })
  const { data: users } = useQuery({ ...userQuery(TEAM_USERS_PARAMS), enabled: canReadUsers })

  const { createRole, updateRole, deleteRole } = useRoleMutations()
  const draftState = useRoleDraft(grantable)
  const { draft, dirty, editExisting, startNew, startCopy, discard } = draftState

  const [selected, setSelected] = useState<Role | null>(null)
  const [pendingSelection, setPendingSelection] = useState<Role | null | undefined>(undefined)
  const [confirmDelete, setConfirmDelete] = useState(false)

  const blocker = useBlocker({ shouldBlockFn: () => dirty, withResolver: true })

  const templateList = useMemo(() => templates ?? [], [templates])
  const ownRoles = useMemo(
    () => ownRolesOf(orgRoles ?? [], templateList),
    [orgRoles, templateList],
  )

  useEffect(() => {
    if (selected !== null || draft !== null) return
    const first = ownRoles[0] ?? templateList[0] ?? null
    if (first) {
      // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- picks a default once query data loads, so the detail pane isn't empty
      setSelected(first)
      editExisting(first)
    }
  }, [selected, draft, ownRoles, templateList, editExisting])

  const assigneesOf = (roleId: string): string[] =>
    (users?.data ?? [])
      .filter((user) => user.roles.some((role) => role.id === roleId))
      .map((user) => initialsOf(user.firstName, user.lastName))
      .filter((initials) => initials.length > 0)

  const select = (role: Role) => {
    if (dirty) {
      setPendingSelection(role)
      return
    }
    setSelected(role)
    editExisting(role)
    createRole.reset()
    updateRole.reset()
  }

  const applyPendingSelection = () => {
    const next = pendingSelection
    setPendingSelection(undefined)
    if (next === undefined) return
    if (next === null) {
      startNew()
      setSelected(null)
      return
    }
    setSelected(next)
    editExisting(next)
  }

  const requestNew = () => {
    if (dirty) {
      setPendingSelection(null)
      return
    }
    setSelected(null)
    startNew()
  }

  const save = () => {
    if (!draft) return
    const description = draft.description.trim() === '' ? null : draft.description.trim()
    const permissions = [...draft.permissions]

    if (draft.kind === 'new') {
      if (!orgId) return
      createRole.mutate(
        { orgId, name: draft.name.trim(), description, permissions },
        {
          onSuccess: (created) => {
            setSelected(created)
            editExisting(created)
          },
        },
      )
      return
    }

    if (!draft.id) return
    updateRole.mutate(
      { roleId: draft.id, name: draft.name.trim(), description, permissions },
      {
        onSuccess: (updated) => {
          setSelected(updated)
          editExisting(updated)
        },
      },
    )
  }

  const cancel = () => {
    if (selected) editExisting(selected)
    else discard()
  }

  const remove = () => {
    if (!draft?.id) return
    deleteRole.mutate(
      { roleId: draft.id },
      {
        onSuccess: () => {
          setConfirmDelete(false)
          setSelected(null)
          discard()
        },
      },
    )
  }

  if (!orgId) {
    return (
      <p className="text-sm text-dark-600">
        Deinem Konto ist keine Organisation zugeordnet. Ohne Organisation lassen sich keine Rollen
        verwalten.
      </p>
    )
  }

  if (templatesLoading || orgRolesLoading) {
    return <Loading className="mt-10 justify-center" label="Rollen werden geladen" />
  }

  const list = (
    <RoleList
      templates={templateList}
      ownRoles={ownRoles}
      selectedId={draft?.kind === 'new' ? null : (selected?.id ?? null)}
      canCreate={canCreate}
      assigneesOf={assigneesOf}
      onSelect={select}
      onCreate={requestNew}
    />
  )

  const detail = draft && (
    <RoleDetail
      role={draft.kind === 'new' ? null : selected}
      draft={draft}
      dirty={dirty}
      grantable={grantable}
      canUpdate={canUpdate}
      canDelete={canDelete}
      canCreate={canCreate}
      assignees={draft.kind === 'new' || !selected ? [] : assigneesOf(selected.id)}
      nameError={nameConflictMessage(createRole.error ?? updateRole.error)}
      saving={createRole.isPending || updateRole.isPending}
      onNameChange={draftState.setName}
      onDescriptionChange={draftState.setDescription}
      onLevelChange={draftState.setLevel}
      onActionToggle={draftState.toggle}
      onCopy={() => selected && startCopy(selected)}
      onSave={save}
      onCancel={cancel}
      onDelete={() => setConfirmDelete(true)}
    />
  )

  return (
    <>
      {isDesktop ? (
        <div className="grid grid-cols-[280px_1fr] gap-6">
          {list}
          <div>{detail}</div>
        </div>
      ) : (
        <>
          {list}
          <Drawer open={draft !== null} onOpenChange={(open) => !open && discard()}>
            <DrawerContent className="max-h-[90vh] overflow-y-auto p-4">{detail}</DrawerContent>
          </Drawer>
        </>
      )}

      <AlertDialog
        open={pendingSelection !== undefined}
        onOpenChange={(open) => !open && setPendingSelection(undefined)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Änderungen verwerfen?</AlertDialogTitle>
            <AlertDialogDescription>
              Du hast Änderungen an dieser Rolle, die noch nicht gespeichert sind.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Weiter bearbeiten</AlertDialogCancel>
            <AlertDialogAction onClick={applyPendingSelection}>Verwerfen</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Rolle löschen?</AlertDialogTitle>
            <AlertDialogDescription>
              {draft?.name} wird entfernt. Personen mit dieser Rolle verlieren die darin
              enthaltenen Rechte.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Abbrechen</AlertDialogCancel>
            <AlertDialogAction onClick={remove}>Löschen</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {blocker.status === 'blocked' && (
        <AlertDialog open onOpenChange={() => blocker.reset?.()}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
              <AlertDialogDescription>
                Deine Änderungen an dieser Rolle sind noch nicht gespeichert.
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel onClick={() => blocker.reset?.()}>
                Weiter bearbeiten
              </AlertDialogCancel>
              <AlertDialogAction onClick={() => blocker.proceed?.()}>Verlassen</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      )}
    </>
  )
}

export default RolesPage
