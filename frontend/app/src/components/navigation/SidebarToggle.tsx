import { PanelLeftClose, PanelLeftOpen } from 'lucide-react'
import React from 'react'
import { useTranslation } from 'react-i18next'

interface SidebarToggleProps {
  collapsed: boolean
  onToggle: () => void
}

const SidebarToggle: React.FC<SidebarToggleProps> = ({ collapsed, onToggle }) => {
  const { t } = useTranslation('navigation')

  return (
    <button
      type="button"
      onClick={onToggle}
      aria-expanded={!collapsed}
      aria-controls="main-navigation"
      aria-label={collapsed ? t('sidebar.toggleExpand') : t('sidebar.toggleCollapse')}
      className="inline-flex cursor-pointer items-center justify-center text-dark transition-colors hover:text-dark-600"
    >
      {collapsed ? <PanelLeftOpen className="w-5 h-5" /> : <PanelLeftClose className="w-5 h-5" />}
    </button>
  )
}

export default SidebarToggle
