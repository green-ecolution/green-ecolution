import { useMutation, useQueryClient } from '@tanstack/react-query'
import { commentApi } from '@/api/backendApi'
import { commentQueryKey, type CommentSubject } from '@/api/queries'
import createToast from '@/hooks/createToast'
import type { TargetRefDto } from '@green-ecolution/backend-client'
import { useTranslation } from 'react-i18next'

export interface UpdateCommentVariables {
  commentId: string
  body: string
}

const targetParams = ({ module, kind, id }: TargetRefDto) => ({ module, kind, targetId: id })

export const useCommentMutations = (subject: CommentSubject) => {
  const queryClient = useQueryClient()
  const showToast = createToast()
  const { t } = useTranslation('comments')
  const queryKey = commentQueryKey(subject)

  const create = useMutation({
    mutationFn: (body: string) =>
      subject.kind === 'plan'
        ? commentApi.createPlanComment({ planId: subject.id, createCommentRequest: { body } })
        : commentApi.createTargetComment({
            ...targetParams(subject.target),
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
      subject.kind === 'plan'
        ? commentApi.updatePlanComment({
            planId: subject.id,
            commentId,
            updateCommentRequest: { body },
          })
        : commentApi.updateTargetComment({
            ...targetParams(subject.target),
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
      subject.kind === 'plan'
        ? commentApi.deletePlanComment({ planId: subject.id, commentId })
        : commentApi.deleteTargetComment({ ...targetParams(subject.target), commentId }),
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
