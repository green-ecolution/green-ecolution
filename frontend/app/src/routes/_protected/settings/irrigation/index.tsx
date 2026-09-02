import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/irrigation/')({
  component: IrrigationComingSoon,
})

function IrrigationComingSoon() {
  const { t } = useTranslation('settings')
  return (
    <SettingsComingSoon
      title={t('comingSoon.irrigation.title')}
      description={t('comingSoon.irrigation.description')}
    />
  )
}
