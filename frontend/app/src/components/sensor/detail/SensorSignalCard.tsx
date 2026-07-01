import { Card, CardContent, CardHeader, CardTitle, SignalBars, cn } from '@green-ecolution/ui'
import { Signal } from 'lucide-react'
import type { Sensor } from '@/api/backendApi'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  SIGNAL_LEVEL_LABEL,
  SIGNAL_LEVEL_TEXT_COLOR,
} from './signalParsing'

interface SensorSignalCardProps {
  sensor: Sensor
}

const SensorSignalCard = ({ sensor }: SensorSignalCardProps) => {
  const signal = parseSignal(sensor.latestData)
  const level = signal ? signalLevelFromRssi(signal.rssiDbm) : null

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            <Signal className="size-5" />
          </div>
          <CardTitle>Signal</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        {!signal || !level ? (
          <p className="text-muted-foreground">Keine Signaldaten empfangen.</p>
        ) : (
          <>
            <div className="flex items-end gap-4">
              <SignalBars
                filled={signalBarsFromRssi(signal.rssiDbm)}
                className={cn('h-11', SIGNAL_LEVEL_TEXT_COLOR[level])}
              />
              <div>
                <p className="text-3xl font-bold leading-none text-green-dark">
                  {signal.rssiDbm}
                  <span className="ml-1 text-base font-semibold text-muted-foreground">dBm</span>
                </p>
                <p className="mt-1 text-sm text-muted-foreground">
                  SNR {signal.snrDb} dB · {signal.gatewayCount} Gateway
                  {signal.gatewayCount === 1 ? '' : 's'}
                </p>
              </div>
            </div>
            <span
              className={cn(
                'mt-3 inline-block rounded-full px-3 py-1 text-xs font-semibold',
                SIGNAL_LEVEL_TEXT_COLOR[level],
              )}
            >
              {SIGNAL_LEVEL_LABEL[level]}
            </span>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorSignalCard
