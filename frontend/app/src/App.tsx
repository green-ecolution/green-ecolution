import { useEffect } from 'react'
import { Outlet, useLocation } from '@tanstack/react-router'
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
  const { pathname } = useLocation()
  const { i18n } = useTranslation()

  // A plugin view fills whatever the header and footer leave over. That only
  // works if main is a flex column -- a percentage height on the frame would
  // resolve against an indefinite flex item and collapse to the 150px default.
  const fillsFreeSpace = pathname.startsWith('/plugin/')

  // Screen readers and the browser's spell checker read this attribute, not
  // the i18next state.
  useEffect(() => {
    document.documentElement.lang = i18n.language
  }, [i18n.language])

  return (
    <>
      <Header />
      <main
        className={`flex-1 transition-[padding] ease-in-out duration-300 motion-reduce:transition-none ${fillsFreeSpace ? 'flex flex-col' : ''} ${collapsed ? 'lg:pl-[4.5rem]' : 'lg:pl-[16rem]'}`}
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
