import { Link, LinkProps } from '@tanstack/react-router'
import { MoveLeft } from 'lucide-react'
import { Button } from '@green-ecolution/ui'

interface BackLinkProps {
  label: string
  link: LinkProps
}

// From lg the header breadcrumb leads back, so the link would only repeat it.
const BackLink = ({ label, link }: BackLinkProps) => (
  <Button
    variant="nav"
    asChild
    className="text-dark-600 text-sm mb-4 p-0 h-auto [&_svg]:size-4 lg:hidden"
  >
    <Link {...link}>
      <MoveLeft className="icon-arrow-back-animate" />
      {label}
    </Link>
  </Button>
)

export default BackLink
