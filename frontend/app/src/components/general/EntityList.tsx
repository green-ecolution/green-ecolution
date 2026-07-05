import type { ReactNode } from 'react'

interface EntityListProps<T> {
  items: T[]
  getKey: (item: T) => string | number
  renderItem: (item: T) => ReactNode
  emptyMessage: string
  layout?: 'list' | 'grid'
}

const EntityList = <T,>({
  items,
  getKey,
  renderItem,
  emptyMessage,
  layout = 'list',
}: EntityListProps<T>) => {
  if (layout === 'grid') {
    if (items.length === 0) {
      return <p className="mt-10 text-center text-dark-600">{emptyMessage}</p>
    }

    return (
      <ul className="grid grid-cols-1 gap-5 sm:grid-cols-2 xl:grid-cols-3">
        {items.map((item) => (
          <li key={getKey(item)} className="h-full">
            {renderItem(item)}
          </li>
        ))}
      </ul>
    )
  }

  return (
    <ul>
      {items.length === 0 ? (
        <li className="text-center text-dark-600 mt-10">
          <p>{emptyMessage}</p>
        </li>
      ) : (
        items.map((item) => (
          <li key={getKey(item)} className="mb-5 last:mb-0">
            {renderItem(item)}
          </li>
        ))
      )}
    </ul>
  )
}

export default EntityList
