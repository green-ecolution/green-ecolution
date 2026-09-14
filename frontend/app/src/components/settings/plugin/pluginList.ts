import type { Locale } from 'date-fns'
import { formatDistanceToNow } from 'date-fns'
import type { TFunction } from 'i18next'
import type { OrganizationResponse } from '@/api/backendApi'

export const organizationNameOf = (
  organizationId: string,
  organizations: OrganizationResponse[],
): string | null => organizations.find((org) => org.id === organizationId)?.name ?? null

export const formatLastSeenAt = (
  lastSeenAt: Date | null | undefined,
  locale: Locale,
  t: TFunction<'settings'>,
): string => {
  if (!lastSeenAt) return t('plugin.neverSeen')
  try {
    return formatDistanceToNow(lastSeenAt, { locale, addSuffix: true })
  } catch {
    return t('plugin.neverSeen')
  }
}
