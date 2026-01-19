import { Outlet } from '@tanstack/react-router'
import Footer from './components/layout/Footer'
import Header from './components/layout/Header'
import useDocumentTitle from '@/hooks/useDocumentTitle'
import { Toaster } from '@green-ecolution/ui'

function App() {
  useDocumentTitle()

  return (
    <>
      <Header />
      <main className="flex-1 lg:pl-20">
        <Outlet />
      </main>
      <Footer />
      <Toaster />
    </>
  )
}

export default App
