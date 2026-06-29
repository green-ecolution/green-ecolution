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
import type { UseFormNavigationBlockerReturn } from '@/hooks/form/useFormNavigationBlocker'

interface UnsavedChangesDialogProps {
  blocker: UseFormNavigationBlockerReturn
}

const UnsavedChangesDialog = ({ blocker }: UnsavedChangesDialogProps) => (
  <AlertDialog open={blocker.isModalOpen} onOpenChange={(open) => !open && blocker.closeModal()}>
    <AlertDialogContent>
      <AlertDialogHeader>
        <AlertDialogTitle>Seite verlassen?</AlertDialogTitle>
        <AlertDialogDescription>{blocker.message}</AlertDialogDescription>
      </AlertDialogHeader>
      <AlertDialogFooter>
        <AlertDialogCancel onClick={blocker.closeModal}>
          Abbrechen
          <X />
        </AlertDialogCancel>
        <AlertDialogAction onClick={blocker.confirmLeave}>
          Verlassen
          <MoveRight className="icon-arrow-animate" />
        </AlertDialogAction>
      </AlertDialogFooter>
    </AlertDialogContent>
  </AlertDialog>
)

export default UnsavedChangesDialog
