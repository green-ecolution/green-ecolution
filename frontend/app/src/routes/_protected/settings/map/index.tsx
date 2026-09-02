import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/map/')({
  component: MapSettingsComingSoon,
})

function MapSettingsComingSoon() {
  const { t } = useTranslation('settings')
  return (
    <SettingsComingSoon
      title={t('comingSoon.map.title')}
      description={t('comingSoon.map.description')}
    />
  )
}
