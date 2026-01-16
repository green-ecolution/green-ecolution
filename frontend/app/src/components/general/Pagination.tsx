import { Pagination as PaginationObject } from '@green-ecolution/backend-client'
import { SimplePagination } from '@green-ecolution/ui'
import { useRouter, useSearch } from '@tanstack/react-router'

interface PaginationProps {
  pagination: PaginationObject
}

const Pagination = ({ pagination }: PaginationProps) => {
  const router = useRouter()
  const search = useSearch({ strict: false })

  const handlePageChange = (page: number) => {
    router
      .navigate({
        to: '.',
        search: { ...search, page },
      })
      .catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <SimplePagination
      className="mt-10"
      pagination={{
        currentPage: pagination.currentPage,
        totalPages: pagination.totalPages,
        hasNextPage: !!pagination.nextPage,
        hasPreviousPage: !!pagination.prevPage,
      }}
      onPageChange={handlePageChange}
    />
  )
}

export default Pagination
