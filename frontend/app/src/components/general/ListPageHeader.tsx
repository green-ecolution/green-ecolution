import type { ReactNode } from 'react'

interface ListPageHeaderProps {
  title: ReactNode
  description: ReactNode
  action?: ReactNode
}

const ListPageHeader = ({ title, description, action }: ListPageHeaderProps) => (
  <article className="mb-20 2xl:w-4/5">
    <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">{title}</h1>
    <p className="mb-5">{description}</p>
    {action}
  </article>
)

export default ListPageHeader
