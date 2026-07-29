import { createFileRoute } from '@tanstack/react-router'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/organization/')({
  component: () => (
    <SettingsComingSoon
      title="Organisation"
      description="Hier verwaltest du künftig die Struktur deiner Organisation und ihre Untereinheiten."
    />
  ),
})
