import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { Link, useNavigate } from '@tanstack/react-router'
import { useQuery } from '@tanstack/react-query'
import { ExternalLink, KeyRound } from 'lucide-react'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Badge,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  FormField,
  SelectField,
  Switch,
  TextareaField,
} from '@green-ecolution/ui'
import type { PluginResponse } from '@/api/backendApi'
import { organizationQueries } from '@/api/queries'
import { usePluginMutations } from '@/hooks/usePluginMutations'
import BackLink from '@/components/general/links/BackLink'
import { Can } from '@/lib/auth/Can'
import { useHasPermission } from '@/lib/auth/useHasPermission'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { intlLocale } from '@/lib/i18n/format'
import PluginPermissionMatrix from './PluginPermissionMatrix'
import PluginKeyDialog from './PluginKeyDialog'
import { usePluginPermissionDraft } from './usePluginPermissionDraft'
import { formatLastSeenAt, organizationNameOf } from './pluginList'
import {
  buildFrontendDto,
  frontendModeOptions,
  validateTarget,
  type FrontendMode,
} from './pluginFrontend'

interface PluginDetailPageProps {
  plugin: PluginResponse
}

const unchangedSet = (draft: ReadonlySet<string>, stored: readonly string[]): boolean => {
  const right = new Set(stored)
  return draft.size === right.size && [...right].every((entry) => draft.has(entry))
}

const PluginDetailPage = ({ plugin }: PluginDetailPageProps) => {
  const { t, i18n } = useTranslation(['settings', 'common'])
  const navigate = useNavigate()
  const dateLocale = useDateLocale()
  const canUpdate = useHasPermission(['plugin:update'])
  const { data: organizations } = useQuery(organizationQueries.list())
  const { updatePlugin, rotatePluginKey, uninstallPlugin } = usePluginMutations()

  const [name, setName] = useState(plugin.name)
  const [description, setDescription] = useState(plugin.description ?? '')
  const [frontendMode, setFrontendMode] = useState<FrontendMode>(
    plugin.frontendMode as FrontendMode,
  )
  const [target, setTarget] = useState(plugin.frontendTarget ?? '')
  const [targetError, setTargetError] = useState<string | null>(null)
  const { permissions, accessPermissions, togglePermission, toggleAccessPermission } =
    usePluginPermissionDraft(plugin.permissions, plugin.requiredPermissions)

  const [issuedKey, setIssuedKey] = useState<string | null>(null)
  const [rotateConfirmOpen, setRotateConfirmOpen] = useState(false)
  const [uninstallConfirmOpen, setUninstallConfirmOpen] = useState(false)

  const organizationName =
    organizationNameOf(plugin.organizationId, organizations ?? []) ?? plugin.organizationId

  // Show the requested state while the round trip runs, so the switch answers
  // the click instead of waiting for the reloaded route data.
  const requestedEnabled = updatePlugin.isPending
    ? updatePlugin.variables?.change.enabled
    : undefined
  const enabled = requestedEnabled ?? plugin.enabled

  const handleSave = () => {
    // Same check the install dialog runs. Without it an empty or http:// target
    // only comes back as the generic save-failed toast, with nothing pointing
    // at the field that caused it.
    const invalidTarget = validateTarget(frontendMode, target, t)
    setTargetError(invalidTarget)
    if (invalidTarget) return

    const trimmedDescription = description.trim()
    // PluginService::update only runs require_superset when the request
    // actually carries a permission set, so that a rename does not demand
    // every right the plugin holds. Echoing both sets back unconditionally
    // defeats that: an admin with plugin:update but without, say, tree:delete
    // could not change the plugin's name. An untouched set is therefore left
    // out of the request entirely.
    updatePlugin.mutate({
      slug: plugin.slug,
      change: {
        name: name.trim(),
        description: trimmedDescription === '' ? null : trimmedDescription,
        frontend: buildFrontendDto(frontendMode, target),
        ...(unchangedSet(permissions, plugin.permissions) ? {} : { permissions: [...permissions] }),
        ...(unchangedSet(accessPermissions, plugin.requiredPermissions)
          ? {}
          : { requiredPermissions: [...accessPermissions] }),
      },
    })
  }

  const handleToggleEnabled = (next: boolean) => {
    updatePlugin.mutate({ slug: plugin.slug, change: { enabled: next } })
  }

  const handleRotate = () => {
    rotatePluginKey.mutate(plugin.slug, {
      onSuccess: (response) => {
        setRotateConfirmOpen(false)
        setIssuedKey(response.key)
      },
    })
  }

  const handleUninstall = () => {
    uninstallPlugin.mutate(plugin.slug, {
      onSuccess: () => {
        setUninstallConfirmOpen(false)
        void navigate({ to: '/settings/plugin' })
      },
    })
  }

  const facts: { label: string; value: string }[] = [
    { label: t('plugin.install.organizationLabel'), value: organizationName },
    {
      label: t('plugin.detail.lastSeenFactLabel'),
      value: formatLastSeenAt(plugin.lastSeenAt, dateLocale, t),
    },
    ...(plugin.createdAt
      ? [
          {
            label: t('plugin.detail.installedFactLabel'),
            value: new Intl.DateTimeFormat(intlLocale(i18n.language), {
              dateStyle: 'medium',
            }).format(plugin.createdAt),
          },
        ]
      : []),
  ]

  return (
    <div className="flex flex-col gap-6">
      <div>
        <BackLink link={{ to: '/settings/plugin' }} label={t('plugin.backLabel')} />

        <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
          <div className="min-w-0">
            <div className="flex flex-wrap items-center gap-3">
              <h2 className="font-lato text-2xl font-bold text-dark break-words">{plugin.name}</h2>
              <Badge variant={enabled ? 'success' : 'muted'}>
                {enabled ? t('plugin.status.enabled') : t('plugin.status.disabled')}
              </Badge>
            </div>
            <p className="mt-1 font-mono text-sm text-dark-600">{plugin.slug}</p>
          </div>

          {plugin.frontendMode === 'external' && plugin.frontendTarget && (
            <Button asChild variant="outline" size="sm" className="sm:shrink-0">
              <Link to="/plugin/$slug" params={{ slug: plugin.slug }}>
                <ExternalLink className="size-4" aria-hidden />
                {t('plugin.detail.openViewButton')}
              </Link>
            </Button>
          )}
        </div>
      </div>

      <Card variant="outlined">
        <CardHeader className="pb-4">
          <CardTitle className="text-base">{t('plugin.detail.operationHeading')}</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-5">
          <Can permission={['plugin:update']}>
            <div className="flex items-start gap-3">
              <Switch
                checked={enabled}
                disabled={updatePlugin.isPending}
                onCheckedChange={handleToggleEnabled}
                aria-label={t('plugin.detail.enableLabel')}
                className="mt-0.5"
              />
              <div>
                <p className="font-medium text-dark">{t('plugin.detail.enableLabel')}</p>
                <p className="text-sm text-dark-600">{t('plugin.detail.enableHint')}</p>
              </div>
            </div>
          </Can>

          <div className="flex flex-wrap items-center justify-between gap-3 rounded-lg bg-dark-50 px-4 py-3">
            <div className="flex items-center gap-3">
              <KeyRound className="size-4 shrink-0 text-dark-500" aria-hidden />
              <div>
                <p className="text-sm font-medium text-dark">
                  {t('plugin.detail.credentialLabel')}
                </p>
                <p className="text-sm text-dark-600">
                  {plugin.hasCredential
                    ? t('plugin.detail.credentialSet')
                    : t('plugin.detail.credentialMissing')}
                </p>
              </div>
            </div>
            <Can permission={['plugin:update']}>
              <Button
                type="button"
                variant="outline"
                size="sm"
                onClick={() => setRotateConfirmOpen(true)}
              >
                {t('plugin.detail.rotateButton')}
              </Button>
            </Can>
          </div>

          <dl className="grid gap-x-8 gap-y-3 sm:grid-cols-3">
            {facts.map((fact) => (
              <div key={fact.label}>
                <dt className="text-sm text-dark-600">{fact.label}</dt>
                <dd className="text-sm font-medium text-dark">{fact.value}</dd>
              </div>
            ))}
          </dl>
        </CardContent>
      </Card>

      <Card variant="outlined">
        <CardHeader className="pb-4">
          <CardTitle className="text-base">{t('plugin.detail.detailsHeading')}</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-5">
          <FormField
            id="plugin-detail-name"
            label={t('plugin.install.nameLabel')}
            value={name}
            onChange={(event) => setName(event.target.value)}
            disabled={!canUpdate}
          />

          <TextareaField
            id="plugin-detail-description"
            label={t('plugin.install.descriptionLabel')}
            value={description}
            onChange={(event) => setDescription(event.target.value)}
            disabled={!canUpdate}
          />

          <SelectField
            id="plugin-detail-frontend-mode"
            label={t('plugin.install.frontendModeLabel')}
            value={frontendMode}
            disabled={!canUpdate}
            onValueChange={(value) => setFrontendMode(value as FrontendMode)}
            className="max-w-sm"
            options={frontendModeOptions(t, plugin.frontendMode)}
          />

          {frontendMode !== 'none' && (
            <FormField
              id="plugin-detail-target"
              label={t('plugin.install.targetLabel')}
              value={target}
              onChange={(event) => {
                setTarget(event.target.value)
                setTargetError(null)
              }}
              error={targetError ?? undefined}
              disabled={!canUpdate}
              className="max-w-sm"
            />
          )}
        </CardContent>
      </Card>

      <Card variant="outlined">
        <CardHeader className="pb-4">
          <CardTitle className="text-base">{t('plugin.detail.permissionsHeading')}</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-6">
          <PluginPermissionMatrix
            heading={t('plugin.install.permissionsHeading')}
            hint={t('plugin.install.permissionsHint')}
            permissions={permissions}
            disabled={!canUpdate}
            onToggle={togglePermission}
          />

          <PluginPermissionMatrix
            heading={t('plugin.install.accessPermissionsHeading')}
            hint={t('plugin.install.accessPermissionsHint')}
            permissions={accessPermissions}
            disabled={!canUpdate}
            onToggle={toggleAccessPermission}
          />
        </CardContent>
      </Card>

      <div className="flex flex-wrap items-center justify-between gap-3 border-t border-dark-200 pt-4">
        <Can permission={['plugin:delete']}>
          <Button
            type="button"
            variant="ghost-destructive"
            onClick={() => setUninstallConfirmOpen(true)}
          >
            {t('plugin.detail.uninstallButton')}
          </Button>
        </Can>

        <Can permission={['plugin:update']}>
          <Button
            type="button"
            className="ml-auto"
            onClick={handleSave}
            disabled={updatePlugin.isPending}
          >
            {t('plugin.detail.saveButton')}
          </Button>
        </Can>
      </div>

      <AlertDialog open={rotateConfirmOpen} onOpenChange={setRotateConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('plugin.detail.rotateConfirmTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('plugin.detail.rotateConfirmDescription')}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common:actions.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleRotate}>
              {t('plugin.detail.rotateButton')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={uninstallConfirmOpen} onOpenChange={setUninstallConfirmOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t('plugin.detail.uninstallConfirmTitle')}</AlertDialogTitle>
            <AlertDialogDescription>
              {t('plugin.detail.uninstallConfirmDescription', { name: plugin.name })}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t('common:actions.cancel')}</AlertDialogCancel>
            <AlertDialogAction onClick={handleUninstall}>
              {t('plugin.detail.uninstallButton')}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <PluginKeyDialog
        open={issuedKey !== null}
        variant="rotated"
        apiKey={issuedKey ?? ''}
        onOpenChange={(open) => {
          if (!open) setIssuedKey(null)
        }}
      />
    </div>
  )
}

export default PluginDetailPage
