import { useEffect, useRef, RefObject, useCallback } from 'react'

const FOCUSABLE_SELECTOR = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(', ')

export interface UseFocusTrapOptions {
  enabled?: boolean
  returnFocusOnDeactivate?: boolean
}

export function useFocusTrap<T extends HTMLElement = HTMLElement>(
  options: UseFocusTrapOptions = {},
): RefObject<T | null> {
  const { enabled = true, returnFocusOnDeactivate = true } = options
  const containerRef = useRef<T | null>(null)
  const previousActiveElement = useRef<HTMLElement | null>(null)

  const getFocusableElements = useCallback((): HTMLElement[] => {
    if (!containerRef.current) return []
    return Array.from(
      containerRef.current.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR),
    ).filter((el) => el.offsetParent !== null)
  }, [])

  useEffect(() => {
    if (!enabled) return

    previousActiveElement.current = document.activeElement as HTMLElement

    const focusableElements = getFocusableElements()
    if (focusableElements.length > 0) {
      focusableElements[0].focus()
    } else {
      containerRef.current?.focus()
    }

    return () => {
      if (returnFocusOnDeactivate && previousActiveElement.current) {
        previousActiveElement.current.focus()
      }
    }
  }, [enabled, returnFocusOnDeactivate, getFocusableElements])

  useEffect(() => {
    if (!enabled) return

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return

      const focusableElements = getFocusableElements()
      if (focusableElements.length === 0) return

      const firstElement = focusableElements[0]
      const lastElement = focusableElements[focusableElements.length - 1]

      if (event.shiftKey) {
        if (document.activeElement === firstElement) {
          event.preventDefault()
          lastElement.focus()
        }
      } else {
        if (document.activeElement === lastElement) {
          event.preventDefault()
          firstElement.focus()
        }
      }
    }

    document.addEventListener('keydown', handleKeyDown)
    return () => document.removeEventListener('keydown', handleKeyDown)
  }, [enabled, getFocusableElements])

  return containerRef
}

export default useFocusTrap
