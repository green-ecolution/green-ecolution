package pagination

import (
	"context"

	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	httpEntities "github.com/green-ecolution/green-ecolution/backend/internal/interface/http/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

func GetValues(ctx context.Context) (page, limit int32, err error) {
	log := logger.GetLogger(ctx)
	page, pageOk := ctx.Value("page").(int32)
	limit, limitOK := ctx.Value("limit").(int32)

	if !pageOk || !limitOK {
		page = 1
		limit = -1
	}

	if page <= 0 || (limit != -1 && limit <= 0) {
		log.Debug("pagination values are invalid", "page", ctx.Value("page"), "limit", ctx.Value("limit"))
		return page, limit, entities.ErrPaginationValueInvalid
	}

	return page, limit, nil
}

func Create(ctx context.Context, totalCount int64) *httpEntities.Pagination {
	page, pageOk := ctx.Value("page").(int32)
	limit, limitOk := ctx.Value("limit").(int32)

	if !pageOk || !limitOk || limit == -1 {
		return nil
	}

	totalPages, nextPage, prevPage := calculatePaginationValues(int32(totalCount), limit, page)

	return &httpEntities.Pagination{
		Total:       totalCount,
		CurrentPage: page,
		TotalPages:  totalPages,
		NextPage:    nextPage,
		PrevPage:    prevPage,
	}
}

func calculatePaginationValues(totalCount, limit, page int32) (totalPages int32, nextPage, prevPage *int32) {
	if limit <= 0 {
		limit = 1
	}

	totalPages = (totalCount + limit - 1) / limit

	if page < totalPages {
		next := page + 1
		nextPage = &next
	}

	if page > 1 {
		prev := page - 1
		prevPage = &prev
	}

	if page == totalPages {
		nextPage = nil
	}

	if page == 1 {
		prevPage = nil
	}

	return totalPages, nextPage, prevPage
}
