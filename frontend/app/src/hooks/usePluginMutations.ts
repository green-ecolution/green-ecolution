import { useMutation } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import type { PluginCreateRequest, PluginUpdateRequest } from '@/api/backendApi'
import { pluginApi } from '@/api/backendApi'
import createToast from '@/hooks/createToast'
import { useInvalidateAggregates } from '@/lib/queryInvalidation'

export interface UpdatePluginVariables {
  slug: string
  change: PluginUpdateRequest
}

export const usePluginMutations = () => {
  const invalidate = useInvalidateAggregates()
  const showToast = createToast()
  const { t } = useTranslation('settings')

  // The detail page reads its plugin from entityRoute loader data, which no
  // query subscription reaches — without reloadRoutes an edit only shows after
  // a full page reload. Uninstall is the exception: it navigates away, and
  // re-running the loader of the deleted plugin renders EntityNotFound first.
  const refresh = (reloadRoutes = true) => invalidate(['plugin'], { reloadRoutes })

  const installPlugin = useMutation({
    mutationFn: (pluginCreateRequest: PluginCreateRequest) =>
      pluginApi.installPlugin({ pluginCreateRequest }),
    onSuccess: () => {
      void refresh(false)
      showToast(t('plugin.detail.toast.installed'))
    },
    onError: () => showToast(t('plugin.detail.toast.installFailed'), 'error'),
  })

  const updatePlugin = useMutation({
    mutationFn: ({ slug, change }: UpdatePluginVariables) =>
      pluginApi.updatePlugin({ pluginSlug: slug, pluginUpdateRequest: change }),
    onSuccess: () => {
      void refresh()
      showToast(t('plugin.detail.toast.saved'))
    },
    onError: () => showToast(t('plugin.detail.toast.saveFailed'), 'error'),
  })

  const rotatePluginKey = useMutation({
    mutationFn: (slug: string) => pluginApi.rotatePluginKey({ pluginSlug: slug }),
    onSuccess: () => {
      void refresh()
      showToast(t('plugin.detail.toast.rotated'))
    },
    onError: () => showToast(t('plugin.detail.toast.rotateFailed'), 'error'),
  })

  const uninstallPlugin = useMutation({
    mutationFn: (slug: string) => pluginApi.uninstallPlugin({ pluginSlug: slug }),
    onSuccess: () => {
      void refresh(false)
      showToast(t('plugin.detail.toast.uninstalled'))
    },
    onError: () => showToast(t('plugin.detail.toast.uninstallFailed'), 'error'),
  })

  return { installPlugin, updatePlugin, rotatePluginKey, uninstallPlugin }
}
