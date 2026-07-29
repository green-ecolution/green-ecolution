import { createFileRoute } from '@tanstack/react-router'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/sensors/')({
  component: () => (
    <SettingsComingSoon
      title="Sensoren & Netz"
      description="Hier konfigurierst du künftig die LoRaWAN-Anbindung und die Vorgaben für neue Geräte."
    />
  ),
})
