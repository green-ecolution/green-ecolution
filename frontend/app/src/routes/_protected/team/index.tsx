import UserCard from '@/components/general/cards/UserCard'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import { createFileRoute } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { userRoleQuery } from '@/api/queries'

export const Route = createFileRoute('/_protected/team/')({
  component: Team,
  pendingComponent: () => <LoadingInfo label="Daten werden geladen" />,
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(userRoleQuery('tbz')),
})

function Team() {
  const { data: userRes } = useSuspenseQuery(userRoleQuery('tbz'))

  return (
    <div className="container mt-6">
      <article className="mb-20 2xl:w-4/5">
        <h1 className="font-lato font-bold text-3xl mb-4 lg:text-4xl xl:text-5xl">
          Alle Mitarbeitenden
        </h1>
        <p className="mb-5">
          Hier finden Sie eine Übersicht aller Mitarbeitenden und weitere Informationen zu deren
          Rollen und welche Führerscheinklasse sie besitzen. Diese Informationen sind wichtig, wenn
          Personen zu einem Einsatzplan eingeteilt werden sollen.
        </p>
      </article>

      <section className="mt-10">
        <header className="hidden border-b pb-2 text-sm text-dark-800 px-8 border-b-dark-200 mb-5 lg:grid lg:grid-cols-[1fr,1.25fr,1fr,1fr] lg:gap-5 xl:px-10">
          <p>Verfügbarkeit</p>
          <p>Name</p>
          <p>Organisation</p>
          <p>Führerscheinklasse</p>
        </header>
        <ul>
          {userRes.data.length === 0 ? (
            <li className="text-center text-dark-600 mt-10">
              <p>Es wurden leider keine Mitarbeitenden gefunden.</p>
            </li>
          ) : (
            userRes.data.map((user) => (
              <li key={user.email} className="mb-5 last:mb-0">
                <UserCard user={user} />
              </li>
            ))
          )}
        </ul>
      </section>
    </div>
  )
}
