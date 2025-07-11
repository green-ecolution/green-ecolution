import { Outlet } from '@tanstack/react-router'
import Footer from './components/layout/Footer'
import Header from './components/layout/Header'
import useDocumentTitle from '@/hooks/useDocumentTitle'
import ToastProvider from './context/ToastContext'

function App() {
  useDocumentTitle()

  return (
    <>
      <ToastProvider>
        <Header />
        <main className="flex-1 lg:pl-20">
          <Outlet />
        </main>
        <Footer />
      </ToastProvider>
    </>
  )
}

export default App
