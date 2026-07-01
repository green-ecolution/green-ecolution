import { Badge, SignalBars } from '@green-ecolution/ui'
import type { TreeCluster } from '@/api/backendApi'
import {
  signalBarsFromRssi,
  signalLevelFromRssi,
  SIGNAL_LEVEL_LABEL,
  SIGNAL_LEVEL_TEXT_COLOR,
  SIGNAL_LEVEL_BADGE_VARIANT,
} from '@/components/sensor/detail/signalParsing'
import { clusterSignalSummary, type ClusterSignalSummary } from './clusterSignalSummary'

interface ClusterSignalCardProps {
  treecluster: TreeCluster
}

const SingleReading = ({ rssiDbm, treeNumber }: { rssiDbm: number; treeNumber: string }) => {
  const level = signalLevelFromRssi(rssiDbm)
  return (
    <div className="mt-3 flex flex-wrap items-center gap-x-4 gap-y-2">
      <SignalBars
        filled={signalBarsFromRssi(rssiDbm)}
        size="md"
        className={SIGNAL_LEVEL_TEXT_COLOR[level]}
      />
      <p className="text-2xl font-bold leading-none text-green-dark tabular-nums">
        {rssiDbm}
        <span className="ml-1 text-sm font-semibold text-dark-600">dBm</span>
      </p>
      <Badge variant={SIGNAL_LEVEL_BADGE_VARIANT[level]}>{SIGNAL_LEVEL_LABEL[level]}</Badge>
      <span className="text-sm text-dark-600">Baum {treeNumber}</span>
    </div>
  )
}

const LegendItem = ({
  colorClass,
  count,
  label,
}: {
  colorClass: string
  count: number
  label: string
}) => (
  <span>
    <span className={`mr-1 inline-block size-2 rounded-sm align-middle ${colorClass}`} />
    {count} {label}
  </span>
)

const Distribution = ({ summary }: { summary: ClusterSignalSummary }) => {
  const pct = (n: number) => `${(n / summary.total) * 100}%`
  const weakestLevel = summary.weakest ? signalLevelFromRssi(summary.weakest.rssiDbm) : 'weak'
  return (
    <div className="mt-3 flex flex-col gap-4 lg:flex-row lg:items-center lg:gap-8">
      <div className="lg:flex-1">
        <span className="flex h-2.5 overflow-hidden rounded-full bg-dark-200">
          {summary.good > 0 && (
            <span className="bg-green-dark" style={{ width: pct(summary.good) }} />
          )}
          {summary.fair > 0 && <span className="bg-yellow" style={{ width: pct(summary.fair) }} />}
          {summary.weak > 0 && <span className="bg-red" style={{ width: pct(summary.weak) }} />}
        </span>
        <div className="mt-2 flex flex-wrap gap-x-4 gap-y-1 text-xs text-dark-700">
          {summary.good > 0 && (
            <LegendItem colorClass="bg-green-dark" count={summary.good} label="gut" />
          )}
          {summary.fair > 0 && (
            <LegendItem colorClass="bg-yellow" count={summary.fair} label="ausreichend" />
          )}
          {summary.weak > 0 && (
            <LegendItem colorClass="bg-red" count={summary.weak} label="schwach" />
          )}
        </div>
      </div>
      {summary.weakest && (
        <div className="flex items-center gap-3 rounded-lg border border-dark-100 bg-background px-4 py-2.5">
          <SignalBars
            filled={signalBarsFromRssi(summary.weakest.rssiDbm)}
            className={SIGNAL_LEVEL_TEXT_COLOR[weakestLevel]}
          />
          <div>
            <p className="text-xs uppercase tracking-wide text-dark-600">Schwächster</p>
            <p className="text-sm font-bold tabular-nums">
              Baum {summary.weakest.treeNumber} · {summary.weakest.rssiDbm} dBm
            </p>
          </div>
        </div>
      )}
    </div>
  )
}

const ClusterSignalCard = ({ treecluster }: ClusterSignalCardProps) => {
  const treesTotal = treecluster.trees?.length ?? 0
  const summary = clusterSignalSummary(treecluster.trees)

  if (summary.total === 0) return null

  return (
    <section aria-labelledby="cluster-signal-heading" className="mt-5">
      <div className="rounded-xl bg-dark-50 p-5 sm:p-6">
        <div className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1">
          <p
            id="cluster-signal-heading"
            className="text-sm font-medium uppercase tracking-wide text-dark-700"
          >
            Signal in der Gruppe
          </p>
          <span className="text-sm text-dark-600 tabular-nums">
            {summary.total} von {treesTotal} {treesTotal === 1 ? 'Baum' : 'Bäumen'} mit Sensor
          </span>
        </div>

        {summary.total === 1 && summary.weakest ? (
          <SingleReading
            rssiDbm={summary.weakest.rssiDbm}
            treeNumber={summary.weakest.treeNumber}
          />
        ) : (
          <Distribution summary={summary} />
        )}
      </div>
    </section>
  )
}

export default ClusterSignalCard
