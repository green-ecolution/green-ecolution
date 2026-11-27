import { useTranslation } from 'react-i18next'
import {
  irrigationJustWateredTtlMaxHours,
  irrigationJustWateredTtlMinHours,
  irrigationWaterDemandMax,
  irrigationWaterDemandMin,
  translateIssue,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import SettingValueField from '@/components/settings/organization/SettingValueField'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { IRRIGATION } from '../../lib/targets'
import type { ModuleSettingsSectionProps } from '@/modules/types'
import { hoursOf, type IrrigationSettingsDraft } from './useIrrigationSettingsDraft'

const IrrigationSettingsSection = ({
  draft: state,
  locked,
}: ModuleSettingsSectionProps<IrrigationSettingsDraft>) => {
  const { t } = useTranslation('irrigation')
  const translate = useIssueTranslator(IRRIGATION)
  const { server, draft, issues } = state
  if (!server || !draft) return null

  const errorOf = (issue: ValidationIssue | null) =>
    issue ? translateIssue(issue, translate) : null
  const usualLiters = server.waterDemand.inheritedValue
  const usualHours = hoursOf(server.justWateredTtlSecs.inheritedValue)

  return (
    <>
      <SettingValueField
        label={t('settings.waterDemandLabel')}
        description={t('settings.waterDemandHint')}
        unit={t('settings.waterDemandUnit')}
        usual={t('settings.liters', { count: usualLiters })}
        usualCount={usualLiters}
        text={draft.waterDemand}
        error={errorOf(issues.waterDemand)}
        disabled={locked}
        min={irrigationWaterDemandMin()}
        max={irrigationWaterDemandMax()}
        step={1}
        onText={(value) => state.setText('waterDemand', value)}
      />
      <SettingValueField
        label={t('settings.justWateredTtlLabel')}
        description={t('settings.justWateredTtlHint')}
        unit={t('settings.justWateredTtlUnit')}
        usual={t('settings.hours', { count: usualHours })}
        usualCount={usualHours}
        text={draft.justWateredTtlHours}
        error={errorOf(issues.justWateredTtlHours)}
        disabled={locked}
        min={irrigationJustWateredTtlMinHours()}
        max={irrigationJustWateredTtlMaxHours()}
        step={1}
        onText={(value) => state.setText('justWateredTtlHours', value)}
      />
    </>
  )
}

export default IrrigationSettingsSection
