import { format } from 'date-fns'
import { de } from 'date-fns/locale'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CopyableText,
  Separator,
} from '@green-ecolution/ui'
import { Fingerprint } from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import SecretReveal from './SecretReveal'

interface SensorIdentitySectionProps {
  sensor: Sensor
}

const formatDate = (iso: string | undefined): string => {
  if (!iso) return '—'
  try {
    return format(new Date(iso), "dd. MMMM yyyy 'um' HH:mm", { locale: de })
  } catch {
    return iso
  }
}

const SensorIdentitySection = ({ sensor }: SensorIdentitySectionProps) => {
  const lora = sensor.lorawan

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            <Fingerprint className="size-5" />
          </div>
          <CardTitle>Identität</CardTitle>
        </div>
      </CardHeader>
      <CardContent className="grid gap-6 md:grid-cols-2">
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">Modell</span>
          <span className="font-lato font-semibold text-xl text-foreground">
            {sensor.model.name}
          </span>
          <span className="text-xs text-muted-foreground">Modell-ID: {sensor.model.id}</span>
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">Provider</span>
          <span className="font-lato font-semibold text-xl text-foreground">
            {sensor.provider ?? '—'}
          </span>
          <span className="text-xs text-muted-foreground">Datenquelle / Integration</span>
        </div>

        {lora && (
          <>
            <Separator className="md:col-span-2 bg-dark-100" />
            <CopyableText label="Serial Number" value={lora.serialNumber} />
            <CopyableText label="Dev EUI" value={lora.devEui} />
            <CopyableText label="App EUI" value={lora.appEui} />
            {lora.atPin && <SecretReveal label="AT-Pin" value={lora.atPin} />}
            {lora.otaPin && <SecretReveal label="OTA-Pin" value={lora.otaPin} />}
          </>
        )}

        <Separator className="md:col-span-2 bg-dark-100" />
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            Angelegt am
          </span>
          <span className="text-sm font-medium">{formatDate(sensor.createdAt)}</span>
        </div>
        <div className="flex flex-col gap-1.5">
          <span className="text-xs uppercase tracking-widest text-muted-foreground">
            Zuletzt aktualisiert
          </span>
          <span className="text-sm font-medium">{formatDate(sensor.updatedAt)}</span>
        </div>
      </CardContent>
    </Card>
  )
}

export default SensorIdentitySection
