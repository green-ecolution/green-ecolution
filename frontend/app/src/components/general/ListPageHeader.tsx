import type { ReactNode } from 'react'

interface ListPageHeaderProps {
  title: ReactNode
  description?: ReactNode
  /** Short figures under the title, e.g. "21 Gruppen · 142 Bäume". */
  meta?: ReactNode
  action?: ReactNode
}

const ListPageHeader = ({ title, description, meta, action }: ListPageHeaderProps) => (
  <header className="mb-8 flex flex-col gap-4 lg:flex-row lg:items-start lg:justify-between">
    <article className="max-w-3xl">
      <h1 className="font-lato font-bold text-3xl lg:text-4xl xl:text-5xl">{title}</h1>
      {meta && <p className="mt-2 font-lato font-semibold text-dark-700">{meta}</p>}
      {description && <p className="mt-3 text-sm text-dark-600 max-w-prose">{description}</p>}
    </article>
    {action && <div className="flex shrink-0 flex-wrap items-center gap-3">{action}</div>}
  </header>
)

export default ListPageHeader
