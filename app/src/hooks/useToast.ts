import { ToastContext } from '@/context/ToastContext'
import { use } from 'react'

const useToast = () => {
  const context = use(ToastContext)
  if (context === undefined) {
    throw new Error('useToast must be used within a ToastProvider')
  }
  return context.showToast
}

export default useToast
