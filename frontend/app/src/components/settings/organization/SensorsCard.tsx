import { Radio } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  defectStreakMax,
  defectStreakMin,
  sensorOfflineAfterMaxHours,
  sensorOfflineAfterMinHours,
  translateIssue,
  type ValidationIssue,
} from '@green-ecolution/domain-wasm'
import { useIssueTranslator } from '@/lib/i18n/validation'
import ModuleAreaCard from './ModuleAreaCard'
import SettingValueField from './SettingValueField'
import { hoursOf, type SensorSettingsDraft } from './useSensorSettingsDraft'

const SensorsCard = ({ draft: state, locked }: { draft: SensorSettingsDraft; locked: boolean }) => {
  const { t } = useTranslation('settings')
  const translate = useIssueTranslator()
  const { server, draft, issues } = state
  if (!server || !draft) return null

  const errorOf = (issue: ValidationIssue | null) =>
    issue ? translateIssue(issue, translate) : null
  const usualHours = hoursOf(server.sensorOfflineAfterSecs.inheritedValue)
  const usualStreak = server.defectStreak.inheritedValue

  return (
    <ModuleAreaCard
      icon={Radio}
      title={t('organization.areas.sensors.title')}
      description={t('organization.areas.sensors.description')}
    >
      <SettingValueField
        label={t('organization.areas.sensors.offlineAfterLabel')}
        description={t('organization.areas.sensors.offlineAfterHint')}
        unit={t('organization.areas.sensors.offlineAfterUnit')}
        usual={t('organization.areas.sensors.hours', { count: usualHours })}
        usualCount={usualHours}
        text={draft.offlineAfterHours}
        error={errorOf(issues.offlineAfterHours)}
        disabled={locked}
        min={sensorOfflineAfterMinHours()}
        max={sensorOfflineAfterMaxHours()}
        step={1}
        onText={(value) => state.setText('offlineAfterHours', value)}
      />
      <SettingValueField
        label={t('organization.areas.sensors.defectStreakLabel')}
        description={t('organization.areas.sensors.defectStreakHint')}
        unit={t('organization.areas.sensors.defectStreakUnit')}
        usual={t('organization.areas.sensors.readings', { count: usualStreak })}
        usualCount={usualStreak}
        text={draft.defectStreak}
        error={errorOf(issues.defectStreak)}
        disabled={locked}
        min={defectStreakMin()}
        max={defectStreakMax()}
        step={1}
        onText={(value) => state.setText('defectStreak', value)}
      />
    </ModuleAreaCard>
  )
}

export default SensorsCard
