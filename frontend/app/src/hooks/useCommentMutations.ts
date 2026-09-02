import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commentApi } from '@/api/backendApi'
import type { CommentSubject } from '@/api/queries'
import createToast from '@/hooks/createToast'
import { useTranslation } from 'react-i18next'

export interface UpdateCommentVariables {
  commentId: string
  body: string
}

export const useCommentMutations = (subject: CommentSubject, parentId: string) => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('comments')
  const queryKey = ['comments', subject, parentId]

  const create = useMutation({
    mutationFn: (body: string) =>
      subject === 'cluster'
        ? commentApi.createClusterComment({ clusterId: parentId, createCommentRequest: { body } })
        : commentApi.createWateringPlanComment({
            wateringPlanId: parentId,
            createCommentRequest: { body },
          }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey })
      showToast(t('toasts.createSuccess'))
    },
    onError: () => {
      showToast(t('toasts.createError'), 'error')
    },
  })

  const update = useMutation({
    mutationFn: ({ commentId, body }: UpdateCommentVariables) =>
      subject === 'cluster'
        ? commentApi.updateClusterComment({
            clusterId: parentId,
            commentId,
            updateCommentRequest: { body },
          })
        : commentApi.updateWateringPlanComment({
            wateringPlanId: parentId,
            commentId,
            updateCommentRequest: { body },
          }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey })
      showToast(t('toasts.updateSuccess'))
    },
    onError: () => {
      showToast(t('toasts.updateError'), 'error')
    },
  })

  const remove = useMutation({
    mutationFn: (commentId: string) =>
      subject === 'cluster'
        ? commentApi.deleteClusterComment({ clusterId: parentId, commentId })
        : commentApi.deleteWateringPlanComment({ wateringPlanId: parentId, commentId }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey })
      showToast(t('toasts.deleteSuccess'))
    },
    onError: () => {
      showToast(t('toasts.deleteError'), 'error')
    },
  })

  return { create, update, remove }
}
