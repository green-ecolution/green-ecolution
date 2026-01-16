import { Link, LinkProps } from '@tanstack/react-router'
import { MoveRight } from 'lucide-react'
import React from 'react'
import { Button } from '@green-ecolution/ui'

interface ButtonLinkProps {
  label: string
  link: LinkProps
  color?: 'green' | 'grey'
  iconClassName?: string
  icon: React.ComponentType<React.SVGProps<SVGSVGElement>>
}

const ButtonLink: React.FC<ButtonLinkProps> = ({
  label,
  icon: Icon = MoveRight,
  link,
  color = 'green',
  iconClassName = '',
}) => {
  return (
    <Button variant={color === 'grey' ? 'outline' : 'default'} asChild>
      <Link {...link}>
        {label}
        {Icon && <Icon className={iconClassName} />}
      </Link>
    </Button>
  )
}

export default ButtonLink
