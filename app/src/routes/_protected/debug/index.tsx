import { useAuthStore, useMapStore, useUserStore } from '@/store/store'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug/')({
  component: Debug,
})

function Debug() {
  const authStore = useAuthStore()
  const mapStore = useMapStore()
  const userStore = useUserStore()

  return (
    <div className="container mt-6">
      <article className="2xl:w-4/5">
        <h1 className="font-lato font-bold text3xl mb-4 lg:text-4xl xl:text-5xl">Debugging</h1>
        <p>
          Diese Ansicht liefert debugging Informationen zu den aktuellen React-Stores im Frontend.
        </p>
      </article>
      <div className="mt-6">
        {JSON.stringify(authStore)}
        {JSON.stringify(mapStore)}
        {JSON.stringify(userStore)}
      </div>
    </div>
  )
}
