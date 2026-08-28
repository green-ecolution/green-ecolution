import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Badge, Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import { sensorQueries } from '@/api/queries'
import { getDataQualityDetails, qualityReasonLabel } from '@/hooks/details/useDetailsForDataHealth'

interface SensorDataQualitySectionProps {
  sensorId: string
}

const SensorDataQualitySection = ({ sensorId }: SensorDataQualitySectionProps) => {
  const { data } = useQuery(sensorQueries.dataQuality(sensorId))
  if (!data || data.issues.length === 0) return null

  const quality = getDataQualityDetails({
    dataHealth: data.health,
    implausibleRecent: data.implausibleRecent,
  })

  return (
    <Card variant="outlined">
      <CardHeader className="flex flex-row items-center justify-between gap-3">
        <CardTitle>Datenqualität</CardTitle>
        <Badge variant={quality.color}>{quality.label}</Badge>
      </CardHeader>
      <CardContent>
        <p className="mb-4 text-sm text-muted-foreground">
          {quality.description}{' '}
          {data.implausibleRecent > 0
            ? `In den letzten sieben Tagen wurden ${data.implausibleRecent} Messwerte als unplausibel markiert und von der Auswertung ausgeschlossen.`
            : 'Die aufgeführten Messwerte liegen länger als sieben Tage zurück.'}
        </p>
        <ul className="flex flex-col gap-2">
          {data.issues.map((issue) => (
            <li
              key={`${issue.recordedAt}-${issue.ability}-${issue.depthCm}`}
              className="rounded-lg border border-dark-50 bg-white p-3 text-sm"
            >
              <p className="font-bold">
                {format(new Date(issue.recordedAt), 'dd.MM.yyyy HH:mm')} · {issue.depthCm} cm Tiefe
              </p>
              <p className="text-dark-800">
                Messwert {issue.value} · {qualityReasonLabel(issue.reason)}
              </p>
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  )
}

export default SensorDataQualitySection
