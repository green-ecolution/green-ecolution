import type { LucideIcon } from 'lucide-react'

interface InfoTileProps {
  icon: LucideIcon
  label: string
  children: React.ReactNode
}

const InfoTile = ({ icon: Icon, label, children }: InfoTileProps) => (
  <div className="p-4 bg-dark-100/50 rounded-lg">
    <div className="flex items-center gap-2 mb-2">
      <Icon className="size-4 text-dark-500" />
      <p className="text-sm text-dark-500">{label}</p>
    </div>
    {children}
  </div>
)

export default InfoTile
