import { MoveRight, X } from 'lucide-react'
import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import {
  AlertDialog,
  AlertDialogContent,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogAction,
  AlertDialogCancel,
  Button,
} from '@green-ecolution/ui'
import createToast from '@/hooks/createToast'
import { LinkProps, useNavigate } from '@tanstack/react-router'
import type { Aggregate } from '@/api/queries'
import { useInvalidateAggregates } from '@/lib/queryInvalidation'
import { useTranslation } from 'react-i18next'
import { resolveApiError } from '@/lib/apiError'

interface DeleteSectionProps {
  mutationFn: () => Promise<unknown>
  entityName: string
  type?: 'archive' | 'delete'
  redirectUrl: LinkProps
  /** Aggregates whose lists and details still contain the removed entity. */
  invalidates: readonly Aggregate[]
}

const DeleteSection: React.FC<DeleteSectionProps> = ({
  mutationFn,
  entityName,
  type = 'delete',
  redirectUrl,
  invalidates,
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const navigate = useNavigate()
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const { t } = useTranslation(['errors', 'common'])

  const actionText =
    type === 'archive'
      ? t('common:deleteSection.archivedParticiple')
      : t('common:deleteSection.deletedParticiple')
  const capitalizedEntity = `${entityName.charAt(0).toUpperCase()}${entityName.slice(1)}`
  const failureKey = type === 'archive' ? 'frame.archiveFailed' : 'frame.deleteFailed'

  const { mutate } = useMutation({
    mutationFn,
    onSuccess: () => {
      setIsModalOpen(false)
      navigate(redirectUrl)
        // After leaving: a page still showing the removed entity would refetch
        // it into a 404.
        .then(() => invalidate(invalidates))
        .then(() =>
          showToast(
            t('common:deleteSection.successToast', {
              entity: capitalizedEntity,
              action: actionText,
            }),
            'success',
          ),
        )
        .catch(() => showToast(t('common:deleteSection.unexpectedError'), 'error'))
    },
    onError: (error: unknown) => {
      void resolveApiError(error).then((info) =>
        showToast(t(failureKey, { entity: capitalizedEntity, reason: info.message }), 'error'),
      )

      console.error(error)
      setIsModalOpen(false)
    },
  })

  return (
    <>
      <Button
        variant="link-destructive"
        onClick={() => setIsModalOpen(true)}
        className="mt-10 mb-4 px-0 group"
      >
        {type === 'archive' ? t('common:actions.archive') : t('common:actions.delete')}
        <MoveRight className="transition-transform duration-base ease-emphasized group-hover:translate-x-1 motion-reduce:transition-none" />
      </Button>

      <AlertDialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {t('common:deleteSection.confirmTitle', { entity: entityName, action: actionText })}
            </AlertDialogTitle>
            <AlertDialogDescription>
              {t('common:deleteSection.confirmDescription', {
                entity: entityName,
                action: actionText,
              })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              {t('common:actions.cancel')}
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={() => mutate()}>
              {t('common:actions.confirm')}
              <MoveRight className="icon-arrow-animate" />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export default DeleteSection
