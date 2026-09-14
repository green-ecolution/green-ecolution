import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Badge, ListCard, ListCardDescription, ListCardTitle } from '@green-ecolution/ui'
import type { PluginResponse } from '@/api/backendApi'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { formatLastSeenAt } from './pluginList'

export const PLUGIN_COLUMNS = '1fr 2fr 1.5fr 1.5fr'

interface PluginCardProps {
  plugin: PluginResponse
  organizationName: string
}

const PluginCard = ({ plugin, organizationName }: PluginCardProps) => {
  const { t } = useTranslation('settings')
  const dateLocale = useDateLocale()

  return (
    <ListCard asChild columns={PLUGIN_COLUMNS}>
      <Link
        to="/settings/plugin/$slug"
        params={{ slug: plugin.slug }}
        aria-label={t('plugin.detailAriaLabel', { name: plugin.name })}
      >
        <div>
          <Badge variant={plugin.enabled ? 'success' : 'muted'} size="lg">
            {plugin.enabled ? t('plugin.status.enabled') : t('plugin.status.disabled')}
          </Badge>
        </div>

        <div className="min-w-0">
          <ListCardTitle className="mb-0.5 truncate">{plugin.name}</ListCardTitle>
          <p className="truncate font-mono text-sm text-dark-600">{plugin.slug}</p>
        </div>

        <ListCardDescription className="min-w-0 truncate">
          <span className="lg:sr-only">{t('plugin.columns.organization')}&nbsp;</span>
          {organizationName}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">{t('plugin.columns.lastSeen')}&nbsp;</span>
          {formatLastSeenAt(plugin.lastSeenAt, dateLocale, t)}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default PluginCard
