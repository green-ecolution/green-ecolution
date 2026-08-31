import { getAuthSession } from '@/lib/auth/session'
import { startSignoutHandover } from '@/lib/auth/handover'
import { createFileRoute, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/logout')({
  beforeLoad: async ({ preload }) => {
    // A hover-triggered preload must not leave the page; only a real click may.
    if (preload) {
      return
    }
    await startSignoutHandover(getAuthSession())
    // Still here, so the handover was aborted; this route renders nothing.
    throw redirect({ to: '/', replace: true })
  },
})
