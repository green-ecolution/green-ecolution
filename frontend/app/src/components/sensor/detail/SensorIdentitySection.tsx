import type { Locale } from 'date-fns'
import { format } from 'date-fns'
import type { TFunction } from 'i18next'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { sensorQueries } from '@/api/queries'
import { Card, CardContent, CardHeader, CardTitle, CopyableText, cn } from '@green-ecolution/ui'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import type { Sensor } from '@/api/backendApi'
import SecretReveal from './SecretReveal'
import { compactCardContent, compactCardHeader, compactCardTitle } from './cardDensity'

interface SensorIdentitySectionProps {
  sensor: Sensor
}

const formatDate = (
  iso: string | undefined,
  locale: Locale,
  t: TFunction<['sensor', 'common']>,
): string => {
  if (!iso) return '—'
  try {
    const date = new Date(iso)
    return t('common:dateTime.at', {
      date: format(date, 'dd.MM.yyyy', { locale }),
      time: format(date, 'HH:mm', { locale }),
    })
  } catch {
    return iso
  }
}

// CopyableText and SecretReveal default to a display size meant for a lone value.
const denseValues = '[&>div]:gap-1 [&_code]:min-h-0 [&_code]:py-1.5 [&_code]:text-sm'

const SensorIdentitySection = ({ sensor }: SensorIdentitySectionProps) => {
  const { t } = useTranslation(['sensor', 'common'])
  const dateLocale = useDateLocale()
  const lora = sensor.lorawan
  const { data: model } = useQuery(sensorQueries.model(sensor.model.id))

  return (
    <Card variant="outlined">
      <CardHeader className={compactCardHeader}>
        <CardTitle className={compactCardTitle}>{t('identity.title')}</CardTitle>
      </CardHeader>
      <CardContent className={cn(compactCardContent, 'flex flex-col gap-4')}>
        <div className="text-sm">
          <p className="text-muted-foreground">{t('identity.modelLabel')}</p>
          <p className="font-semibold">{sensor.model.name}</p>
          {model?.description && (
            <p className="mt-0.5 max-w-prose text-muted-foreground">{model.description}</p>
          )}
        </div>
        {lora && (
          <div
            className={cn('grid gap-3 sm:grid-cols-2 xl:grid-cols-3 [&>*]:min-w-0', denseValues)}
          >
            <CopyableText label={t('identity.devEuiLabel')} value={lora.devEui} />
            <CopyableText label={t('identity.appEuiLabel')} value={lora.appEui} />
            <CopyableText label={t('identity.serialNumberLabel')} value={lora.serialNumber} />
            {lora.atPin && <SecretReveal label={t('identity.atPinLabel')} value={lora.atPin} />}
            {lora.otaPin && <SecretReveal label={t('identity.otaPinLabel')} value={lora.otaPin} />}
          </div>
        )}
        <dl className="grid grid-cols-2 gap-x-6 gap-y-1 text-sm sm:max-w-md">
          <div>
            <dt className="text-muted-foreground">{t('identity.createdAtLabel')}</dt>
            <dd className="font-medium">{formatDate(sensor.createdAt, dateLocale, t)}</dd>
          </div>
          <div>
            <dt className="text-muted-foreground">{t('identity.updatedAtLabel')}</dt>
            <dd className="font-medium">{formatDate(sensor.updatedAt, dateLocale, t)}</dd>
          </div>
        </dl>
      </CardContent>
    </Card>
  )
}

export default SensorIdentitySection
