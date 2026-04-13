/// <reference types="vite/client" />
/// <reference types="vite-plugin-pwa/react" />

interface ImportMetaEnv {
  readonly BASE_URL: string

  readonly VITE_BACKEND_BASEURL: string
  readonly VITE_APP_CITY?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}

declare const __APP_VERSION__: string
declare const __APP_BUILD_TIME__: string
declare const __APP_CITY__: string
