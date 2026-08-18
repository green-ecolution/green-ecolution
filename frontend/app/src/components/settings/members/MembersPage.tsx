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
  Button,
  Drawer,
  DrawerContent,
  DrawerFooter,
} from '@green-ecolution/ui'
import type { UserResponse } from '@/api/backendApi'
import { organizationQueries, roleQueries, userQueries } from '@/api/queries'
import { useUserMutations } from '@/hooks/useUserMutations'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import MemberActionButtons from './MemberActionButtons'
import MemberDetail from './MemberDetail'
import MemberList from './MemberList'
import { buildTree, flatten } from '../organization/organizationTree'
import { fullNameOf, isFiltered } from './memberList'
import { useMemberProfileDraft } from './useMemberProfileDraft'

const PER_PAGE = 50
const SEARCH_DEBOUNCE_MS = 300

const statusOf = (error: unknown): number | undefined =>
  (error as { response?: { status?: number } } | null)?.response?.status

const roleErrorMessage = (error: unknown): string | null => {
  switch (statusOf(error)) {
    case 403:
      return 'Diese Rolle enthält Rechte, die über deine eigenen hinausgehen.'
    case 409:
      return 'Am eigenen Konto lassen sich Rollen nicht ändern.'
    default:
      return null
  }
}

const organizationErrorMessage = (error: unknown): string | null => {
  switch (statusOf(error)) {
    case 403:
      return 'In dieser Organisation darfst du keine Personen verwalten.'
    case 409:
      return 'Die eigene Organisation lässt sich hier nicht ändern.'
    default:
      return null
  }
}

const trimmedOrNull = (value: string): string | null => {
  const trimmed = value.trim()
  return trimmed.length === 0 ? null : trimmed
}

const MembersPage = () => {
  const canUpdate = useHasPermission(['user:update'])
  const canReadRoles = useHasPermission(['role:read'])
  const canReadOrganizations = useHasPermission(['organization:read'])
  const isDesktop = useMediaQuery('(min-width: 1024px)')

  const [search, setSearch] = useState('')
  const [debouncedSearch, setDebouncedSearch] = useState('')
  const [organizationFilter, setOrganizationFilter] = useState<string | null>(null)
  const [roleFilter, setRoleFilter] = useState<string | null>(null)
  const [page, setPage] = useState(1)
  const [selectedId, setSelectedId] = useState<string | null>(null)
  const [loadedUser, setLoadedUser] = useState<UserResponse | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)
  const [pendingSelection, setPendingSelection] = useState<string | null>(null)
  const [pendingClose, setPendingClose] = useState(false)
  const [confirmOrgChange, setConfirmOrgChange] = useState<string | null>(null)

  useEffect(() => {
    const timeout = setTimeout(() => setDebouncedSearch(search.trim()), SEARCH_DEBOUNCE_MS)
    return () => clearTimeout(timeout)
  }, [search])

  const { data: me } = useQuery(userQueries.me())

  const { data: userPage, isLoading: usersLoading } = useQuery(
    userQueries.list({
      page,
      perPage: PER_PAGE,
      query: debouncedSearch || undefined,
      organizationId: organizationFilter ?? undefined,
      roleId: roleFilter ?? undefined,
    }),
  )

  const { data: organizations } = useQuery({
    ...organizationQueries.list(),
    enabled: canReadOrganizations,
  })

  const members = useMemo(() => userPage?.data ?? [], [userPage])
  const selected = members.find((user) => user.id === selectedId) ?? null
  const selectedOrgId = selected?.organization?.id ?? null

  // Assignable roles come from the selected person's own organization, never
  // from the caller's wider visible subtree: a role grants its permissions
  // for the owning organization and its whole subtree, so offering a parent
  // organization's role here would let the caller hand out rights over that
  // parent's entire subtree (the backend rejects the mismatch with 409 as a
  // second line of defense). Do not swap this for `roleQueries.visible()` —
  // that query is for the list filter below only.
  const { data: assignableOrgRoles } = useQuery({
    ...roleQueries.org(selectedOrgId ?? ''),
    enabled: canReadRoles && selectedOrgId !== null,
  })

  // The list filter covers every organization the caller can see, unlike the
  // assignment picker above.
  const { data: filterVisibleRoles } = useQuery({
    ...roleQueries.visible(),
    enabled: canReadRoles,
  })

  const { assignRole, revokeRole, setOrganization, updateProfile } = useUserMutations()
  const draftState = useMemberProfileDraft()
  const { draft, dirty, edit, discard } = draftState

  const blocker = useBlocker({ shouldBlockFn: () => dirty, withResolver: true })

  const total = userPage?.pagination.totalRecords ?? 0
  const totalPages = userPage?.pagination.totalPages ?? 1
  const filtered = isFiltered(search, organizationFilter, roleFilter)

  const assignableRoles = (assignableOrgRoles ?? []).filter(
    (role) => !role.isTemplate && !(selected?.roles ?? []).some((held) => held.id === role.id),
  )
  const filterRoles = (filterVisibleRoles ?? []).filter((role) => !role.isTemplate)
  // Tree order instead of the API's alphabetical order, so both filters read like
  // the organization structure they describe.
  const visibleOrganizations = organizations ?? []
  const ownOrgId = me?.organization?.id
  const orgRoot = ownOrgId ? buildTree(visibleOrganizations, ownOrgId) : null
  const organizationList = orgRoot ? flatten(orgRoot) : visibleOrganizations
  const isSelf = selected !== null && selected.id === me?.id

  useEffect(() => {
    // Only on desktop, where the detail pane is always visible. On mobile the
    // detail is a drawer, so auto-selecting would pop it open on page load.
    if (!isDesktop || selectedId !== null || members.length === 0) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- picks a default once query data loads, so the detail pane isn't empty
    setSelectedId(members[0].id)
  }, [isDesktop, selectedId, members])

  useEffect(() => {
    // Keyed on the object, not the id: after a save the invalidated query hands
    // back a new object and the draft must pick up the server's truth. Structural
    // sharing keeps the reference stable when nothing changed, so an in-progress
    // edit survives a refetch that returns the same data.
    if (selected === loadedUser) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- loads the freshly fetched member into the draft
    setLoadedUser(selected)
    if (selected) edit(selected)
    // A person nobody can see must not keep a dirty draft around: it would arm
    // the unsaved-changes blocker for a form that cannot be reached or saved.
    else discard()
  }, [selected, loadedUser, edit, discard])

  useEffect(() => {
    // A selection that vanished from an unfiltered list is gone for good (paged
    // away or deleted), so drop it and let the auto-selection take over. With a
    // filter active the selection is only hidden, and the pane offers a reset.
    if (selectedId === null || selected !== null || usersLoading || filtered) return
    // eslint-disable-next-line react-hooks/set-state-in-effect, react-x/set-state-in-effect -- drops a selection the refreshed list no longer contains
    setSelectedId(null)
  }, [selectedId, selected, usersLoading, filtered])

  const goTo = (userId: string) => {
    setSelectedId(userId)
    setDetailOpen(true)
    assignRole.reset()
    revokeRole.reset()
    setOrganization.reset()
    updateProfile.reset()
  }

  const select = (user: UserResponse) => {
    if (user.id === selectedId) {
      setDetailOpen(true)
      return
    }
    if (dirty) {
      setPendingSelection(user.id)
      return
    }
    goTo(user.id)
  }

  const resetDraft = () => {
    if (selected) edit(selected)
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
    const next = pendingSelection
    setPendingSelection(null)
    if (next !== null) goTo(next)
  }

  const changeSearch = (value: string) => {
    setSearch(value)
    setPage(1)
  }

  const changeOrganizationFilter = (orgId: string | null) => {
    setOrganizationFilter(orgId)
    setPage(1)
    // A role belongs to exactly one organization, and the backend requires a
    // holder's organization to match the role's. A role from elsewhere can
    // therefore never match, and leaving it selected strands the list on an
    // empty result the user cannot explain.
    if (orgId !== null && roleFilter !== null) {
      const role = filterRoles.find((candidate) => candidate.id === roleFilter)
      if (role?.organizationId !== orgId) setRoleFilter(null)
    }
  }

  const changeRoleFilter = (roleId: string | null) => {
    setRoleFilter(roleId)
    setPage(1)
  }

  const resetFilters = () => {
    setSearch('')
    setDebouncedSearch('')
    setOrganizationFilter(null)
    setRoleFilter(null)
    setPage(1)
  }

  const assign = (roleId: string) => {
    if (!selected) return
    assignRole.mutate(
      { userId: selected.id, roleId },
      // Clears a stale failure of the sibling mutation, which shares the card's
      // single error slot.
      { onSuccess: () => revokeRole.reset() },
    )
  }

  const revoke = (roleId: string) => {
    if (!selected) return
    revokeRole.mutate({ userId: selected.id, roleId }, { onSuccess: () => assignRole.reset() })
  }

  const moveToOrganization = () => {
    if (!selected || confirmOrgChange === null) return
    setOrganization.mutate({ userId: selected.id, organizationId: confirmOrgChange })
  }

  const save = () => {
    if (!draft) return
    updateProfile.mutate(
      {
        userId: draft.userId,
        employeeId: trimmedOrNull(draft.employeeId),
        phoneNumber: trimmedOrNull(draft.phoneNumber),
        // Replace-style endpoint: the draft round-trips the avatar so saving a
        // profile cannot wipe it.
        avatarUrl: draft.avatarUrl,
        status: draft.status,
        drivingLicenses: draft.drivingLicenses,
      },
      {
        onSuccess: (updated) => {
          setLoadedUser(updated)
          edit(updated)
        },
      },
    )
  }

  const list = (
    <MemberList
      members={members}
      total={total}
      page={page}
      totalPages={totalPages}
      loading={usersLoading}
      selectedId={selectedId}
      search={search}
      organizationFilter={organizationFilter}
      roleFilter={roleFilter}
      organizations={organizationList}
      roles={filterRoles}
      onSearchChange={changeSearch}
      onOrganizationFilterChange={changeOrganizationFilter}
      onRoleFilterChange={changeRoleFilter}
      onPageChange={setPage}
      onSelect={select}
    />
  )

  const renderDetail = (renderActionBar: boolean) => {
    if (selectedId !== null && selected === null) {
      if (!filtered) return null
      return (
        <div className="flex flex-col items-start gap-3">
          <p className="text-sm text-dark-600">
            Die ausgewählte Person passt nicht zu Suche und Filter.
          </p>
          <Button type="button" variant="outline" onClick={resetFilters}>
            Suche und Filter zurücksetzen
          </Button>
        </div>
      )
    }

    return (
      selected &&
      draft && (
        <MemberDetail
          user={selected}
          draft={draft}
          dirty={dirty}
          isSelf={isSelf}
          canUpdate={canUpdate}
          canReadRoles={canReadRoles}
          canReadOrganizations={canReadOrganizations}
          organizations={organizationList}
          assignableRoles={assignableRoles}
          roleError={roleErrorMessage(assignRole.error ?? revokeRole.error)}
          organizationError={organizationErrorMessage(setOrganization.error)}
          saving={updateProfile.isPending}
          onStatusChange={draftState.setStatus}
          onDrivingLicensesChange={draftState.setDrivingLicenses}
          onPhoneNumberChange={draftState.setPhoneNumber}
          onEmployeeIdChange={draftState.setEmployeeId}
          onRoleAssign={assign}
          onRoleRevoke={revoke}
          onOrganizationChange={setConfirmOrgChange}
          onSave={save}
          onCancel={resetDraft}
          renderActionBar={renderActionBar}
        />
      )
    )
  }

  const targetOrganization = organizationList.find((org) => org.id === confirmOrgChange) ?? null
  const sourceOrganizationName = selected?.organization?.name ?? null

  return (
    <>
      {isDesktop ? (
        <div className="grid grid-cols-[300px_1fr] gap-6">
          {list}
          {/* --member-panel-bg lets the sticky action bar blend into its surface. */}
          <div style={{ '--member-panel-bg': 'var(--color-dark-50)' } as CSSProperties}>
            {renderDetail(true)}
          </div>
        </div>
      ) : (
        <>
          {list}
          <Drawer open={detailOpen} onOpenChange={requestClose}>
            <DrawerContent className="max-h-[90vh]">
              {/* Content scrolls in its own region; the actions live in a fixed
                  footer below so they stay anchored to the drawer bottom. */}
              <div className="min-h-0 flex-1 overflow-y-auto p-4">{renderDetail(false)}</div>
              {draft && dirty && canUpdate && (
                <DrawerFooter className="flex-col-reverse gap-2 border-t border-dark-200 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
                  <MemberActionButtons
                    saving={updateProfile.isPending}
                    onSave={save}
                    onCancel={resetDraft}
                  />
                </DrawerFooter>
              )}
            </DrawerContent>
          </Drawer>
        </>
      )}

      <AlertDialog
        open={confirmOrgChange !== null}
        onOpenChange={(open) => {
          if (!open) setConfirmOrgChange(null)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Person verschieben?</AlertDialogTitle>
            <AlertDialogDescription>
              {selected && targetOrganization
                ? sourceOrganizationName
                  ? `${fullNameOf(selected)} wechselt von ${sourceOrganizationName} zu ${targetOrganization.name}. Die Änderung wirkt sofort, zugewiesene Rollen bleiben bestehen.`
                  : `${fullNameOf(selected)} wird ${targetOrganization.name} zugeordnet. Die Änderung wirkt sofort, zugewiesene Rollen bleiben bestehen.`
                : null}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Abbrechen</AlertDialogCancel>
            <AlertDialogAction onClick={moveToOrganization}>Verschieben</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={pendingSelection !== null || pendingClose}
        onOpenChange={(open) => {
          if (open) return
          setPendingSelection(null)
          setPendingClose(false)
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Änderungen verwerfen?</AlertDialogTitle>
            <AlertDialogDescription>
              Du hast Änderungen am Profil dieser Person, die noch nicht gespeichert sind.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Weiter bearbeiten</AlertDialogCancel>
            <AlertDialogAction onClick={confirmDiscard}>Verwerfen</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {blocker.status === 'blocked' && (
        <AlertDialog open onOpenChange={() => blocker.reset?.()}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
              <AlertDialogDescription>
                Deine Änderungen am Profil dieser Person sind noch nicht gespeichert.
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

export default MembersPage
