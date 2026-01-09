import PrimaryButton from '../buttons/PrimaryButton'
import SecondaryButton from '../buttons/SecondaryButton'
import { BaseModal } from './BaseModal'
import { ConfirmModalProps } from './types'

export function ConfirmModal({
  title,
  description,
  confirmText,
  onConfirm,
  onCancel,
  isOpen,
  showButtons = true,
  children,
}: ConfirmModalProps) {
  return (
    <BaseModal isOpen={isOpen} onClose={onCancel} title={title} description={description}>
      {children}

      {showButtons && (
        <div className="flex flex-wrap gap-2 items-center mt-6">
          <PrimaryButton isDanger onClick={onConfirm} label={confirmText} />
          <SecondaryButton
            label="Abbrechen"
            className="flex items-center rounded-md text-gray-500 bg-transparent hover:bg-gray-500 px-4 py-2"
            onClick={onCancel}
          />
        </div>
      )}
    </BaseModal>
  )
}

export default ConfirmModal
