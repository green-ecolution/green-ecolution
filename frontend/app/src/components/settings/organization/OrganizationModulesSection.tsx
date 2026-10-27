import { useId } from 'react'
import { Lock } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  Badge,
  Button,
  Label,
  Switch,
  cn,
} from '@green-ecolution/ui'
import { SettingOriginDto } from '@/api/backendApi'
import { moduleManifests } from '@/modules/registry'
import { CARD, CARD_TITLE } from './OrganizationDetail'
import { originLabel } from './settingOrigin'
import type { ModuleActivationDraft, ModuleActivationRow } from './useModuleActivationDraft'

interface OrganizationModulesSectionProps {
  draft: ModuleActivationDraft
  canUpdate: boolean
  organizationName: (organizationId: string | null | undefined) => string | null
}

interface ModuleRowProps {
  row: ModuleActivationRow
  label: string
  sourceName: string | null
  locked: boolean
  onOwn: (value: boolean | null) => void
}

const ModuleRow = ({ row, label, sourceName, locked, onOwn }: ModuleRowProps) => {
  const { t } = useTranslation('settings')
  const switchId = useId()

  return (
    <li
      className={cn(
        'flex flex-wrap items-center justify-between gap-x-6 gap-y-2 border-l-2 pl-4',
        row.origin === SettingOriginDto.Own ? 'border-green-dark' : 'border-dark-200',
      )}
    >
      <div className="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
        <Label htmlFor={switchId}>{label}</Label>
        <Badge variant={row.origin === SettingOriginDto.Own ? 'outline-green-dark' : 'muted'}>
          {originLabel(row.origin, sourceName, t)}
        </Badge>
      </div>
      <div className="flex items-center gap-4">
        {!locked && row.own !== null && (
          <Button
            type="button"
            variant="link"
            size="sm"
            className="h-auto px-0 text-sm"
            aria-label={t('organization.settings.inheritAgainFor', { field: label })}
            onClick={() => onOwn(null)}
          >
            {t('organization.settings.inheritAgain')}
          </Button>
        )}
        <Switch
          id={switchId}
          checked={row.enabled}
          disabled={locked}
          onCheckedChange={(checked) => onOwn(checked)}
        />
      </div>
    </li>
  )
}

const OrganizationModulesSection = ({
  draft,
  canUpdate,
  organizationName,
}: OrganizationModulesSectionProps) => {
  const { t, i18n } = useTranslation('settings')
  const enforced = draft.enforcedBy !== null
  const enforcedByName = organizationName(draft.enforcedBy?.id)

  const labelOf = (key: string): string => {
    const manifest = moduleManifests.find((candidate) => candidate.key === key)
    return manifest ? i18n.t(manifest.labelKey) : key
  }

  return (
    <section className={CARD}>
      <h3 className={CARD_TITLE}>{t('organization.modules.title')}</h3>
      <p className="mt-1 max-w-[70ch] text-sm text-dark-600">{t('organization.modules.hint')}</p>

      {enforced ? (
        <Alert variant="info" size="default" className="mt-4 flex w-full gap-4" role="alert">
          <AlertIcon variant="info" icon={Lock} />
          <AlertContent>
            <AlertDescription>
              {enforcedByName
                ? t('organization.settings.enforcedNotice', { name: enforcedByName })
                : t('organization.settings.enforcedNoticeUnknown')}
            </AlertDescription>
          </AlertContent>
        </Alert>
      ) : (
        !canUpdate && (
          <p className="mt-2 text-sm text-dark-600">{t('organization.settings.readOnlyNotice')}</p>
        )
      )}

      <ul className="mt-5 flex flex-col gap-4">
        {draft.rows.map((row) => (
          <ModuleRow
            key={row.key}
            row={row}
            label={labelOf(row.key)}
            sourceName={organizationName(row.sourceId)}
            locked={row.locked || !canUpdate}
            onOwn={(value) => draft.setOwn(row.key, value)}
          />
        ))}
      </ul>
    </section>
  )
}

export default OrganizationModulesSection
