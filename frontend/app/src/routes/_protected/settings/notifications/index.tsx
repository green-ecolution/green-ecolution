import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import SettingsComingSoon from '@/components/settings/SettingsComingSoon'

export const Route = createFileRoute('/_protected/settings/notifications/')({
  component: NotificationsComingSoon,
})

function NotificationsComingSoon() {
  const { t } = useTranslation('settings')
  return (
    <SettingsComingSoon
      title={t('comingSoon.notifications.title')}
      description={t('comingSoon.notifications.description')}
    />
  )
}
