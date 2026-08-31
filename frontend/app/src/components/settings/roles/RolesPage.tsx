import { useEffect, useMemo, useState, type CSSProperties } from 'react'
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
  DrawerFooter,
  Loading,
} from '@green-ecolution/ui'
import type { Role } from '@/api/backendApi'
import { userQueries, roleQueries } from '@/api/queries'
import { useRoleMutations } from '@/hooks/useRoleMutations'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { usePermissions } from '@/lib/auth/usePermissions'
import { useContainerWiderThan } from '@/hooks/useContainerWiderThan'
import { TWO_PANE_MIN_WIDTH } from '../twoPaneWidth'
import { statusOf } from '@/lib/httpError'
import { initialsOf } from '@/lib/initials'
import RoleActionButtons from './RoleActionButtons'
import RoleDetail from './RoleDetail'
import RoleList from './RoleList'
import { ownRolesOf } from './roleList'
import { useRoleDraft } from './useRoleDraft'

const TEAM_USERS_PARAMS = { page: 1, perPage: 100 }

const nameConflictMessage = (error: unknown): string | null =>
  statusOf(error) === 409 ? 'Eine Rolle mit diesem Namen existiert bereits.' : null

const RolesPage = () => {
  const grantable = usePermissions()
  const canCreate = useHasPermission(['role:create'])
  const canUpdate = useHasPermission(['role:update'])
  const canDelete = useHasPermission(['role:delete'])
  const canReadUsers = useHasPermission(['user:read'])
  const { ref: layoutRef, isWide } = useContainerWiderThan<HTMLDivElement>(TWO_PANE_MIN_WIDTH)

  const { data: me } = useQuery(userQueries.me())
  const orgId = me?.organization?.id ?? null

  const { data: templates, isLoading: templatesLoading } = useQuery(roleQueries.templates())
  const { data: orgRoles, isLoading: orgRolesLoading } = useQuery({
    ...roleQueries.org(orgId ?? ''),
    enabled: orgId !== null,
  })
  const { data: users } = useQuery({
    ...userQueries.list(TEAM_USERS_PARAMS),
    enabled: canReadUsers,
  })

  const { createRole, updateRole, deleteRole } = useRoleMutations()
  const draftState = useRoleDraft(grantable)
  const { draft, dirty, editExisting, startNew, startCopy, discard } = draftState

  const [selected, setSelected] = useState<Role | null>(null)
  const [pendingSelection, setPendingSelection] = useState<Role | null | undefined>(undefined)
  const [pendingClose, setPendingClose] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)

  const blocker = useBlocker({
    shouldBlockFn: () => dirty,
    // Also gates the browser's own unload prompt; see useFormNavigationBlocker.
    enableBeforeUnload: () => dirty,
    withResolver: true,
  })

  const templateList = useMemo(() => templates ?? [], [templates])
  const ownRoles = useMemo(() => ownRolesOf(orgRoles ?? []), [orgRoles])

  useEffect(() => {
    // Only in the two-pane layout, where the detail pane is always visible. In
    // the Drawer layout, auto-selecting would pop it open on page load.
    if (!isWide || selected !== null || draft !== null) return
    const first = ownRoles[0] ?? templateList[0] ?? null
    if (first) {
      // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- picks a default once query data loads, so the detail pane isn't empty
      setSelected(first)
      editExisting(first)
    }
  }, [isWide, selected, draft, ownRoles, templateList, editExisting])

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

  const requestClose = (open: boolean) => {
    if (open) return
    if (dirty) {
      setPendingClose(true)
      return
    }
    discard()
  }

  const confirmDiscard = () => {
    if (pendingClose) {
      setPendingClose(false)
      discard()
      return
    }
    applyPendingSelection()
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

  const renderDetail = (renderActionBar: boolean) =>
    draft && (
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
        renderActionBar={renderActionBar}
      />
    )

  return (
    <>
      {/* Measured, not the branch content, so the ResizeObserver keeps observing
          across the two-pane <-> Drawer switch instead of losing its element. */}
      <div ref={layoutRef}>
        {isWide ? (
          <div className="grid grid-cols-[280px_minmax(0,1fr)] gap-6">
            {list}
            {/* --role-panel-bg lets the sticky action bar blend into its surface. */}
            <div
              className="min-w-0"
              style={{ '--role-panel-bg': 'var(--color-dark-50)' } as CSSProperties}
            >
              {renderDetail(true)}
            </div>
          </div>
        ) : (
          <>
            {list}
            <Drawer open={draft !== null} onOpenChange={requestClose}>
              <DrawerContent className="max-h-[90vh]">
                {/* Content scrolls in its own region; the actions live in a fixed
                    footer below so they stay anchored to the drawer bottom. */}
                <div className="min-h-0 flex-1 overflow-y-auto p-4">{renderDetail(false)}</div>
                {draft && dirty && (
                  <DrawerFooter className="flex-col-reverse gap-2 border-t border-dark-200 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
                    <RoleActionButtons
                      isNew={draft.kind === 'new'}
                      saving={createRole.isPending || updateRole.isPending}
                      nameEmpty={draft.name.trim() === ''}
                      onSave={save}
                      onCancel={cancel}
                    />
                  </DrawerFooter>
                )}
              </DrawerContent>
            </Drawer>
          </>
        )}
      </div>

      <AlertDialog
        open={pendingSelection !== undefined || pendingClose}
        onOpenChange={(open) => {
          if (open) return
          setPendingSelection(undefined)
          setPendingClose(false)
        }}
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
            <AlertDialogAction onClick={confirmDiscard}>Verwerfen</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Rolle löschen?</AlertDialogTitle>
            <AlertDialogDescription>
              {draft?.name} wird entfernt. Personen mit dieser Rolle verlieren die darin enthaltenen
              Rechte.
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
