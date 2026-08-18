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
import { useContainerWiderThan } from '@/hooks/useContainerWiderThan'
import { TWO_PANE_MIN_WIDTH } from '../twoPaneWidth'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { initialsOf } from '@/lib/initials'
import ContactPersonPicker from './ContactPersonPicker'
import CreateOrganizationDialog from './CreateOrganizationDialog'
import OrganizationActionButtons from './OrganizationActionButtons'
import OrganizationDetail from './OrganizationDetail'
import OrganizationTree from './OrganizationTree'
import { buildTree, pathTo, type OrgNode } from './organizationTree'
import { useOrganizationDraft } from './useOrganizationDraft'

const MEMBERS_PER_PAGE = 100

const statusOf = (error: unknown): number | undefined =>
  (error as { response?: { status?: number } } | null)?.response?.status

// Frontend permission gating is scope-blind, so a user whose role is owned by a
// child organization reaches this page and then gets a 403 from the backend.
const loadErrorMessage = (error: unknown): string =>
  statusOf(error) === 403
    ? 'Du darfst diese Organisation nicht einsehen. Vermutlich gilt deine Rolle nur für eine untergeordnete Organisation.'
    : 'Die Organisationsstruktur konnte nicht geladen werden. Bitte versuche es später erneut.'

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
  const { ref: layoutRef, isWide } = useContainerWiderThan<HTMLDivElement>(TWO_PANE_MIN_WIDTH)

  const { data: me } = useQuery(userQueries.me())
  const rootId = me?.organization?.id ?? null

  const {
    data: orgs,
    isLoading: orgsLoading,
    error: orgsError,
  } = useQuery(organizationQueries.list())

  const { createOrganization, updateOrganization, deleteOrganization } = useOrganizationMutations()
  const draftState = useOrganizationDraft()
  const { draft, dirty, addressErrors, addressComplete, edit } = draftState

  const [selection, setSelection] = useState<string | null>(null)
  const [expandedOverride, setExpandedOverride] = useState<ReadonlySet<string> | null>(null)
  const [loadedDetail, setLoadedDetail] = useState<OrganizationDetailResponse | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)
  const [pendingSelection, setPendingSelection] = useState<string | null | undefined>(undefined)
  const [pendingClose, setPendingClose] = useState(false)
  const [confirmDelete, setConfirmDelete] = useState(false)
  const [createOpen, setCreateOpen] = useState(false)
  const [pickerOpen, setPickerOpen] = useState(false)

  const selectedId = selection ?? rootId

  const { data: detail, error: detailError } = useQuery({
    ...organizationQueries.byId(selectedId ?? ''),
    enabled: selectedId !== null,
  })

  // Asking the backend for exactly this organization's members: filtering a
  // truncated first page client-side hides anyone beyond it.
  const { data: memberPage } = useQuery({
    ...userQueries.list({
      page: 1,
      perPage: MEMBERS_PER_PAGE,
      organizationId: selectedId ?? undefined,
    }),
    enabled: canReadUsers && selectedId !== null,
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

  const members = memberPage?.data ?? []
  const memberInitials = members
    .map((user) => initialsOf(user.firstName, user.lastName))
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

  // A failed load must not be reported as absence — the data may exist and just
  // be unreachable for this account.
  if (orgsError) {
    return (
      <p role="alert" className="text-sm text-dark-600">
        {loadErrorMessage(orgsError)}
      </p>
    )
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

  // null stands for "open the create dialog", mirroring how RolesPage encodes
  // "start a new role" in the same pending-intent state.
  const requestCreate = () => {
    if (dirty) {
      setPendingSelection(null)
      return
    }
    createOrganization.reset()
    setCreateOpen(true)
  }

  const applyPendingSelection = () => {
    const next = pendingSelection
    setPendingSelection(undefined)
    if (next === undefined) return
    if (next === null) {
      resetDraft()
      createOrganization.reset()
      setCreateOpen(true)
      return
    }
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
    const name = draft.name.trim()
    const filled = [street, postalCode, city].filter((value) => value.length > 0)
    // Repeats what the disabled action bar already prevents: neither an empty name
    // nor half an address may reach the backend, whoever calls this.
    if (name.length === 0) return
    if (filled.length !== 0 && filled.length !== 3) return
    const address: AddressDto | null = filled.length === 3 ? { street, postalCode, city } : null

    updateOrganization.mutate({
      orgId: detail.id,
      name,
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
        // A 409 explains itself in a toast, which the open dialog would cover.
        onError: () => setConfirmDelete(false),
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
      onCreate={requestCreate}
    />
  )

  const renderDetail = (renderActionBar: boolean) => {
    if (detailError) {
      return (
        <p role="alert" className="text-sm text-dark-600">
          {loadErrorMessage(detailError)}
        </p>
      )
    }
    return (
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
          onSubOrganizationCreate={requestCreate}
          onSelectChild={select}
          onSave={save}
          onCancel={resetDraft}
          onDelete={() => setConfirmDelete(true)}
          renderActionBar={renderActionBar}
        />
      )
    )
  }

  return (
    <>
      {/* Measured, not the branch content, so the ResizeObserver keeps observing
          across the two-pane <-> Drawer switch instead of losing its element. */}
      <div ref={layoutRef}>
        {isWide ? (
          <div className="grid grid-cols-[300px_minmax(0,1fr)] gap-6">
            {tree}
            {/* --org-panel-bg lets the sticky action bar blend into its surface. */}
            <div
              className="min-w-0"
              style={{ '--org-panel-bg': 'var(--color-dark-50)' } as CSSProperties}
            >
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
      </div>

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
