import { describe, it, expect, vi, beforeEach, type Mock } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useFormNavigationBlocker } from './useFormNavigationBlocker'
import { useBlocker } from '@tanstack/react-router'

vi.mock('@tanstack/react-router', () => ({
  useBlocker: vi.fn(),
}))

type ShouldBlockFn = (args: { next: { pathname: string } }) => boolean

describe('useFormNavigationBlocker', () => {
  const mockProceed = vi.fn()
  const mockReset = vi.fn()
  let capturedShouldBlockFn: ShouldBlockFn

  beforeEach(() => {
    vi.clearAllMocks()
    ;(useBlocker as Mock).mockImplementation(
      ({ shouldBlockFn }: { shouldBlockFn: ShouldBlockFn }) => {
        capturedShouldBlockFn = shouldBlockFn
        return {
          proceed: mockProceed,
          reset: mockReset,
          status: 'idle',
        }
      },
    )
  })

  describe('shouldBlockFn logic', () => {
    it('does not block when isDirty is false', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: false,
          message: 'Test message',
        }),
      )

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/some/path' } })
      expect(shouldBlock).toBe(false)
    })

    it('blocks when isDirty is true and path is not allowed', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/some/path' } })
      expect(shouldBlock).toBe(true)
    })

    it('does not block when navigating to allowed path', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          allowedPaths: ['/map/tree/edit'],
          message: 'Test message',
        }),
      )

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/map/tree/edit' } })
      expect(shouldBlock).toBe(false)
    })

    it('does not block when path starts with allowed path', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          allowedPaths: ['/map/tree'],
          message: 'Test message',
        }),
      )

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/map/tree/edit/123' } })
      expect(shouldBlock).toBe(false)
    })

    it('blocks when path does not match any allowed paths', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          allowedPaths: ['/map/tree/edit', '/map/cluster/select'],
          message: 'Test message',
        }),
      )

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/trees' } })
      expect(shouldBlock).toBe(true)
    })

    it('does not block after allowNavigation is called', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      act(() => {
        result.current.allowNavigation()
      })

      const shouldBlock = capturedShouldBlockFn({ next: { pathname: '/some/path' } })
      expect(shouldBlock).toBe(false)
    })

    it('resets allowNavigation flag after one navigation', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      act(() => {
        result.current.allowNavigation()
      })

      // First navigation should not block
      let shouldBlock = capturedShouldBlockFn({ next: { pathname: '/some/path' } })
      expect(shouldBlock).toBe(false)

      // Second navigation should block again
      shouldBlock = capturedShouldBlockFn({ next: { pathname: '/some/path' } })
      expect(shouldBlock).toBe(true)
    })
  })

  describe('isModalOpen', () => {
    it('returns false when status is idle', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(result.current.isModalOpen).toBe(false)
    })

    it('returns true when status is blocked', () => {
      ;(useBlocker as Mock).mockImplementation(
        ({ shouldBlockFn }: { shouldBlockFn: ShouldBlockFn }) => {
          capturedShouldBlockFn = shouldBlockFn
          return {
            proceed: mockProceed,
            reset: mockReset,
            status: 'blocked',
          }
        },
      )

      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(result.current.isModalOpen).toBe(true)
    })
  })

  describe('closeModal', () => {
    it('calls reset when closeModal is invoked', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      act(() => {
        result.current.closeModal()
      })

      expect(mockReset).toHaveBeenCalledTimes(1)
    })

    it('handles undefined reset gracefully', () => {
      ;(useBlocker as Mock).mockImplementation(
        ({ shouldBlockFn }: { shouldBlockFn: ShouldBlockFn }) => {
          capturedShouldBlockFn = shouldBlockFn
          return {
            proceed: mockProceed,
            reset: undefined,
            status: 'idle',
          }
        },
      )

      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(() => {
        act(() => {
          result.current.closeModal()
        })
      }).not.toThrow()
    })
  })

  describe('confirmLeave', () => {
    it('calls proceed when confirmLeave is invoked', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      act(() => {
        result.current.confirmLeave()
      })

      expect(mockProceed).toHaveBeenCalledTimes(1)
    })

    it('calls onLeave callback before proceed', () => {
      const onLeave = vi.fn()
      const callOrder: string[] = []

      mockProceed.mockImplementation(() => callOrder.push('proceed'))
      onLeave.mockImplementation(() => callOrder.push('onLeave'))

      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
          onLeave,
        }),
      )

      act(() => {
        result.current.confirmLeave()
      })

      expect(onLeave).toHaveBeenCalledTimes(1)
      expect(mockProceed).toHaveBeenCalledTimes(1)
      expect(callOrder).toEqual(['onLeave', 'proceed'])
    })

    it('handles undefined proceed gracefully', () => {
      ;(useBlocker as Mock).mockImplementation(
        ({ shouldBlockFn }: { shouldBlockFn: ShouldBlockFn }) => {
          capturedShouldBlockFn = shouldBlockFn
          return {
            proceed: undefined,
            reset: mockReset,
            status: 'idle',
          }
        },
      )

      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(() => {
        act(() => {
          result.current.confirmLeave()
        })
      }).not.toThrow()
    })

    it('handles undefined onLeave gracefully', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(() => {
        act(() => {
          result.current.confirmLeave()
        })
      }).not.toThrow()

      expect(mockProceed).toHaveBeenCalledTimes(1)
    })
  })

  describe('message', () => {
    it('returns the provided message', () => {
      const { result } = renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Custom warning message',
        }),
      )

      expect(result.current.message).toBe('Custom warning message')
    })
  })

  describe('useBlocker configuration', () => {
    it('calls useBlocker with withResolver true', () => {
      renderHook(() =>
        useFormNavigationBlocker({
          isDirty: true,
          message: 'Test message',
        }),
      )

      expect(useBlocker).toHaveBeenCalledWith(
        expect.objectContaining({
          withResolver: true,
        }),
      )
    })
  })
})
