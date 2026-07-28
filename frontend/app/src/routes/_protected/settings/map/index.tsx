import { createFileRoute } from '@tanstack/react-router'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/map/')({
  component: () => (
    <SettingsComingSoon
      title="Karte & Einheiten"
      description="Hier wählst du künftig Kartenhintergrund, Startausschnitt und die Einheiten für Längen und Mengen."
    />
  ),
})
