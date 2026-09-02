import { useEffect } from 'react'
import { Outlet } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import Footer from './components/layout/Footer'
import Header from './components/layout/Header'
import useDocumentTitle from '@/hooks/useDocumentTitle'
import { Toaster } from '@green-ecolution/ui'
import UpdateNotification from './components/layout/UpdateNotification'
import { useSidebarCollapsed } from '@/hooks/useSidebarCollapsed'

function App() {
  useDocumentTitle()
  const collapsed = useSidebarCollapsed()
  const { i18n } = useTranslation()

  // Screen readers and the browser's spell checker read this attribute, not
  // the i18next state.
  useEffect(() => {
    document.documentElement.lang = i18n.language
  }, [i18n.language])

  return (
    <>
      <Header />
      <main
        className={`flex-1 transition-[padding] ease-in-out duration-300 motion-reduce:transition-none ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'}`}
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
