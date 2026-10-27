import { useTranslation } from 'react-i18next'
import {
  irrigationJustWateredTtlMaxHours,
  irrigationJustWateredTtlMinHours,
  irrigationWaterDemandMax,
  irrigationWaterDemandMin,
  translateIssue,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import SettingRow from '@/components/settings/organization/SettingRow'
import { useIssueTranslator } from '@/lib/i18n/validation'
import type { ModuleSettingsSectionProps } from '@/modules/types'
import { hoursOf, type IrrigationSettingsDraft } from './useIrrigationSettingsDraft'

const IrrigationSettingsSection = ({
  draft: state,
  locked,
  enforced,
  organizationName,
}: ModuleSettingsSectionProps<IrrigationSettingsDraft>) => {
  const { t } = useTranslation('irrigation')
  const translate = useIssueTranslator()
  const { server, draft, issues } = state
  if (!server || !draft) return null

  const errorOf = (issue: ValidationIssue | null) =>
    issue ? translateIssue(issue, translate) : null
  const liters = (value: number) => t('settings.liters', { count: value })
  const hours = (seconds: number) => t('settings.hours', { count: hoursOf(seconds) })
  const waterDemandOwnValue = server.waterDemand.ownValue
  const ttlOwnValue = server.justWateredTtlSecs.ownValue

  return (
    <div className="mt-5 grid gap-6 @min-[48rem]:grid-cols-2">
      <SettingRow
        label={t('settings.waterDemandLabel')}
        description={t('settings.waterDemandHint')}
        unit={t('settings.waterDemandUnit')}
        effective={liters(server.waterDemand.value)}
        origin={server.waterDemand.origin}
        sourceName={organizationName(server.waterDemand.source?.id)}
        dormantValue={enforced && waterDemandOwnValue != null ? liters(waterDemandOwnValue) : null}
        lastChange={server.waterDemand.lastChange}
        text={draft.waterDemand.text}
        own={draft.waterDemand.own}
        error={errorOf(issues.waterDemand)}
        locked={locked}
        min={irrigationWaterDemandMin()}
        max={irrigationWaterDemandMax()}
        step={1}
        onOwn={(own) => state.setOwn('waterDemand', own)}
        onText={(value) => state.setText('waterDemand', value)}
      />
      <SettingRow
        label={t('settings.justWateredTtlLabel')}
        description={t('settings.justWateredTtlHint')}
        unit={t('settings.justWateredTtlUnit')}
        effective={hours(server.justWateredTtlSecs.value)}
        origin={server.justWateredTtlSecs.origin}
        sourceName={organizationName(server.justWateredTtlSecs.source?.id)}
        dormantValue={enforced && ttlOwnValue != null ? hours(ttlOwnValue) : null}
        lastChange={server.justWateredTtlSecs.lastChange}
        text={draft.justWateredTtlHours.text}
        own={draft.justWateredTtlHours.own}
        error={errorOf(issues.justWateredTtlHours)}
        locked={locked}
        min={irrigationJustWateredTtlMinHours()}
        max={irrigationJustWateredTtlMaxHours()}
        step={1}
        onOwn={(own) => state.setOwn('justWateredTtlHours', own)}
        onText={(value) => state.setText('justWateredTtlHours', value)}
      />
    </div>
  )
}

export default IrrigationSettingsSection
