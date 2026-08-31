import { Eye, RadioTower, Route, Shield, ShieldCheck, TreeDeciduous } from 'lucide-react'
import type { Role } from '@/api/backendApi'

interface RoleIconProps {
  role: Pick<Role, 'templateKey'> | null
  className?: string
}

/**
 * Delivered roles get a themed glyph keyed by their stable template key; own
 * roles share one icon for now. Rendered with literal JSX so the icon is never
 * held in a render-scope variable (react-x/static-components).
 */
const RoleIcon = ({ role, className }: RoleIconProps) => {
  switch (role?.templateKey) {
    case 'administrator':
      return <ShieldCheck className={className} aria-hidden />
    case 'tree_care':
      return <TreeDeciduous className={className} aria-hidden />
    case 'sensors':
      return <RadioTower className={className} aria-hidden />
    case 'route_planning':
      return <Route className={className} aria-hidden />
    case 'observer':
      return <Eye className={className} aria-hidden />
    default:
      return <Shield className={className} aria-hidden />
  }
}

export default RoleIcon
