import { Link, LinkProps } from '@tanstack/react-router'
import { MoveRight } from 'lucide-react'
import { Button } from '@green-ecolution/ui'

interface GeneralLinkProps {
  label: string
  link: LinkProps
  theme?: 'grey' | 'green'
}

const GeneralLink = ({ label, link, theme = 'green' }: GeneralLinkProps) => (
  <Button
    variant="nav"
    asChild
    className={`p-0 h-auto [&_svg]:size-4 ${theme === 'green' ? 'text-green-dark' : 'text-dark-600'}`}
  >
    <Link {...link}>
      {label}
      <MoveRight className="icon-arrow-animate" />
    </Link>
  </Button>
)

export default GeneralLink
