import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/sensors/')({
  component: SensorsComingSoon,
})

function SensorsComingSoon() {
  const { t } = useTranslation('settings')
  return (
    <SettingsComingSoon
      title={t('comingSoon.sensors.title')}
      description={t('comingSoon.sensors.description')}
    />
  )
}
