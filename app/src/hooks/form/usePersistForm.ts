import { useEffect } from 'react'
import { FieldValues, UseFormWatch } from 'react-hook-form'

export interface FormPersistConfig<T extends FieldValues> {
  storage?: Storage
  watch: UseFormWatch<T>
  exclude?: string[]
}

const useFormPersist = <T extends FieldValues>(
  name: string,
  { storage, watch, exclude = [] }: FormPersistConfig<T>,
) => {
  const getStorage = storage ?? window.sessionStorage

  useEffect(() => {
    const { unsubscribe } = watch((data) => {
      const entries = Object.entries(data) as [string, T[keyof T]][]
      const values = exclude.length
        ? entries
            .filter(([key]) => !exclude.includes(key))
            .reduce((obj, [key, val]) => Object.assign(obj, { [key]: val }), {})
        : Object.assign({}, data)

      if (Object.entries(values).length) {
        getStorage.setItem(name, JSON.stringify(values))
      }
    })

    return () => unsubscribe()
  }, [watch, getStorage, name, exclude])

  return {
    clear: () => getStorage.removeItem(name),
  }
}

export default useFormPersist
