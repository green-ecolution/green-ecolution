import type { ReactNode } from 'react'
import type { LinkProps } from '@tanstack/react-router'
import BackLink from './links/BackLink'

interface FormPageHeaderProps {
  backLink: { link: LinkProps; label: string }
  title: ReactNode
  children?: ReactNode
}

const FormPageHeader = ({ backLink, title, children }: FormPageHeaderProps) => (
  <article className="2xl:w-4/5">
    <BackLink label={backLink.label} link={backLink.link} />
    <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">{title}</h1>
    {children}
  </article>
)

export default FormPageHeader
