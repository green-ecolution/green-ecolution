import { toast } from '@green-ecolution/ui'

type ToastType = 'success' | 'error'

const useToast = () => {
  return (message: string, type: ToastType = 'success') => {
    toast[type](message)
  }
}

export default useToast
