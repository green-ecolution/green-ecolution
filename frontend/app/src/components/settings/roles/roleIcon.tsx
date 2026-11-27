import { Shield } from 'lucide-react'
import type { Role } from '@/api/backendApi'
import { moduleManifests } from '@/modules/registry'
import { roleTemplateOf } from '@/modules/roleTemplates'

interface RoleIconProps {
  role: Pick<Role, 'templateKey'> | null
  className?: string
}

/** Delivered roles get the glyph their template declares; own roles share one icon. */
const RoleIcon = ({ role, className }: RoleIconProps) => {
  const template = roleTemplateOf(role?.templateKey, moduleManifests)
  if (!template) return <Shield className={className} aria-hidden />
  return <template.icon className={className} aria-hidden />
}

export default RoleIcon
