package info

import (
	"context"
	"errors"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
	entities "github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils/enums"
)

type InfoService struct {
	infoRepository entities.InfoRepository
}

func NewInfoService(infoRepository entities.InfoRepository) *InfoService {
	return &InfoService{
		infoRepository: infoRepository,
	}
}

func (s *InfoService) GetAppInfo(ctx context.Context) (*entities.App, error) {
	log := logger.GetLogger(ctx)

	isAuth := ctx.Value(enums.ContextKeyClaims) != nil

	appInfo, err := s.infoRepository.GetAppInfo(ctx)
	if err != nil {
		if errors.Is(err, entities.ErrIPNotFound) {
			log.Debug("failed to receive ip from local system", "error", err)
		}
		if errors.Is(err, entities.ErrIFacesNotFound) {
			log.Debug("failed to receive network interfaces from local system", "error", err)
		}
		if errors.Is(err, entities.ErrIFacesAddressNotFound) {
			log.Debug("failed to receive network interfaces address from local system", "error", err)
		}
		if errors.Is(err, entities.ErrHostnameNotFound) {
			log.Debug("failed to receive network hostname from local system", "error", err)
		}

		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	if !isAuth {
		appInfo.Server = entities.Server{}
	}

	return appInfo, nil
}

func (s *InfoService) GetAppInfoResponse(ctx context.Context) (*entities.App, error) {
	appInfo, err := s.GetAppInfo(ctx)
	if err != nil {
		return nil, err
	}

	return appInfo, nil
}

func (s *InfoService) GetMapInfo(ctx context.Context) (*entities.Map, error) {
	return s.infoRepository.GetMapInfo(ctx)
}

func (s *InfoService) GetServerInfo(ctx context.Context) (*entities.Server, error) {
	log := logger.GetLogger(ctx)

	isAuth := ctx.Value(enums.ContextKeyClaims) != nil
	if !isAuth {
		log.Debug("unauthorized access to server info")
		return nil, ports.NewError(ports.Unauthorized, "authentication required for server info")
	}

	serverInfo, err := s.infoRepository.GetServerInfo(ctx)
	if err != nil {
		if errors.Is(err, entities.ErrIPNotFound) {
			log.Debug("failed to receive ip from local system", "error", err)
		}
		if errors.Is(err, entities.ErrIFacesNotFound) {
			log.Debug("failed to receive network interfaces from local system", "error", err)
		}
		if errors.Is(err, entities.ErrIFacesAddressNotFound) {
			log.Debug("failed to receive network interfaces address from local system", "error", err)
		}
		if errors.Is(err, entities.ErrHostnameNotFound) {
			log.Debug("failed to receive network hostname from local system", "error", err)
		}

		return nil, ports.MapError(ctx, err, ports.ErrorLogAll)
	}

	return serverInfo, nil
}

func (s *InfoService) GetServices(ctx context.Context) (*entities.Services, error) {
	return s.infoRepository.GetServices(ctx)
}

func (s *InfoService) GetStatistics(ctx context.Context) (*entities.DataStatistics, error) {
	return s.infoRepository.GetStatistics(ctx)
}

func (s *InfoService) Ready() bool {
	return s.infoRepository != nil
}
