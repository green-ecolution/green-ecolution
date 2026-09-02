import { createFileRoute } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'

export const Route = createFileRoute('/_protected/settings/plugin/$pluginName/')({
  component: PluginView,
})

function PluginView() {
  const { pluginName } = Route.useParams()
  const { t } = useTranslation('settings')

  return (
    <div className="container mt-6">
      <p>{t('plugin.unavailable', { pluginName })}</p>
    </div>
  )
}
