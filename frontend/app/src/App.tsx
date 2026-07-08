import { Outlet } from '@tanstack/react-router'
import Footer from './components/layout/Footer'
import Header from './components/layout/Header'
import useDocumentTitle from '@/hooks/useDocumentTitle'
import { Toaster } from '@green-ecolution/ui'
import UpdateNotification from './components/layout/UpdateNotification'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'

function App() {
  useDocumentTitle()
  const collapsed = useSidebarCollapsed()

  return (
    <>
      <Header />
      <main
        className={`flex-1 transition-[padding] ease-in-out duration-300 ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'}`}
      >
        <Outlet />
      </main>
      <Footer />
      <Toaster />
      <UpdateNotification />
    </>
  )
}

export default App
