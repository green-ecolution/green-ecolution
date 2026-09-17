import type { ReactNode } from 'react'

interface ListToolbarProps {
  search: ReactNode
  filters: ReactNode
  sort: ReactNode
  action?: ReactNode
}

const ListToolbar = ({ search, filters, sort, action }: ListToolbarProps) => (
  <div className="flex flex-col gap-3 lg:flex-row lg:flex-wrap lg:items-center">
    {search}
    <div className="flex flex-wrap items-center gap-2">{filters}</div>
    <div className="flex flex-wrap items-center gap-2 lg:ml-auto">
      {sort}
      {action}
    </div>
  </div>
)

export default ListToolbar
