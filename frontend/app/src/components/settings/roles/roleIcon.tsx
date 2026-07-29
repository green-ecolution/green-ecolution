import { Eye, RadioTower, Route, Shield, ShieldCheck, TreeDeciduous } from 'lucide-react'
import type { Role } from '@/api/backendApi'

interface RoleIconProps {
  role: Pick<Role, 'name' | 'isTemplate'> | null
  className?: string
}

/**
 * System roles get a themed glyph keyed by their (immutable) template name;
 * own roles share one icon for now. Rendered with literal JSX so the icon is
 * never held in a render-scope variable (react-x/static-components).
 */
const RoleIcon = ({ role, className }: RoleIconProps) => {
  if (role?.isTemplate) {
    switch (role.name) {
      case 'Administrator':
        return <ShieldCheck className={className} aria-hidden />
      case 'Baumpflege':
        return <TreeDeciduous className={className} aria-hidden />
      case 'Sensorik':
        return <RadioTower className={className} aria-hidden />
      case 'Routenplanung':
        return <Route className={className} aria-hidden />
      case 'Beobachter':
        return <Eye className={className} aria-hidden />
    }
  }
  return <Shield className={className} aria-hidden />
}

export default RoleIcon
