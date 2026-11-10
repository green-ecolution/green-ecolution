import type { ReactNode } from 'react'

interface ListToolbarProps {
  search: ReactNode
  filters: ReactNode
  sort: ReactNode
}

// On phones the filters and the sort share one row that scrolls sideways; wrapped,
// they used to fill the first screen before a single list entry showed.
const ListToolbar = ({ search, filters, sort }: ListToolbarProps) => (
  <div className="flex flex-col gap-3">
    <div className="w-full sm:max-w-md">{search}</div>
    <div className="-mx-4 flex items-center gap-2 overflow-x-auto px-4 py-1 sm:mx-0 sm:flex-wrap sm:overflow-visible sm:px-0">
      <div className="flex shrink-0 items-center gap-2 sm:shrink sm:flex-wrap">{filters}</div>
      {sort && <div className="shrink-0 sm:ml-auto">{sort}</div>}
    </div>
  </div>
)

export default ListToolbar
