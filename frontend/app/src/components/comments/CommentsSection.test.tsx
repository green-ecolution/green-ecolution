import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { type Permissions } from '@/lib/auth/permissions'

const CLUSTER_ID = '11111111-1111-4111-8111-111111111111'
const ME_ID = 'user-me'
const OTHER_ID = 'user-other'

const listClusterComments = vi.fn()
const createClusterComment = vi.fn()
const updateClusterComment = vi.fn()
const deleteClusterComment = vi.fn()
const getMe = vi.fn()

vi.mock('@/api/backendApi', () => ({
  commentApi: {
    listClusterComments: (...args: unknown[]) => listClusterComments(...args) as unknown,
    createClusterComment: (...args: unknown[]) => createClusterComment(...args) as unknown,
    updateClusterComment: (...args: unknown[]) => updateClusterComment(...args) as unknown,
    deleteClusterComment: (...args: unknown[]) => deleteClusterComment(...args) as unknown,
  },
  userApi: {
    getMe: (...args: unknown[]) => getMe(...args) as unknown,
  },
}))

const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const permissions = vi.fn((): Permissions => new Set<string>())
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

const jwtPayload = {
  preferred_username: 'erika.mustermann',
  email: 'erika.mustermann@example.com',
  given_name: 'Erika',
  family_name: 'Mustermann',
}
const accessToken = `x.${btoa(JSON.stringify(jwtPayload))}.y`
vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true, accessToken }),
}))

const { default: CommentsSection } = await import('./CommentsSection')

const comment = (overrides: Partial<Record<string, unknown>> = {}) => ({
  id: 'comment-1',
  authorId: OTHER_ID,
  authorName: 'Max Beispiel',
  body: 'Erster Kommentar',
  createdAt: '2024-01-01T00:00:00Z',
  editedAt: null,
  ...overrides,
})

const pageOf = (data: unknown[], overrides: Partial<Record<string, unknown>> = {}) => ({
  data,
  pagination: {
    currentPage: 1,
    nextPage: null,
    perPage: 20,
    prevPage: null,
    totalPages: 1,
    totalRecords: data.length,
    ...overrides,
  },
})

const renderSection = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return render(
    <QueryClientProvider client={queryClient}>
      <CommentsSection subject="cluster" parentId={CLUSTER_ID} />
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.clearAllMocks()
  permissions.mockReturnValue(new Set<string>())
  getMe.mockResolvedValue({ id: ME_ID, roles: [] })
})

describe('CommentsSection', () => {
  it('renders the fetched comments', async () => {
    listClusterComments.mockResolvedValue(
      pageOf([
        comment({ id: 'c1', body: 'Erster Kommentar' }),
        comment({ id: 'c2', body: 'Zweiter Kommentar' }),
      ]),
    )

    renderSection()

    expect(await screen.findByText('Erster Kommentar')).toBeInTheDocument()
    expect(screen.getByText('Zweiter Kommentar')).toBeInTheDocument()
  })

  it('submits a new comment through the API and clears the field', async () => {
    listClusterComments.mockResolvedValue(pageOf([]))
    createClusterComment.mockResolvedValue(
      comment({ id: 'new-1', authorId: ME_ID, body: 'Neuer Kommentar' }),
    )
    const user = userEvent.setup()

    renderSection()
    // Wait for the list request to settle before typing, so the submit is not
    // racing the initial fetch.
    await waitFor(() => expect(listClusterComments).toHaveBeenCalled())

    const textbox = screen.getByRole('textbox', { name: 'Kommentar hinzufügen' })
    await user.type(textbox, 'Neuer Kommentar')
    await user.click(screen.getByRole('button', { name: 'Kommentieren' }))

    await waitFor(() =>
      expect(createClusterComment).toHaveBeenCalledWith({
        clusterId: CLUSTER_ID,
        createCommentRequest: { body: 'Neuer Kommentar' },
      }),
    )
    await waitFor(() => expect(textbox).toHaveValue(''))
  })

  it("shows the edit action only on the signed-in user's own comment", async () => {
    listClusterComments.mockResolvedValue(
      pageOf([
        comment({ id: 'own', authorId: ME_ID, body: 'Mein Kommentar' }),
        comment({ id: 'foreign', authorId: OTHER_ID, body: 'Fremder Kommentar' }),
      ]),
    )

    renderSection()
    await screen.findByText('Mein Kommentar')

    expect(screen.getAllByRole('button', { name: 'Bearbeiten' })).toHaveLength(1)
  })

  it('shows delete only for the comment author without moderator rights', async () => {
    listClusterComments.mockResolvedValue(
      pageOf([
        comment({ id: 'own', authorId: ME_ID, body: 'Mein Kommentar' }),
        comment({ id: 'foreign', authorId: OTHER_ID, body: 'Fremder Kommentar' }),
      ]),
    )

    renderSection()
    await screen.findByText('Mein Kommentar')

    expect(screen.getAllByRole('button', { name: 'Löschen' })).toHaveLength(1)
  })

  it('does not show delete on a foreign comment for a moderator with delete rights', async () => {
    permissions.mockReturnValue(new Set(['tree_cluster:delete']))
    listClusterComments.mockResolvedValue(
      pageOf([
        comment({ id: 'own', authorId: ME_ID, body: 'Mein Kommentar' }),
        comment({ id: 'foreign', authorId: OTHER_ID, body: 'Fremder Kommentar' }),
      ]),
    )

    renderSection()
    await screen.findByText('Mein Kommentar')

    expect(screen.getAllByRole('button', { name: 'Löschen' })).toHaveLength(1)
  })

  it('only deletes a comment once the confirmation dialog is accepted', async () => {
    listClusterComments.mockResolvedValue(
      pageOf([comment({ id: 'own', authorId: ME_ID, body: 'Mein Kommentar' })]),
    )
    deleteClusterComment.mockResolvedValue(undefined)
    const user = userEvent.setup()

    renderSection()
    await screen.findByText('Mein Kommentar')

    await user.click(screen.getByRole('button', { name: 'Löschen' }))
    expect(deleteClusterComment).not.toHaveBeenCalled()

    const dialog = await screen.findByRole('alertdialog')
    expect(within(dialog).getByText('Kommentar löschen?')).toBeInTheDocument()

    await user.click(within(dialog).getByRole('button', { name: 'Löschen' }))

    await waitFor(() =>
      expect(deleteClusterComment).toHaveBeenCalledWith({
        clusterId: CLUSTER_ID,
        commentId: 'own',
      }),
    )
  })

  it('fetches the second page when "load more" is clicked', async () => {
    listClusterComments.mockImplementation(({ page }: { page: number }) =>
      Promise.resolve(
        page === 1
          ? pageOf([comment({ id: 'p1', body: 'Seite eins' })], { nextPage: 2, totalPages: 2 })
          : pageOf([comment({ id: 'p2', body: 'Seite zwei' })], { currentPage: 2, totalPages: 2 }),
      ),
    )
    const user = userEvent.setup()

    renderSection()
    await screen.findByText('Seite eins')

    await user.click(screen.getByRole('button', { name: 'Weitere Kommentare laden' }))

    await waitFor(() => expect(screen.getByText('Seite zwei')).toBeInTheDocument())
    expect(listClusterComments).toHaveBeenCalledWith({
      clusterId: CLUSTER_ID,
      page: 2,
      perPage: 20,
    })
  })

  // 'Noch keine Kommentare.' is the UI library's own empty-state copy. It can
  // only appear if the list gets rendered with nothing in it, so asserting its
  // absence is what guards the "show nothing when empty" rule.
  it('shows nothing but the composer when there are no comments', async () => {
    listClusterComments.mockResolvedValue(pageOf([]))

    renderSection()

    expect(await screen.findByRole('textbox')).toBeInTheDocument()
    await waitFor(() =>
      expect(screen.queryByText('Noch keine Kommentare.')).not.toBeInTheDocument(),
    )
  })

  it('explains a failed load instead of showing an empty list', async () => {
    listClusterComments.mockRejectedValue(new Error('boom'))

    renderSection()

    expect(await screen.findByText('Kommentare konnten nicht geladen werden')).toBeInTheDocument()
    expect(screen.queryByText('Noch keine Kommentare.')).not.toBeInTheDocument()
  })

  it('keeps already loaded comments visible when a later page fails', async () => {
    listClusterComments
      .mockResolvedValueOnce(
        pageOf([comment({ id: 'p1', body: 'Seite eins' })], { nextPage: 2, totalPages: 2 }),
      )
      .mockRejectedValueOnce(new Error('boom'))
    const user = userEvent.setup()

    renderSection()
    await screen.findByText('Seite eins')

    await user.click(screen.getByRole('button', { name: 'Weitere Kommentare laden' }))

    expect(await screen.findByText('Kommentare konnten nicht geladen werden')).toBeInTheDocument()
    expect(screen.getByText('Seite eins')).toBeInTheDocument()
  })
})
