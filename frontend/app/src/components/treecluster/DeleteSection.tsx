import { MoveRight } from 'lucide-react'
import { useState } from 'react'
import { useMutation } from '@tanstack/react-query'
import Modal from '../general/Modal'
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
      <button
        type="submit"
        onClick={() => setIsModalOpen(true)}
        className="mt-10 group flex items-center gap-x-2 text-red font-medium text-base mb-4"
      >
        {type === 'archive' ? 'Archivieren' : 'Löschen'}
        <MoveRight className="w-4 h-4 transition-all ease-in-out duration-300 group-hover:translate-x-1" />
      </button>

      <Modal
        title={`Soll ${entityName} wirklich ${actionText} werden?`}
        description={`Sobald ${entityName} ${actionText} wurde, können die Daten nicht wieder hergestellt werden.`}
        confirmText="Bestätigen"
        onConfirm={() => mutate()}
        onCancel={() => setIsModalOpen(false)}
        isOpen={isModalOpen}
      />
    </>
  )
}

export default DeleteSection
