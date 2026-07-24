import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import React from 'react'

interface SidebarToggleProps {
  collapsed: boolean
  onToggle: () => void
}

const SidebarToggle: React.FC<SidebarToggleProps> = ({ collapsed, onToggle }) => (
  <button
    type="button"
    onClick={onToggle}
    aria-expanded={!collapsed}
    aria-controls="main-navigation"
    aria-label={collapsed ? 'Seitennavigation ausklappen' : 'Seitennavigation einklappen'}
    className="inline-flex cursor-pointer items-center justify-center text-dark transition-colors hover:text-dark-600"
  >
    {collapsed ? <PanelLeftOpen className="w-5 h-5" /> : <PanelLeftClose className="w-5 h-5" />}
  </button>
)

export default SidebarToggle
