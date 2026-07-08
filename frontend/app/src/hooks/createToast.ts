import { toast } from '@green-ecolution/ui'

type ToastType = 'success' | 'error'
type ToastOptions = Parameters<(typeof toast)['success']>[1]

const createToast = () => {
  return (message: string, type: ToastType = 'success', options?: ToastOptions) => {
    toast[type](message, options)
  }
}

export default createToast
