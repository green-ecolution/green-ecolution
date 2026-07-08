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
    title={collapsed ? 'Ausklappen' : 'Einklappen'}
    className={`flex w-full items-center gap-x-3 text-light border border-transparent text-sm py-2.5 rounded-xl transition-all ease-in-out duration-300 hover:bg-green-light/20 hover:text-green-light-200 ${collapsed ? 'justify-center px-2' : 'justify-start px-3'}`}
  >
    <span className="shrink-0">
      {collapsed ? <PanelLeftOpen className="w-5 h-5" /> : <PanelLeftClose className="w-5 h-5" />}
    </span>
    {!collapsed && <span className="font-lato font-semibold tracking-[0.1]">Einklappen</span>}
  </button>
)

export default SidebarToggle
