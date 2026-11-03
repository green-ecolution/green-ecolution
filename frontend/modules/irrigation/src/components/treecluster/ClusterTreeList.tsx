import { useState } from 'react'
import { Badge, Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import TreeIcon from '@/components/icons/Tree'
import SensorIcon from '@/components/icons/Sensor'
import type { Tree } from '../../api/client'

interface ClusterTreeListProps {
  trees: Tree[]
}

const COLLAPSED_COUNT = 3

const rowClassName = 'flex items-center gap-3 py-3 text-sm'
const toggleClassName =
  'rounded-sm font-semibold text-green-dark hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-green-dark'

const ClusterTreeList = ({ trees }: ClusterTreeListProps) => {
  const { t } = useTranslation('treecluster')
  const [expanded, setExpanded] = useState(false)

  const sorted = [...trees].sort((a, b) => Number(Boolean(b.sensor)) - Number(Boolean(a.sensor)))
  const collapsible = sorted.length > COLLAPSED_COUNT
  const visible = collapsible && !expanded ? sorted.slice(0, COLLAPSED_COUNT) : sorted
  const hiddenCount = sorted.length - COLLAPSED_COUNT

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('treeList.title', { count: trees.length })}</CardTitle>
      </CardHeader>
      <CardContent>
        {trees.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('treeList.emptyMessage')}</p>
        ) : (
          <ul className="divide-y divide-dark-100">
            {visible.map((tree) => {
              const hasSensor = Boolean(tree.sensor)
              return (
                <li key={tree.id}>
                  <Link
                    to="/trees/$treeId"
                    params={{ treeId: tree.id }}
                    className={`${rowClassName} group rounded-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-green-dark`}
                  >
                    {hasSensor ? (
                      <SensorIcon className="size-4 shrink-0 text-green-dark" aria-hidden />
                    ) : (
                      <TreeIcon className="size-4 shrink-0 text-dark-400" aria-hidden />
                    )}
                    <span className="min-w-0 flex-1 truncate group-hover:underline">
                      <span className={hasSensor ? 'font-semibold' : undefined}>
                        {tree.species}
                      </span>
                      <span className="text-dark-600">
                        {' · '}
                        {t('treeList.number', { number: tree.number })}
                      </span>
                    </span>
                    {hasSensor ? (
                      <Badge variant="green-light">{t('treeList.sensorBadge')}</Badge>
                    ) : (
                      <span className="shrink-0 text-xs text-muted-foreground">
                        {t('treeList.noSensor')}
                      </span>
                    )}
                  </Link>
                </li>
              )
            })}
            {collapsible && (
              <li className={rowClassName}>
                {expanded ? (
                  <button
                    type="button"
                    className={`ml-auto ${toggleClassName}`}
                    onClick={() => setExpanded(false)}
                  >
                    {t('treeList.showLess')}
                  </button>
                ) : (
                  <>
                    <TreeIcon className="size-4 shrink-0 text-dark-400" aria-hidden />
                    <span className="flex-1 text-dark-600">
                      {t('treeList.moreTrees', { count: hiddenCount })}
                    </span>
                    <button
                      type="button"
                      className={toggleClassName}
                      onClick={() => setExpanded(true)}
                    >
                      {t('treeList.showAll')}
                    </button>
                  </>
                )}
              </li>
            )}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterTreeList
