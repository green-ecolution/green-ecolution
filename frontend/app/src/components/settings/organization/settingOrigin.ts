import type { TFunction } from 'i18next'
import { SettingOriginDto } from '@/api/backendApi'

export const originLabel = (
  origin: SettingOriginDto,
  sourceName: string | null,
  t: TFunction<'settings'>,
): string => {
  switch (origin) {
    case SettingOriginDto.Own:
      return t('organization.settings.origin.own')
    case SettingOriginDto.Inherited:
      return sourceName
        ? t('organization.settings.origin.inherited', { name: sourceName })
        : t('organization.settings.origin.inheritedUnknown')
    case SettingOriginDto.Default:
      return t('organization.settings.origin.default')
  }
}
