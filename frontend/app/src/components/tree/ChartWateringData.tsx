import { sensorDataQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Line } from 'recharts'
import { type ChartConfig } from '@green-ecolution/ui'
import TimeSeriesChart from './TimeSeriesChart'

interface Watermark {
  depth: number
  resistance: number
}

const climateConfig = {
  temperature: {
    label: 'Bodentemperatur in Celsius',
    color: '#ACB63B',
  },
  humidity: {
    label: 'Bodenfeuchte in Prozent',
    color: '#4C7741',
  },
} satisfies ChartConfig

const watermarkConfig = {
  depth_30: {
    label: 'Ω in 30cm',
    color: '#8884d8',
  },
  depth_60: {
    label: 'Ω in 60cm',
    color: '#ACB63B',
  },
  depth_90: {
    label: 'Ω in 90cm',
    color: '#E44E4D',
  },
} satisfies ChartConfig

interface ChartWateringDataProps {
  sensorId: string
}

const ChartWateringData: React.FC<ChartWateringDataProps> = ({ sensorId }) => {
  const { data: sensorDataRes } = useSuspenseQuery(sensorDataQuery(sensorId, { perPage: 5000 }))
  const readings = sensorDataRes.data
  const transformedDataForTemperature = readings
    .map((entry) => {
      const payload = entry.data as Record<string, unknown>
      return {
        name: format(new Date(entry.updatedAt), 'dd.MM.yyyy'),
        temperature: payload.temperature,
        humidity: payload.humidity,
      }
    })
    .reverse()

  const transformedDataForWatermarks = readings
    .map((entry) => {
      const payload = entry.data as Record<string, unknown>
      const formattedEntry: Record<string, number | string> = {
        name: format(new Date(entry.updatedAt), 'dd.MM.yyyy'),
      }

      const watermarks = (payload.watermarks as Watermark[]) ?? []
      watermarks.forEach((watermark: Watermark) => {
        formattedEntry[`depth_${watermark.depth}`] = watermark.resistance
      })

      return formattedEntry
    })
    .reverse()

  if (readings.length <= 1) return null

  return (
    <>
      <h3 className="font-bold mb-4 text-dark-600 text-center">
        Bodentemperatur und -feuchtigkeit im Verlaufe der Zeit:
      </h3>
      <TimeSeriesChart config={climateConfig} data={transformedDataForTemperature}>
        <Line
          type="monotone"
          dataKey="temperature"
          stroke="var(--color-temperature)"
          strokeWidth={2}
          dot={{ r: 5 }}
        />
        <Line
          type="monotone"
          dataKey="humidity"
          stroke="var(--color-humidity)"
          strokeWidth={2}
          dot={{ r: 5 }}
        />
      </TimeSeriesChart>
      <h3 className="font-bold mb-4 mt-10 text-dark-600 text-center">
        Wasserspannung in den Tiefen 30cm, 60cm und 90cm im Verlaufe der Zeit:
      </h3>
      <TimeSeriesChart config={watermarkConfig} data={transformedDataForWatermarks}>
        <Line
          type="monotone"
          dataKey="depth_30"
          stroke="var(--color-depth_30)"
          strokeWidth={2}
          dot={{ r: 5 }}
        />
        <Line
          type="monotone"
          dataKey="depth_60"
          stroke="var(--color-depth_60)"
          strokeWidth={2}
          dot={{ r: 5 }}
        />
        <Line
          type="monotone"
          dataKey="depth_90"
          stroke="var(--color-depth_90)"
          strokeWidth={2}
          dot={{ r: 5 }}
        />
      </TimeSeriesChart>
    </>
  )
}

export default ChartWateringData
