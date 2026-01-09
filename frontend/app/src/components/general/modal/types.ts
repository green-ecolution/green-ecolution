import { ReactNode } from 'react'

export interface BaseModalProps {
  isOpen: boolean
  onClose: () => void
  children: ReactNode
  title: string
  description?: string
  closeOnBackdropClick?: boolean
  closeOnEscape?: boolean
  trapFocus?: boolean
  returnFocusOnClose?: boolean
  size?: 'sm' | 'md' | 'lg'
  className?: string
}

export interface ConfirmModalProps {
  title: string
  description: string
  confirmText: string
  onConfirm?: () => void
  onCancel: () => void
  isOpen: boolean
  showButtons?: boolean
  children?: ReactNode
}

export const MODAL_Z_INDEX = {
  backdrop: 1050,
  content: 1060,
} as const

export const MODAL_SIZE_CLASSES = {
  sm: 'max-w-sm',
  md: 'max-w-[30rem]',
  lg: 'max-w-2xl',
} as const
