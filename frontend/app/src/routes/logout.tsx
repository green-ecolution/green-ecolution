import { getAuthSession } from '@/lib/auth/session'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/logout')({
  beforeLoad: async ({ preload }) => {
    // A hover-triggered preload must not leave the page; only a real click may.
    if (preload) {
      return
    }
    await getAuthSession().signoutRedirect()
  },
})
