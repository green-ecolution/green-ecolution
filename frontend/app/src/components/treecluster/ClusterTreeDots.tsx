import React from 'react'

interface ClusterTreeDotsProps {
  treeCount: number
  sensorCount: number
}

const MAX_DOTS = 24

const ClusterTreeDots: React.FC<ClusterTreeDotsProps> = ({ treeCount, sensorCount }) => {
  const shown = Math.min(treeCount, MAX_DOTS)
  const overflow = treeCount - shown
  const highlighted = Math.min(sensorCount, shown)

  if (treeCount === 0) {
    return <p className="text-sm text-dark-600">Noch keine Bäume zugeordnet</p>
  }

  return (
    <div>
      <div className="flex flex-wrap items-center gap-1.5" aria-hidden>
        {Array.from({ length: shown }, (_, i) => {
          const isSensor = i < highlighted
          return (
            <span
              key={i}
              data-dot={isSensor ? 'sensor' : 'tree'}
              className={
                isSensor
                  ? 'h-2.5 w-2.5 rounded-full bg-green-dark ring-2 ring-green-dark-200'
                  : 'h-2.5 w-2.5 rounded-full bg-dark-300'
              }
            />
          )
        })}
        {overflow > 0 && (
          <span className="ml-0.5 text-xs font-semibold text-dark-600">+{overflow}</span>
        )}
      </div>

      {sensorCount > 0 && (
        <p className="mt-2.5 flex items-center gap-1.5 text-xs text-dark-600">
          <span className="h-2 w-2 rounded-full bg-green-dark" aria-hidden />
          Sensor-Baum · misst die Gruppe
        </p>
      )}
    </div>
  )
}

export default ClusterTreeDots
