import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import React from 'react'
import { navItemClasses } from './navItemStyles'

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
    title={collapsed ? 'Ausklappen' : 'Einklappen'}
    className={`${navItemClasses} w-full border-transparent ${collapsed ? 'justify-center px-2' : 'justify-start px-3'}`}
  >
    <span className="shrink-0">
      {collapsed ? <PanelLeftOpen className="w-5 h-5" /> : <PanelLeftClose className="w-5 h-5" />}
    </span>
    {!collapsed && <span className="font-lato font-semibold tracking-[0.1]">Einklappen</span>}
  </button>
)

export default SidebarToggle
