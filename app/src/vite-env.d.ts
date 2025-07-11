/// <reference types: string

interface ImportMetaEnv {
  readonly BASE_URL: string

  readonly VITE_BACKEND_BASEURL: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
