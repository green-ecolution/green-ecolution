import { Suspense } from 'react'
import {
  Badge,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  SignalBars,
  cn,
  type BadgeProps,
} from '@green-ecolution/ui'
import { Signal } from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  SIGNAL_LEVEL_LABEL,
  SIGNAL_LEVEL_TEXT_COLOR,
  type SignalLevel,
} from './signalParsing'
import ChartSignalData from './ChartSignalData'

const LEVEL_BADGE_VARIANT: Record<SignalLevel, BadgeProps['variant']> = {
  good: 'success',
  fair: 'warning',
  weak: 'error',
}

interface SensorSignalCardProps {
  sensor: Sensor
}

const Stat = ({ label, value }: { label: string; value: string }) => (
  <div className="flex flex-col rounded-xl bg-dark-50 px-4 py-2.5">
    <dt className="text-xs uppercase tracking-wide text-muted-foreground">{label}</dt>
    <dd className="text-base font-bold tabular-nums">{value}</dd>
  </div>
)

const SensorSignalCard = ({ sensor }: SensorSignalCardProps) => {
  const signal = parseSignal(sensor.latestData)
  const level = signal ? signalLevelFromRssi(signal.rssiDbm) : null

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
              <Signal className="size-5" />
            </div>
            <CardTitle>Signal</CardTitle>
          </div>
          {level && <Badge variant={LEVEL_BADGE_VARIANT[level]}>{SIGNAL_LEVEL_LABEL[level]}</Badge>}
        </div>
      </CardHeader>
      <CardContent>
        {!signal || !level ? (
          <p className="text-muted-foreground">Keine Signaldaten empfangen.</p>
        ) : (
          <>
            <div className="flex flex-wrap items-end gap-x-6 gap-y-4">
              <div className="flex items-end gap-4">
                <SignalBars
                  filled={signalBarsFromRssi(signal.rssiDbm)}
                  className={cn('h-12', SIGNAL_LEVEL_TEXT_COLOR[level])}
                />
                <p className="text-4xl font-bold leading-none text-green-dark tabular-nums">
                  {signal.rssiDbm}
                  <span className="ml-1.5 text-base font-semibold text-muted-foreground">dBm</span>
                </p>
              </div>
              <dl className="ml-auto flex gap-3">
                <Stat label="SNR" value={`${signal.snrDb} dB`} />
                <Stat label="Gateways" value={String(signal.gatewayCount)} />
              </dl>
            </div>
            <Suspense fallback={null}>
              <ChartSignalData sensorId={sensor.id} />
            </Suspense>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorSignalCard
