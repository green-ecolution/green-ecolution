import { createFileRoute } from '@tanstack/react-router'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/notifications/')({
  component: () => (
    <SettingsComingSoon
      title="Benachrichtigungen"
      description="Hier bestimmst du künftig, worüber und auf welchem Weg dich die Anwendung informiert."
    />
  ),
})
