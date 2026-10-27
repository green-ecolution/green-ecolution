import { useEffect, useEffectEvent, useState } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { Droplet } from 'lucide-react'
import { FormField } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import type { StopOutcomeDto, TreeClusterInListResponse } from '@green-ecolution/backend-client'
import type { OutcomeInputProps } from '@/modules/types'
import { clusterQueries } from '../../api/queries'
import { clusterOutcomeSchema } from '../../schema/clusterOutcomeSchema'

const LITERS_PER_TREE = 80

const defaultLiters = (cluster: TreeClusterInListResponse) =>
  String(cluster.treeIds.length * LITERS_PER_TREE)

const toOutcomes = (
  clusters: TreeClusterInListResponse[],
  liters: Record<string, string>,
): { outcomes: StopOutcomeDto[]; valid: boolean } => {
  const parsed = clusterOutcomeSchema.safeParse({
    outcomes: clusters.map(({ id }) => ({ stop: id, amount: liters[id] })),
  })
  if (!parsed.success) return { outcomes: [], valid: false }
  return {
    outcomes: parsed.data.outcomes.map(({ stop, amount }) => ({ stop, completed: true, amount })),
    valid: true,
  }
}

const OutcomeFields = ({
  clusters,
  onChange,
}: {
  clusters: TreeClusterInListResponse[]
  onChange: OutcomeInputProps['onChange']
}) => {
  const { t } = useTranslation('irrigation')
  const [liters, setLiters] = useState<Record<string, string>>(() =>
    Object.fromEntries(clusters.map((cluster) => [cluster.id, defaultLiters(cluster)])),
  )
  const report = useEffectEvent((current: Record<string, string>) => {
    const { outcomes, valid } = toOutcomes(clusters, current)
    onChange(outcomes, valid)
  })
  useEffect(() => report(liters), [liters])

  return (
    <fieldset className="mt-6">
      <legend className="block font-semibold text-dark-800 mb-2.5">
        {t('planning.outcome.waterPerClusterLegend')}
      </legend>
      <p className="-mt-2 text-sm text-dark-600 mb-2.5">{t('planning.outcome.defaultsHint')}</p>
      <ul className="flex flex-col">
        {clusters.map((cluster) => {
          const treeCount = cluster.treeIds.length
          return (
            <li
              key={cluster.id}
              className="flex items-center justify-between gap-4 border-b border-dark-100 py-3 last:border-0"
            >
              <div className="flex min-w-0 items-center gap-2.5">
                <span
                  aria-hidden
                  className="flex size-8 shrink-0 items-center justify-center rounded-full bg-green-light-100"
                >
                  <Droplet className="size-4 text-green-dark" />
                </span>
                <div className="min-w-0">
                  <p className="truncate font-medium text-dark">{cluster.name}</p>
                  {treeCount > 0 && (
                    <p className="text-xs tabular-nums text-dark-600">
                      {t('planning.outcome.treeCount', { count: treeCount })}
                    </p>
                  )}
                </div>
              </div>
              <div className="flex shrink-0 items-center gap-2">
                <FormField
                  type="number"
                  label={t('planning.outcome.litersForClusterLabel', { name: cluster.name })}
                  value={liters[cluster.id]}
                  onChange={(event) =>
                    setLiters((current) => ({ ...current, [cluster.id]: event.target.value }))
                  }
                  className="max-w-28"
                  hideLabel
                />
                <span className="text-sm text-dark-600">
                  {t('planning.outcome.litersUnitLabel')}
                </span>
              </div>
            </li>
          )
        })}
      </ul>
    </fieldset>
  )
}

const ClusterOutcomeInput = ({ plan, onChange }: OutcomeInputProps) => {
  const { data } = useSuspenseQuery(clusterQueries.stops(plan.stops))
  return <OutcomeFields key={plan.id} clusters={data.clusters} onChange={onChange} />
}

export default ClusterOutcomeInput
