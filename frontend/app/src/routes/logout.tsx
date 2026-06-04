import { getAuthSession } from '@/lib/auth/session'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/logout')({
  beforeLoad: async () => {
    await getAuthSession().signoutRedirect()
  },
})
