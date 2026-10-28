import { useTranslation } from 'react-i18next'
import {
  translateIssue,
  wasteFullThresholdMax,
  wasteFullThresholdMin,
} from '@green-ecolution/domain-wasm'
import SettingRow from '@/components/settings/organization/SettingRow'
import { useIssueTranslator } from '@/lib/i18n/validation'
import type { ModuleSettingsSectionProps } from '@/modules/types'
import type { WasteSettingsDraft } from './useWasteSettingsDraft'

const WasteSettingsSection = ({
  draft: state,
  locked,
  enforced,
  organizationName,
}: ModuleSettingsSectionProps<WasteSettingsDraft>) => {
  const { t } = useTranslation('waste')
  const translate = useIssueTranslator()
  const { server, draft, issue } = state
  if (!server || !draft) return null

  const percent = (value: number) => t('units.percent', { value })
  const field = server.fullThreshold
  const ownValue = field.ownValue

  return (
    <div className="mt-5 grid gap-6 @min-[48rem]:grid-cols-2">
      <SettingRow
        label={t('settings.fullThresholdLabel')}
        description={t('settings.fullThresholdHint')}
        unit={t('settings.fullThresholdUnit')}
        effective={percent(field.value)}
        origin={field.origin}
        sourceName={organizationName(field.source?.id)}
        dormantValue={enforced && ownValue != null ? percent(ownValue) : null}
        lastChange={field.lastChange}
        text={draft.text}
        own={draft.own}
        error={issue ? translateIssue(issue, translate) : null}
        locked={locked}
        min={wasteFullThresholdMin()}
        max={wasteFullThresholdMax()}
        step={1}
        onOwn={state.setOwn}
        onText={state.setText}
      />
    </div>
  )
}

export default WasteSettingsSection
