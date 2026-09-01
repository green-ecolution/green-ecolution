import type { Locale } from 'date-fns'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CopyableText,
  Separator,
} from '@green-ecolution/ui'
import { Fingerprint } from 'lucide-react'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import type { Sensor } from '@/api/backendApi'
import SecretReveal from './SecretReveal'

interface SensorIdentitySectionProps {
  sensor: Sensor
}

const formatDate = (iso: string | undefined, locale: Locale): string => {
  if (!iso) return '—'
  try {
    return format(new Date(iso), "dd. MMMM yyyy 'um' HH:mm", { locale })
  } catch {
    return iso
  }
}

const SensorIdentitySection = ({ sensor }: SensorIdentitySectionProps) => {
  const { t } = useTranslation('sensor')
  const dateLocale = useDateLocale()
  const lora = sensor.lorawan

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            <Fingerprint className="size-5" />
          </div>
          <CardTitle>{t('identity.title')}</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="grid gap-6 md:grid-cols-2">
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            {t('identity.modelLabel')}
          </span>
          <span className="font-lato font-semibold text-xl text-foreground">
            {sensor.model.name}
          </span>
          <span className="text-xs text-muted-foreground">
            {t('identity.modelIdLabel', { id: sensor.model.id })}
          </span>
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            {t('identity.providerLabel')}
          </span>
          <span className="font-lato font-semibold text-xl text-foreground">
            {sensor.provider ?? '—'}
          </span>
          <span className="text-xs text-muted-foreground">{t('identity.providerSourceLabel')}</span>
        </div>

        {lora && (
          <>
            <Separator className="md:col-span-2 bg-dark-100" />
            <CopyableText label={t('identity.serialNumberLabel')} value={lora.serialNumber} />
            <CopyableText label={t('identity.devEuiLabel')} value={lora.devEui} />
            <CopyableText label={t('identity.appEuiLabel')} value={lora.appEui} />
            {lora.atPin && <SecretReveal label={t('identity.atPinLabel')} value={lora.atPin} />}
            {lora.otaPin && <SecretReveal label={t('identity.otaPinLabel')} value={lora.otaPin} />}
          </>
        )}

        <Separator className="md:col-span-2 bg-dark-100" />
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            {t('identity.createdAtLabel')}
          </span>
          <span className="text-sm font-medium">{formatDate(sensor.createdAt, dateLocale)}</span>
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            {t('identity.updatedAtLabel')}
          </span>
          <span className="text-sm font-medium">{formatDate(sensor.updatedAt, dateLocale)}</span>
        </div>
      </CardContent>
    </Card>
  )
}

export default SensorIdentitySection
