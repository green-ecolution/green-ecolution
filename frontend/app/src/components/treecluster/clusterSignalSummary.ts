import type { Tree } from '@/api/backendApi'
import { parseSignal, signalLevelFromRssi } from '@/components/sensor/detail/signalParsing'

export interface ClusterSignalSummary {
  total: number
  good: number
  fair: number
  weak: number
  weakest: { treeNumber: string; rssiDbm: number } | null
}

export const clusterSignalSummary = (trees: Tree[] | undefined): ClusterSignalSummary => {
  const summary: ClusterSignalSummary = { total: 0, good: 0, fair: 0, weak: 0, weakest: null }
  for (const tree of trees ?? []) {
    const signal = parseSignal(tree.sensor?.latestData)
    if (!signal) continue
    summary.total += 1
    summary[signalLevelFromRssi(signal.rssiDbm)] += 1
    if (!summary.weakest || signal.rssiDbm < summary.weakest.rssiDbm) {
      summary.weakest = { treeNumber: tree.number, rssiDbm: signal.rssiDbm }
    }
  }
  return summary
}
