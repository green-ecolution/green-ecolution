import { useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from '@green-ecolution/ui'
import { sensorQueries } from '@/api/queries'
import { getDataQualityDetails, qualityReasonLabel } from '@/hooks/details/useDetailsForDataHealth'

interface SensorDataQualitySectionProps {
  sensorId: string
}

const windowSummary = (implausibleRecent: number): string => {
  if (implausibleRecent === 0)
    return 'Die aufgeführten Messwerte liegen länger als sieben Tage zurück.'
  if (implausibleRecent === 1) return 'In den letzten sieben Tagen wurde ein Messwert verworfen.'
  return `In den letzten sieben Tagen wurden ${implausibleRecent} Messwerte verworfen.`
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
      <CardHeader>
        <CardTitle>Datenqualität</CardTitle>
      </CardHeader>
      <CardContent>
        <Alert variant={quality.alert} className="mb-4 flex w-full items-start gap-3">
          <AlertIcon variant={quality.alert} />
          <AlertContent>
            <AlertTitle>{quality.label}</AlertTitle>
            <AlertDescription>
              {quality.description} {windowSummary(data.implausibleRecent)}
            </AlertDescription>
          </AlertContent>
        </Alert>
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
