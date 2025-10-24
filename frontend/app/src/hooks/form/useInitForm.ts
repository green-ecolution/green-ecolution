import { useSuspenseQuery, UseSuspenseQueryOptions } from '@tanstack/react-query'

export const useInitFormQuery = <TData, TSchema>(
  queryOptions: UseSuspenseQueryOptions<TData>,
  handler: (v: TData) => TSchema,
) => {
  const { data } = useSuspenseQuery(queryOptions)
  return { initForm: handler(data), loadedData: data }
}
