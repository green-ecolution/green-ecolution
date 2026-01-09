import { useEffect, useId, useCallback } from 'react'
import { createPortal } from 'react-dom'
import { X } from 'lucide-react'
import { useFocusTrap } from '@/hooks/useFocusTrap'
import { BaseModalProps, MODAL_Z_INDEX, MODAL_SIZE_CLASSES } from './types'

export function BaseModal({
  isOpen,
  onClose,
  children,
  title,
  description,
  closeOnBackdropClick = true,
  closeOnEscape = true,
  trapFocus = true,
  returnFocusOnClose = true,
  size = 'md',
  className = '',
}: BaseModalProps) {
  const titleId = useId()
  const descriptionId = useId()

  const focusTrapRef = useFocusTrap<HTMLDivElement>({
    enabled: isOpen && trapFocus,
    returnFocusOnDeactivate: returnFocusOnClose,
  })

  const handleEscapeKey = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape' && closeOnEscape) {
        onClose()
      }
    },
    [closeOnEscape, onClose],
  )

  const handleBackdropClick = useCallback(() => {
    if (closeOnBackdropClick) {
      onClose()
    }
  }, [closeOnBackdropClick, onClose])

  useEffect(() => {
    if (!isOpen) return

    document.addEventListener('keydown', handleEscapeKey)
    document.body.style.overflow = 'hidden'

    return () => {
      document.removeEventListener('keydown', handleEscapeKey)
      document.body.style.overflow = ''
    }
  }, [isOpen, handleEscapeKey])

  if (!isOpen) return null

  const modalContent = (
    <>
      <div
        onClick={handleBackdropClick}
        className="bg-dark-900/90 fixed inset-0"
        style={{ zIndex: MODAL_Z_INDEX.backdrop }}
        aria-hidden="true"
      />
      <div
        ref={focusTrapRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={description ? descriptionId : undefined}
        className={`fixed font-nunito-sans inset-x-4 shadow-xl bg-white top-1/2 -translate-y-1/2 p-6 rounded-xl mx-auto ${MODAL_SIZE_CLASSES[size]} ${className}`}
        style={{ zIndex: MODAL_Z_INDEX.content }}
      >
        <div className="flex items-center justify-between gap-x-5 mb-5">
          <h2 id={titleId} className="text-xl font-lato font-semibold">
            {title}
          </h2>
          <button
            aria-label="Dialog schließen"
            className="text-dark-400 hover:text-dark-600 stroke-1 rounded focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-green-dark"
            onClick={onClose}
            type="button"
          >
            <X />
          </button>
        </div>

        {description && (
          <p id={descriptionId} className="mb-6 text-base text-gray-600">
            {description}
          </p>
        )}

        {children}
      </div>
    </>
  )

  return createPortal(modalContent, document.body)
}

export default BaseModal
