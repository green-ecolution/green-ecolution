import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Plus, Puzzle } from 'lucide-react'
import { Button, ListCardHeader, Loading } from '@green-ecolution/ui'
import { pluginsQuery } from '@/api/queries'
import { usePluginMutations } from '@/hooks/usePluginMutations'
import EntityList from '@/components/general/EntityList'
import PluginInstallDialog, {
  type PluginInstallPayload,
} from '@/components/settings/plugin/PluginInstallDialog'
import PluginKeyDialog from '@/components/settings/plugin/PluginKeyDialog'
import PluginCard, { PLUGIN_COLUMNS } from '@/components/settings/plugin/PluginCard'
import { organizationNameOf } from '@/components/settings/plugin/pluginList'
import { buildTree, flatten } from '../organization/organizationTree'
import { useSettingsOrganization } from '../organization/useSettingsOrganization'

const PluginsPage = () => {
  const { t } = useTranslation(['settings', 'common'])
  const { orgId, organizations, can } = useSettingsOrganization()
  const { data: pluginList, isPending } = useQuery(pluginsQuery())
  const { installPlugin } = usePluginMutations()

  const [installOpen, setInstallOpen] = useState(false)
  const [issuedKey, setIssuedKey] = useState<string | null>(null)

  const handleInstall = (payload: PluginInstallPayload) => {
    installPlugin.mutate(
      {
        slug: payload.slug,
        name: payload.name,
        description: payload.description,
        organizationId: payload.organizationId,
        permissions: payload.permissions,
        requiredPermissions: payload.requiredPermissions,
        deviceCapabilities: payload.deviceCapabilities,
        frontend: payload.frontend,
      },
      {
        onSuccess: (response) => {
          setInstallOpen(false)
          setIssuedKey(response.key)
        },
      },
    )
  }

  const subtree = buildTree(organizations, orgId)
  const listedOrgIds = new Set(subtree ? flatten(subtree).map((org) => org.id) : [orgId])
  const plugins = (pluginList ?? []).filter((plugin) => listedOrgIds.has(plugin.organizationId))

  return (
    <div className="flex flex-col gap-8">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <article className="flex-1">
          <h2 className="font-lato text-xl font-bold">{t('plugin.overviewTitle')}</h2>
          <p className="mt-2 max-w-prose text-sm text-dark-600">{t('plugin.overviewIntro')}</p>
        </article>
        {can(['plugin:create']) && (
          <Button
            type="button"
            size="sm"
            className="w-full sm:w-auto sm:shrink-0"
            onClick={() => setInstallOpen(true)}
          >
            <Plus className="size-4" aria-hidden />
            {t('plugin.installButton')}
          </Button>
        )}
      </div>

      {isPending ? (
        <Loading className="mt-10 justify-center" label={t('plugin.loading')} />
      ) : plugins.length === 0 ? (
        <div className="rounded-xl border border-dashed border-dark-200 bg-white p-8 text-center">
          <Puzzle className="mx-auto size-8 text-dark-400" aria-hidden />
          <p className="mx-auto mt-4 max-w-prose text-sm text-dark-600">{t('plugin.empty')}</p>
        </div>
      ) : (
        <section>
          <ListCardHeader columns={PLUGIN_COLUMNS}>
            <p>{t('plugin.columns.status')}</p>
            <p>{t('plugin.columns.name')}</p>
            <p>{t('plugin.columns.organization')}</p>
            <p>{t('plugin.columns.lastSeen')}</p>
          </ListCardHeader>

          <EntityList
            items={plugins}
            getKey={(plugin) => plugin.slug}
            emptyMessage={t('plugin.empty')}
            renderItem={(plugin) => (
              <PluginCard
                plugin={plugin}
                organizationName={
                  organizationNameOf(plugin.organizationId, organizations) ?? plugin.organizationId
                }
                orgId={orgId}
              />
            )}
          />
        </section>
      )}

      <PluginInstallDialog
        open={installOpen}
        organizationId={orgId}
        saving={installPlugin.isPending}
        onOpenChange={setInstallOpen}
        onSubmit={handleInstall}
      />

      <PluginKeyDialog
        open={issuedKey !== null}
        variant="installed"
        apiKey={issuedKey ?? ''}
        onOpenChange={(open) => {
          if (!open) setIssuedKey(null)
        }}
      />
    </div>
  )
}

export default PluginsPage
