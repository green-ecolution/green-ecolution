import React from 'react'
import PrimaryButton from './buttons/PrimaryButton'
import SecondaryButton from './buttons/SecondaryButton'
import { X } from 'lucide-react'

interface ModalProps {
  title: string
  description: string
  confirmText: string
  onConfirm?: () => void
  onCancel: () => void
  isOpen: boolean
  showButtons?: boolean
  children?: React.ReactNode
}

const Modal: React.FC<ModalProps> = ({
  title,
  description,
  confirmText,
  onConfirm,
  onCancel,
  isOpen,
  showButtons = true,
  children,
}) => {
  return (
    <>
      <div
        onClick={onCancel}
        className={`bg-dark-900/90 fixed inset-0 z-[1000] ${isOpen ? 'block' : 'hidden'}`}
      ></div>
      <div
        role="dialog"
        aria-modal="true"
        className={`fixed font-nunito-sans z-[1010] inset-x-4 shadow-xl bg-white top-1/2 -translate-y-1/2 p-6 rounded-xl mx-auto max-w-[30rem] ${isOpen ? 'block' : 'hidden'}`}
      >
        <div className="flex items-center justify-between gap-x-5 mb-5">
          <h2 className="text-xl font-lato font-semibold">{title}</h2>
          <button
            aria-label="Close Dialog"
            className="text-dark-400 hover:text-dark-600 stroke-1"
            onClick={onCancel}
            type="button"
          >
            <X />
          </button>
        </div>
        <p className="mb-6 text-base text-gray-600">{description}</p>

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
      </div>
    </>
  )
}

export default Modal
