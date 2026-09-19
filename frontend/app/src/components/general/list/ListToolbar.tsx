import type { ReactNode } from 'react'

interface ListToolbarProps {
  search: ReactNode
  filters: ReactNode
  sort: ReactNode
  action?: ReactNode
}

const ListToolbar = ({ search, filters, sort, action }: ListToolbarProps) => (
  <div className="flex flex-col gap-3">
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
      <div className="w-full sm:max-w-md">{search}</div>
      {action && <div className="sm:ml-auto">{action}</div>}
    </div>
    <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
      <div className="flex flex-wrap items-center gap-2">{filters}</div>
      <div className="sm:ml-auto">{sort}</div>
    </div>
  </div>
)

export default ListToolbar
