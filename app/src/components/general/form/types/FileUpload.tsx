import React, { useEffect, useRef } from 'react'
import { Trash2 } from 'lucide-react'

interface FileUploadProps {
  fileType: string
  name: string
  message?: string
  isFileSelected: boolean
  color: string
  clearFileInput: () => void
  handleFileChange: (event: React.ChangeEvent<HTMLInputElement>) => void
}

const FileUpload: React.FC<FileUploadProps> = ({
  fileType,
  name = 'file',
  message = '',
  handleFileChange,
  clearFileInput,
  isFileSelected,
  color,
}) => {
  const inputFileRef = useRef<HTMLInputElement | null>(null)

  const handleClearInput = () => {
    if (inputFileRef.current) {
      inputFileRef.current.value = ''
    }
    clearFileInput()
  }

  useEffect(() => {
    if (message && inputFileRef.current) {
      inputFileRef.current.value = ''
    }
  }, [message])

  return (
    <>
      <div className="flex items-center gap-x-4">
        <input
          ref={inputFileRef}
          accept={fileType}
          id={name}
          type="file"
          onChange={handleFileChange}
          className="w-full lg:w-1/2 text-dark-800 border border-green-light rounded-lg bg-white px-4 py-3 focus:outline-green-dark"
        />

        {isFileSelected && (
          <button type="button" onClick={handleClearInput} className="flex items-center">
            <Trash2 className="text-dark-600" />
            <span className="sr-only">Datei unselektieren</span>
          </button>
        )}
      </div>

      {message && !isFileSelected && (
        <div className={`mt-4 font-semibold text-sm ${color}`}>{message}</div>
      )}
    </>
  )
}

export default FileUpload
