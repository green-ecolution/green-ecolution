import { createFileRoute } from '@tanstack/react-router'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/irrigation/')({
  component: () => (
    <SettingsComingSoon
      title="Bewässerung"
      description="Hier legst du künftig Schwellenwerte und Bewässerungsregeln für die gesamte Anwendung fest."
    />
  ),
})
