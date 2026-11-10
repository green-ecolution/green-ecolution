import type { ComponentProps, ReactNode } from 'react'
import { Link, type LinkProps } from '@tanstack/react-router'
import { StatusCard } from '@green-ecolution/ui'

interface DashboardStatTileProps {
  label: string
  /** Undefined while loading. */
  value: number | undefined
  description?: ReactNode
  status?: ComponentProps<typeof StatusCard>['status']
  link: LinkProps
}

/** A figure on the dashboard that leads to the list behind it. Renders an `<li>`. */
const DashboardStatTile = ({ label, value, description, status, link }: DashboardStatTileProps) => (
  <li className="h-full">
    <Link
      {...link}
      className="block h-full rounded-xl transition-shadow duration-base ease-out hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 motion-reduce:transition-none"
    >
      <StatusCard
        label={label}
        value={value ?? '–'}
        description={description}
        status={value === undefined ? 'default' : status}
        indicator="dot"
        isLarge
      />
    </Link>
  </li>
)

export default DashboardStatTile
