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
import useToast from '@/hooks/useToast'
import { LinkProps, useNavigate } from '@tanstack/react-router'

interface DeleteSectionProps {
  mutationFn: () => Promise<unknown>
  entityName: string
  type?: 'archive' | 'delete'
  redirectUrl: LinkProps
}

const DeleteSection: React.FC<DeleteSectionProps> = ({
  mutationFn,
  entityName,
  type = 'delete',
  redirectUrl,
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false)
  const navigate = useNavigate()
  const showToast = useToast()

  const actionText = type === 'archive' ? 'archiviert' : 'gelöscht'

  const { mutate } = useMutation({
    mutationFn,
    onSuccess: () => {
      setIsModalOpen(false)
      navigate(redirectUrl)
        .then(() =>
          showToast(
            `${entityName.charAt(0).toUpperCase()}${entityName.slice(1)} wurde erfolgreich ${actionText}.`,
            'success',
          ),
        )
        .catch(() => showToast('Ein fehler is aufgetreten', 'error'))
    },
    onError: (error: unknown) => {
      if (error instanceof Error) {
        showToast(error.message, 'error')
      } else {
        showToast(
          'Leider ist ein Fehler eim löschen des Baumes aufgetreten. Versuche es später erneut.',
          'error',
        )
      }

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
        {type === 'archive' ? 'Archivieren' : 'Löschen'}
        <MoveRight className="transition-all duration-300 group-hover:translate-x-1" />
      </Button>

      <AlertDialog open={isModalOpen} onOpenChange={setIsModalOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              Soll {entityName} wirklich {actionText} werden?
            </AlertDialogTitle>
            <AlertDialogDescription>
              Sobald {entityName} {actionText} wurde, können die Daten nicht wieder hergestellt
              werden.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              Abbrechen
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={() => mutate()}>
              Bestätigen
              <MoveRight />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}

export default DeleteSection
