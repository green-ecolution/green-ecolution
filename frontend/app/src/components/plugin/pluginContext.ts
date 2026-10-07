import type { PluginContext } from '@green-ecolution/plugin-interface'
import { pluginApi } from '@/api/backendApi'

// Bypasses the query cache on purpose: a cached view response would replay a
// ticket that is already redeemed or expired.
export const pluginContextFactory =
  (base: Omit<PluginContext, 'viewTicket'>): (() => Promise<PluginContext>) =>
  async () => {
    const { viewTicket } = await pluginApi.getPluginView({ pluginSlug: base.plugin.slug })
    return { ...base, viewTicket }
  }
