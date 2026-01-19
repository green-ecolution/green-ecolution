import { infoQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/info')({
  component: Info,
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Loading App Info" />,
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(infoQuery()),
})

function Info() {
  const { data } = useSuspenseQuery(infoQuery())

  return (
    <div>
      <h1>App Info</h1>
      <pre>{JSON.stringify(data, null, 2)}</pre>
    </div>
  )
}
