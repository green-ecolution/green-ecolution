import { createFileRoute, redirect, type RegisteredRouter } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/settings/')({
  beforeLoad: () => {
    // /settings/profile lands in a later task; widen `to` to string so this compiles before then.
    throw redirect<RegisteredRouter, string>({ to: '/settings/profile' })
  },
})
