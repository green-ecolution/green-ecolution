import { useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery, useInfiniteQuery } from '@tanstack/react-query'
import { formatDistanceToNow } from 'date-fns'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertIcon,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CommentComposer,
  CommentItem,
  CommentList,
  type CommentAuthor,
} from '@green-ecolution/ui'
import { commentQueries, userQueries, type CommentSubject } from '@/api/queries'
import { useCommentMutations } from '@/hooks/useCommentMutations'
import { useCurrentUser } from '@/lib/auth/useCurrentUser'
import { useCurrentUserAvatar } from '@/lib/auth/useCurrentUserAvatar'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useDateLocale } from '@/lib/i18n/useFormatters'

interface CommentsSectionProps {
  subject: CommentSubject
  parentId: string
}

const CommentsSection = ({ subject, parentId }: CommentsSectionProps) => {
  const { t } = useTranslation('comments')
  const headingId = useId()
  const dateLocale = useDateLocale()

  const { data: me } = useQuery(userQueries.me())
  const currentUser = useCurrentUser()
  const avatarUrl = useCurrentUserAvatar()
  const fullName = `${currentUser.firstName} ${currentUser.lastName}`.trim()
  const displayName = fullName || currentUser.username
  const composerAuthor: CommentAuthor = { name: displayName, avatarUrl }

  const canDeleteAny = useHasPermission(
    subject === 'cluster' ? ['tree_cluster:delete'] : ['watering_plan:delete'],
  )

  const commentsQuery = useInfiniteQuery(commentQueries.list(subject, parentId))
  const comments = commentsQuery.data?.pages.flatMap((page) => page.data) ?? []

  // A failed page-two fetch must not hide the comments page one already
  // delivered, so the alert accompanies the list instead of replacing it.
  const showList = !commentsQuery.isError || comments.length > 0

  const { create, update, remove } = useCommentMutations(subject, parentId)
  const [pendingDeleteId, setPendingDeleteId] = useState<string | null>(null)

  const handleCreate = async (body: string) => {
    await create.mutateAsync(body)
  }

  const handleConfirmDelete = () => {
    if (pendingDeleteId) remove.mutate(pendingDeleteId)
  }

  return (
    <section aria-labelledby={headingId}>
      <Card variant="outlined">
        <CardHeader>
          <CardTitle asChild>
            <h2 id={headingId}>{t('heading')}</h2>
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-6">
          <CommentComposer
            author={composerAuthor}
            onSubmit={handleCreate}
            isSubmitting={create.isPending}
            placeholder={t('composer.placeholder')}
          />

          {commentsQuery.isError && (
            <Alert variant="destructive" className="flex w-full gap-4">
              <AlertIcon variant="destructive" />
              <AlertContent>
                <AlertTitle>{t('list.errorTitle')}</AlertTitle>
                <AlertDescription>{t('list.errorDescription')}</AlertDescription>
              </AlertContent>
            </Alert>
          )}

          {showList && (
            <CommentList isLoading={commentsQuery.isLoading} emptyLabel={t('list.empty')}>
              {comments.map((comment) => {
                const isOwnComment = comment.authorId === me?.id
                return (
                  <CommentItem
                    key={comment.id}
                    author={{ name: comment.authorName ?? t('list.unknownAuthor') }}
                    body={comment.body}
                    timestamp={formatDistanceToNow(new Date(comment.createdAt), {
                      addSuffix: true,
                      locale: dateLocale,
                    })}
                    editedLabel={comment.editedAt ? t('list.edited') : undefined}
                    canEdit={isOwnComment}
                    canDelete={isOwnComment || canDeleteAny}
                    isSaving={update.isPending && update.variables?.commentId === comment.id}
                    onEdit={async (body) => {
                      await update.mutateAsync({ commentId: comment.id, body })
                    }}
                    onDelete={() => setPendingDeleteId(comment.id)}
                  />
                )
              })}
            </CommentList>
          )}

          {commentsQuery.hasNextPage && (
            <Button
              variant="outline"
              size="sm"
              className="self-center"
              onClick={() => void commentsQuery.fetchNextPage()}
              disabled={commentsQuery.isFetchingNextPage}
            >
              {t('list.loadMore')}
            </Button>
          )}
        </CardContent>
      </Card>

      <AlertDialog
        open={pendingDeleteId !== null}
        onOpenChange={(open) => !open && setPendingDeleteId(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('deleteDialog.title')}</AlertDialogTitle>
            <AlertDialogDescription>{t('deleteDialog.description')}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('deleteDialog.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleConfirmDelete}>
              {t('deleteDialog.confirm')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </section>
  )
}

export default CommentsSection
