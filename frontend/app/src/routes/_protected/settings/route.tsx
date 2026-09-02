import { createFileRoute, Outlet } from '@tanstack/react-router'
import SettingsLayout from '@/components/settings/SettingsLayout'

export const Route = createFileRoute('/_protected/settings')({
  component: () => (
    <SettingsLayout>
      <Outlet />
    </SettingsLayout>
  ),
  loader: () => ({ crumb: { titleKey: 'settings' as const } }),
})
