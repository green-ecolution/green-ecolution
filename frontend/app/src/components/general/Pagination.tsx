import { Pagination as PaginationObject } from '@green-ecolution/backend-client'
import React from 'react'
import PaginationLink from './links/PaginationLink'
import { ChevronLeft, ChevronRight } from 'lucide-react'

interface PaginationProps {
  pagination: PaginationObject
}

const Pagination: React.FC<PaginationProps> = ({ pagination }) => {
  return (
    <nav aria-label="Seiten-Paginierung" className="mt-10">
      <ul className="flex flex-wrap items-center justify-center gap-x-2">
        <li>
          <PaginationLink
            aria-label="Vorherige Seite anzeigen"
            color="green-dark"
            disabled={pagination.currentPage - 1 == 0}
            icon={ChevronLeft}
            link={{
              search: (prev) => ({
                ...prev,
                page: pagination.currentPage - 1,
              }),
            }}
          />
        </li>
        {pagination.prevPage > 1 && (
          <li className="flex items-end gap-x-2">
            <PaginationLink
              aria-label="Zur ersten Seite springen"
              color="grey"
              label={1}
              link={{
                search: (prev) => ({
                  ...prev,
                  page: 1,
                }),
              }}
            />
            {pagination.prevPage > 2 && (
              <span aria-hidden="true" className="text-dark-600 font-semibold">
                ...
              </span>
            )}
          </li>
        )}

        {pagination.prevPage && (
          <li>
            <PaginationLink
              aria-label={`Zur vorherigen Seite ${pagination.prevPage} springen`}
              color="grey"
              label={pagination.prevPage}
              link={{
                search: (prev) => ({
                  ...prev,
                  page: pagination.prevPage,
                }),
              }}
            />
          </li>
        )}
        <li>
          <PaginationLink
            aria-label={`Aktuelle Seite ${pagination.prevPage}`}
            color="green-light"
            label={pagination.currentPage}
            link={{
              search: (prev) => ({
                ...prev,
                page: pagination.currentPage,
              }),
            }}
          />
        </li>
        {pagination.nextPage && (
          <li>
            <PaginationLink
              aria-label={`Zur nächsten Seite ${pagination.nextPage} springen`}
              color="grey"
              label={pagination.nextPage}
              link={{
                search: (prev) => ({
                  ...prev,
                  page: pagination.nextPage,
                }),
              }}
            />
          </li>
        )}
        {pagination.nextPage && pagination.nextPage < pagination.totalPages && (
          <li className="flex items-end gap-x-2">
            {pagination.nextPage < pagination.totalPages - 1 && (
              <span aria-hidden="true" className="text-dark-600 font-semibold">
                ...
              </span>
            )}
            <PaginationLink
              aria-label="Zur letzten Seite springen"
              color="grey"
              label={pagination.totalPages}
              link={{
                search: (prev) => ({
                  ...prev,
                  page: pagination.totalPages,
                }),
              }}
            />
          </li>
        )}
        <li>
          <PaginationLink
            aria-label="Vorherige Seite anzeigen"
            color="green-dark"
            disabled={pagination.currentPage == pagination.totalPages}
            icon={ChevronRight}
            link={{
              search: (prev) => ({
                ...prev,
                page: pagination.currentPage + 1,
              }),
            }}
          />
        </li>
      </ul>
    </nav>
  )
}

export default Pagination
