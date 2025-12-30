import { useBlocker } from '@tanstack/react-router'
import { useCallback, useRef } from 'react'

export interface UseFormNavigationBlockerOptions {
  isDirty: boolean
  allowedPaths?: string[]
  onLeave?: () => void
  message: string
}

export interface UseFormNavigationBlockerReturn {
  isModalOpen: boolean
  allowNavigation: () => void
  closeModal: () => void
  confirmLeave: () => void
  message: string
}

export const useFormNavigationBlocker = ({
  isDirty,
  allowedPaths = [],
  onLeave,
  message,
}: UseFormNavigationBlockerOptions): UseFormNavigationBlockerReturn => {
  const allowNavigationRef = useRef(false)

  const { proceed, reset, status } = useBlocker({
    shouldBlockFn: ({ next }) => {
      if (allowNavigationRef.current) {
        allowNavigationRef.current = false
        return false
      }

      if (!isDirty) {
        return false
      }

      const isAllowedPath = allowedPaths.some((path) => next.pathname.startsWith(path))
      if (isAllowedPath) {
        return false
      }

      return true
    },
    withResolver: true,
  })

  const allowNavigation = useCallback(() => {
    allowNavigationRef.current = true
  }, [])

  const closeModal = useCallback(() => {
    reset?.()
  }, [reset])

  const confirmLeave = useCallback(() => {
    onLeave?.()
    proceed?.()
  }, [onLeave, proceed])

  return {
    isModalOpen: status === 'blocked',
    allowNavigation,
    closeModal,
    confirmLeave,
    message,
  }
}
