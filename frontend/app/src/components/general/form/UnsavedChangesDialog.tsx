import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import type { UseFormNavigationBlockerReturn } from '@/hooks/form/useFormNavigationBlocker'

interface UnsavedChangesDialogProps {
  blocker: UseFormNavigationBlockerReturn
}

const UnsavedChangesDialog = ({ blocker }: UnsavedChangesDialogProps) => {
  const { t } = useTranslation('common')

  return (
    <AlertDialog open={blocker.isModalOpen} onOpenChange={(open) => !open && blocker.closeModal()}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('dialog.unsavedChanges.title')}</AlertDialogTitle>
          <AlertDialogDescription>{blocker.message}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={blocker.closeModal}>
            {t('actions.cancel')}
            <X />
          </AlertDialogCancel>
          <AlertDialogAction onClick={blocker.confirmLeave}>
            {t('dialog.unsavedChanges.confirm')}
            <MoveRight className="icon-arrow-animate" />
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}

export default UnsavedChangesDialog
