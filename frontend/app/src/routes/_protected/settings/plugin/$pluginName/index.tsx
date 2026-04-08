import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/settings/plugin/$pluginName/')({
  component: PluginView,
})

function PluginView() {
  const { pluginName } = Route.useParams()

  return (
    <div className="container mt-6">
      <p>
        Das Plugin <strong>{pluginName}</strong> kann derzeit nicht geladen werden. Das Plugin-System
        wird überarbeitet.
      </p>
    </div>
  )
}
