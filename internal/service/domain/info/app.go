package info

import (
	"context"
	"errors"

	domain "github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/backend/internal/service"
	"github.com/green-ecolution/backend/internal/storage"
)

type InfoService struct {
	infoRepository storage.InfoRepository
}

func NewInfoService(infoRepository storage.InfoRepository) *InfoService {
	return &InfoService{
		infoRepository: infoRepository,
	}
}

func (s *InfoService) GetAppInfo(ctx context.Context) (*domain.App, error) {
	log := logger.GetLogger(ctx)
	appInfo, err := s.infoRepository.GetAppInfo(ctx)
	if err != nil {
		if errors.Is(err, storage.ErrIPNotFound) {
			log.Debug("failed to receive ip from local system", "error", err)
		}
		if errors.Is(err, storage.ErrIFacesNotFound) {
			log.Debug("failed to receive network interfaces from local system", "error", err)
		}
		if errors.Is(err, storage.ErrIFacesAddressNotFound) {
			log.Debug("failed to receive network interfaces address from local system", "error", err)
		}
		if errors.Is(err, storage.ErrHostnameNotFound) {
			log.Debug("failed to receive network hostname from local system", "error", err)
		}

		return nil, service.MapError(ctx, err, service.ErrorLogAll)
	}

	return appInfo, nil
}

func (s *InfoService) GetAppInfoResponse(ctx context.Context) (*domain.App, error) {
	appInfo, err := s.GetAppInfo(ctx)
	if err != nil {
		return nil, err
	}

	return appInfo, nil
}

func (s *InfoService) Ready() bool {
	return s.infoRepository != nil
}
