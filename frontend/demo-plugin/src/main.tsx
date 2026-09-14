import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { PluginProvider } from '@green-ecolution/plugin-interface'
import App from './App'
import './index.css'

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <PluginProvider>
      <App />
    </PluginProvider>
  </StrictMode>,
)
