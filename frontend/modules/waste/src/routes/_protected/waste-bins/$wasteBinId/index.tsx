import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { pendingLoading } from '@/lib/router'
import { wasteBinQueries } from '../../../../api/queries'
import WasteBinDetail from '../../../../components/WasteBinDetail'

const wasteBinRoute = getRouteApi('/_protected/waste-bins/$wasteBinId')

export const Route = createFileRoute('/_protected/waste-bins/$wasteBinId/')({
  pendingComponent: pendingLoading({ key: 'waste:detail.loadingLabel' }),
  component: SingleWasteBin,
})

function SingleWasteBin() {
  const { wasteBinId } = wasteBinRoute.useParams()
  // Live query instead of loader data: the fill level arrives with sensor
  // readings and must keep polling.
  const { data: bin } = useSuspenseQuery({
    ...wasteBinQueries.detail(wasteBinId),
    refetchInterval: 30_000,
    refetchOnWindowFocus: true,
  })

  return (
    <div className="container mt-6">
      <WasteBinDetail bin={bin} />
    </div>
  )
}
