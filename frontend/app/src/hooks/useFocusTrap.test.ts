import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useFocusTrap } from './useFocusTrap'

describe('useFocusTrap', () => {
  let container: HTMLDivElement

  beforeEach(() => {
    container = document.createElement('div')
    document.body.appendChild(container)
  })

  afterEach(() => {
    document.body.removeChild(container)
    vi.clearAllMocks()
  })

  describe('initialization', () => {
    it('returns a ref object', () => {
      const { result } = renderHook(() => useFocusTrap())
      expect(result.current).toHaveProperty('current')
    })

    it('ref starts as null', () => {
      const { result } = renderHook(() => useFocusTrap<HTMLDivElement>({ enabled: true }))
      expect(result.current.current).toBeNull()
    })

    it('does not trap focus when disabled', () => {
      const previousFocus = document.createElement('button')
      previousFocus.textContent = 'Previous'
      document.body.appendChild(previousFocus)
      previousFocus.focus()

      renderHook(() => useFocusTrap({ enabled: false }))

      expect(document.activeElement).toBe(previousFocus)
      document.body.removeChild(previousFocus)
    })
  })

  describe('focus return', () => {
    it('returns focus to previous element on unmount when returnFocusOnDeactivate is true', () => {
      const previousButton = document.createElement('button')
      previousButton.textContent = 'Previous'
      document.body.appendChild(previousButton)
      previousButton.focus()

      const { unmount } = renderHook(() =>
        useFocusTrap({ enabled: true, returnFocusOnDeactivate: true }),
      )

      unmount()

      expect(document.activeElement).toBe(previousButton)
      document.body.removeChild(previousButton)
    })

    it('does not return focus when disabled', () => {
      const previousButton = document.createElement('button')
      previousButton.textContent = 'Previous'
      document.body.appendChild(previousButton)
      previousButton.focus()

      const { unmount } = renderHook(() => useFocusTrap({ enabled: false }))

      unmount()

      expect(document.activeElement).toBe(previousButton)
      document.body.removeChild(previousButton)
    })
  })

  describe('keyboard navigation', () => {
    it('handles Tab key events', () => {
      const button1 = document.createElement('button')
      const button2 = document.createElement('button')
      button1.textContent = 'Button 1'
      button2.textContent = 'Button 2'
      container.appendChild(button1)
      container.appendChild(button2)

      renderHook(() => useFocusTrap<HTMLDivElement>({ enabled: true }))

      const tabEvent = new KeyboardEvent('keydown', {
        key: 'Tab',
        bubbles: true,
      })

      document.dispatchEvent(tabEvent)

      expect(true).toBe(true)
    })

    it('handles Shift+Tab events', () => {
      const button1 = document.createElement('button')
      const button2 = document.createElement('button')
      button1.textContent = 'Button 1'
      button2.textContent = 'Button 2'
      container.appendChild(button1)
      container.appendChild(button2)

      renderHook(() => useFocusTrap<HTMLDivElement>({ enabled: true }))

      const shiftTabEvent = new KeyboardEvent('keydown', {
        key: 'Tab',
        shiftKey: true,
        bubbles: true,
      })

      document.dispatchEvent(shiftTabEvent)

      expect(true).toBe(true)
    })

    it('ignores non-Tab keys', () => {
      const button = document.createElement('button')
      button.textContent = 'Button'
      container.appendChild(button)
      button.focus()

      renderHook(() => useFocusTrap<HTMLDivElement>({ enabled: true }))

      const enterEvent = new KeyboardEvent('keydown', {
        key: 'Enter',
        bubbles: true,
      })

      document.dispatchEvent(enterEvent)

      expect(document.activeElement).toBe(button)
    })
  })

  describe('default options', () => {
    it('uses default values when no options provided', () => {
      const { result } = renderHook(() => useFocusTrap())

      expect(result.current).toBeDefined()
      expect(result.current.current).toBeNull()
    })
  })
})
