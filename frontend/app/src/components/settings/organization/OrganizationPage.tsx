import { useEffect, useState, type CSSProperties } from 'react'
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
import type { AddressDto, OrganizationDetailResponse, OrganizationResponse } from '@/api/backendApi'
import { organizationQueries, userQueries } from '@/api/queries'
import { useOrganizationMutations } from '@/hooks/useOrganizationMutations'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import ContactPersonPicker from './ContactPersonPicker'
import CreateOrganizationDialog from './CreateOrganizationDialog'
import OrganizationActionButtons from './OrganizationActionButtons'
import OrganizationDetail from './OrganizationDetail'
import OrganizationTree from './OrganizationTree'
import { buildTree, pathTo, type OrgNode } from './organizationTree'
import { useOrganizationDraft } from './useOrganizationDraft'

const MEMBER_USERS_PARAMS = { page: 1, perPage: 100 }

const memberInitialsOf = (firstName?: string | null, lastName?: string | null): string =>
  `${firstName?.charAt(0) ?? ''}${lastName?.charAt(0) ?? ''}`.toUpperCase()

const statusOf = (error: unknown): number | undefined =>
  (error as { response?: { status?: number } } | null)?.response?.status

const nameConflictMessage = (error: unknown): string | null =>
  statusOf(error) === 409 ? 'Eine Organisation mit diesem Namen existiert bereits.' : null

const contactPersonMessage = (error: unknown): string | null =>
  statusOf(error) === 422
    ? 'Diese Person ist dieser Organisation nicht zugeordnet und kann nicht Kontaktperson sein.'
    : null

const nodeOf = (node: OrgNode, orgId: string): OrgNode | null => {
  if (node.org.id === orgId) return node
  for (const child of node.children) {
    const found = nodeOf(child, orgId)
    if (found) return found
  }
  return null
}

const OrganizationPage = () => {
  const canCreate = useHasPermission(['organization:create'])
  const canUpdate = useHasPermission(['organization:update'])
  const canDelete = useHasPermission(['organization:delete'])
  const canReadUsers = useHasPermission(['user:read'])
  const isDesktop = useMediaQuery('(min-width: 1024px)')

  const { data: me } = useQuery(userQueries.me())
  const rootId = me?.organization?.id ?? null

  const { data: orgs, isLoading: orgsLoading } = useQuery(organizationQueries.list())
  const { data: users } = useQuery({
    ...userQueries.list(MEMBER_USERS_PARAMS),
    enabled: canReadUsers,
  })

  const { createOrganization, updateOrganization, deleteOrganization } = useOrganizationMutations()
  const draftState = useOrganizationDraft()
  const { draft, dirty, addressErrors, addressComplete, edit } = draftState

  const [selection, setSelection] = useState<string | null>(null)
  const [expandedOverride, setExpandedOverride] = useState<ReadonlySet<string> | null>(null)
  const [loadedDetail, setLoadedDetail] = useState<OrganizationDetailResponse | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)
  const [pendingSelection, setPendingSelection] = useState<string | undefined>(undefined)
  const [pendingClose, setPendingClose] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [createOpen, setCreateOpen] = useState(false)
  const [pickerOpen, setPickerOpen] = useState(false)

  const selectedId = selection ?? rootId

  const { data: detail } = useQuery({
    ...organizationQueries.byId(selectedId ?? ''),
    enabled: selectedId !== null,
  })

  const blocker = useBlocker({ shouldBlockFn: () => dirty, withResolver: true })

  useEffect(() => {
    // Keyed on the object, not the id: after a save the invalidated query hands
    // back a new object and the draft must pick up the server's truth. Structural
    // sharing keeps the reference stable when nothing changed, so an in-progress
    // edit survives a refetch that returns the same data.
    if (!detail || detail === loadedDetail) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- loads the freshly fetched organization into the draft
    setLoadedDetail(detail)
    edit(detail)
  }, [detail, loadedDetail, edit])

  const members = (users?.data ?? []).filter((user) => user.organization?.id === selectedId)
  const memberInitials = members
    .map((user) => memberInitialsOf(user.firstName, user.lastName))
    .filter((initials) => initials.length > 0)

  if (rootId === null) {
    return (
      <p className="text-sm text-dark-600">
        Deinem Konto ist keine Organisation zugeordnet. Ohne Organisation lässt sich keine
        Organisationsstruktur verwalten.
      </p>
    )
  }

  if (orgsLoading) {
    return <Loading className="mt-10 justify-center" label="Organisationen werden geladen" />
  }

  const root = buildTree(orgs ?? [], rootId)

  if (!root) {
    return (
      <p className="text-sm text-dark-600">
        Deine Organisation wurde nicht gefunden. Ohne sie lässt sich die Organisationsstruktur nicht
        anzeigen.
      </p>
    )
  }

  const selectedNode = selectedId === null ? null : nodeOf(root, selectedId)
  const readOnly = detail?.parentId == null

  // Without an override the path down to the selection is open, so the selected
  // node is never hidden inside a collapsed branch.
  const expanded =
    expandedOverride ?? new Set(pathTo(root, selectedId ?? root.org.id).map((org) => org.id))

  const toggle = (orgId: string) => {
    const next = new Set(expanded)
    if (!next.delete(orgId)) next.add(orgId)
    setExpandedOverride(next)
  }

  const reveal = (orgId: string) => {
    const ancestors = pathTo(root, orgId)
      .slice(0, -1)
      .map((org) => org.id)
    if (ancestors.length === 0) return
    setExpandedOverride(new Set([...expanded, ...ancestors]))
  }

  const goTo = (orgId: string) => {
    setSelection(orgId)
    reveal(orgId)
    setDetailOpen(true)
    createOrganization.reset()
    updateOrganization.reset()
  }

  const select = (org: OrganizationResponse) => {
    if (org.id === selectedId) {
      setDetailOpen(true)
      return
    }
    if (dirty) {
      setPendingSelection(org.id)
      return
    }
    goTo(org.id)
  }

  const resetDraft = () => {
    if (detail) edit(detail)
  }

  const applyPendingSelection = () => {
    const next = pendingSelection
    setPendingSelection(undefined)
    if (next === undefined) return
    goTo(next)
  }

  const requestClose = (open: boolean) => {
    if (open) return
    if (dirty) {
      setPendingClose(true)
      return
    }
    setDetailOpen(false)
  }

  const confirmDiscard = () => {
    if (pendingClose) {
      setPendingClose(false)
      setDetailOpen(false)
      resetDraft()
      return
    }
    applyPendingSelection()
  }

  const save = () => {
    if (!draft || !detail) return
    const street = draft.street.trim()
    const postalCode = draft.postalCode.trim()
    const city = draft.city.trim()
    const filled = [street, postalCode, city].filter((value) => value.length > 0)
    // An address is stored whole or not at all; a partial one is rejected here too
    // because Enter in a field can bypass the disabled save button.
    if (filled.length !== 0 && filled.length !== 3) return
    const address: AddressDto | null = filled.length === 3 ? { street, postalCode, city } : null

    updateOrganization.mutate({
      orgId: detail.id,
      name: draft.name.trim(),
      address,
      contactPersonId: draft.contactPersonId,
    })
  }

  const create = (name: string) => {
    if (!detail) return
    createOrganization.mutate(
      { parentId: detail.id, name },
      {
        onSuccess: (created) => {
          setCreateOpen(false)
          setExpandedOverride(new Set([...expanded, detail.id]))
          setSelection(created.id)
          setDetailOpen(true)
        },
      },
    )
  }

  const remove = () => {
    if (!detail) return
    deleteOrganization.mutate(
      { orgId: detail.id },
      {
        onSuccess: () => {
          setConfirmDelete(false)
          setDetailOpen(false)
          setSelection(detail.parentId ?? rootId)
        },
      },
    )
  }

  const tree = (
    <OrganizationTree
      root={root}
      selectedId={selectedId}
      expanded={expanded}
      canCreate={canCreate}
      onSelect={select}
      onToggle={toggle}
      onCreate={() => {
        createOrganization.reset()
        setCreateOpen(true)
      }}
    />
  )

  const renderDetail = (renderActionBar: boolean) =>
    detail &&
    draft &&
    selectedNode && (
      <OrganizationDetail
        node={selectedNode}
        root={root}
        detail={detail}
        draft={draft}
        dirty={dirty}
        addressErrors={addressErrors}
        addressComplete={addressComplete}
        readOnly={readOnly}
        canUpdate={canUpdate}
        canCreate={canCreate}
        canDelete={canDelete}
        canReadUsers={canReadUsers}
        memberInitials={memberInitials}
        saving={updateOrganization.isPending}
        nameError={nameConflictMessage(updateOrganization.error)}
        contactPersonError={contactPersonMessage(updateOrganization.error)}
        onNameChange={draftState.setName}
        onStreetChange={draftState.setStreet}
        onPostalCodeChange={draftState.setPostalCode}
        onCityChange={draftState.setCity}
        onContactPersonRequest={() => setPickerOpen(true)}
        onSubOrganizationCreate={() => {
          createOrganization.reset()
          setCreateOpen(true)
        }}
        onSelectChild={select}
        onSave={save}
        onCancel={resetDraft}
        onDelete={() => setConfirmDelete(true)}
        renderActionBar={renderActionBar}
      />
    )

  return (
    <>
      {isDesktop ? (
        <div className="grid grid-cols-[300px_1fr] gap-6">
          {tree}
          {/* --org-panel-bg lets the sticky action bar blend into its surface. */}
          <div style={{ '--org-panel-bg': 'var(--color-dark-50)' } as CSSProperties}>
            {renderDetail(true)}
          </div>
        </div>
      ) : (
        <>
          {tree}
          <Drawer open={detailOpen} onOpenChange={requestClose}>
            <DrawerContent className="max-h-[90vh]">
              {/* Content scrolls in its own region; the actions live in a fixed
                  footer below so they stay anchored to the drawer bottom. */}
              <div className="min-h-0 flex-1 overflow-y-auto p-4">{renderDetail(false)}</div>
              {draft && dirty && !readOnly && (
                <DrawerFooter className="flex-col-reverse gap-2 border-t border-dark-200 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
                  <OrganizationActionButtons
                    saving={updateOrganization.isPending}
                    nameEmpty={draft.name.trim().length === 0}
                    addressComplete={addressComplete}
                    onSave={save}
                    onCancel={resetDraft}
                  />
                </DrawerFooter>
              )}
            </DrawerContent>
          </Drawer>
        </>
      )}

      <CreateOrganizationDialog
        open={createOpen}
        parentName={detail?.name ?? ''}
        saving={createOrganization.isPending}
        nameError={nameConflictMessage(createOrganization.error)}
        onOpenChange={setCreateOpen}
        onSubmit={create}
      />

      <ContactPersonPicker
        open={pickerOpen}
        members={members}
        selectedId={draft?.contactPersonId ?? null}
        onOpenChange={setPickerOpen}
        onSelect={(userId) => {
          draftState.setContactPersonId(userId)
          setPickerOpen(false)
        }}
      />

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
              Du hast Änderungen an dieser Organisation, die noch nicht gespeichert sind.
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
            <AlertDialogTitle>Organisation löschen?</AlertDialogTitle>
            <AlertDialogDescription>
              {detail?.name} wird entfernt. Das ist nur möglich, solange keine Unterorganisationen
              und keine Mitarbeitenden zugeordnet sind.
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
                Deine Änderungen an dieser Organisation sind noch nicht gespeichert.
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

export default OrganizationPage
