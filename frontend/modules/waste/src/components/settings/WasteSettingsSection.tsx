import { useTranslation } from 'react-i18next'
import {
  translateIssue,
  wasteFullThresholdMax,
  wasteFullThresholdMin,
} from '@green-ecolution/domain-wasm'
import SettingValueField from '@/components/settings/organization/SettingValueField'
import { useIssueTranslator } from '@/lib/i18n/validation'
import type { ModuleSettingsSectionProps } from '@/modules/types'
import type { WasteSettingsDraft } from './useWasteSettingsDraft'

const WasteSettingsSection = ({
  draft: state,
  locked,
}: ModuleSettingsSectionProps<WasteSettingsDraft>) => {
  const { t } = useTranslation('waste')
  const translate = useIssueTranslator()
  const { server, draft, issue } = state
  if (!server || !draft) return null
  const usual = server.fullThreshold.inheritedValue

  return (
    <SettingValueField
      label={t('settings.fullThresholdLabel')}
      description={t('settings.fullThresholdHint')}
      unit={t('settings.fullThresholdUnit')}
      usual={t('settings.percent', { count: usual })}
      usualCount={usual}
      text={draft.text}
      error={issue ? translateIssue(issue, translate) : null}
      disabled={locked}
      min={wasteFullThresholdMin()}
      max={wasteFullThresholdMax()}
      step={1}
      onText={state.setText}
    />
  )
}

export default WasteSettingsSection
