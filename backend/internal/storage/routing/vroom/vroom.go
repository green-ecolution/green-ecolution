package vroom

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log/slog"
	"net/http"
	"net/url"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

const (
	treeScale = 80 // how much water does a tree need
)

type VroomClientConfig struct {
	url           *url.URL
	client        *http.Client
	startPoint    []float64
	endPoint      []float64
	wateringPoint []float64
}

type VroomClientOption func(*VroomClientConfig)

type VroomClient struct {
	cfg VroomClientConfig
}

func WithClient(client *http.Client) VroomClientOption {
	return func(cfg *VroomClientConfig) {
		cfg.client = client
	}
}

func WithHostURL(hostURL *url.URL) VroomClientOption {
	slog.Debug("use vroom client with host url", "host_url", hostURL)
	return func(cfg *VroomClientConfig) {
		cfg.url = hostURL
	}
}

func WithStartPoint(startPoint []float64) VroomClientOption {
	slog.Debug("use vroom client with start point", "start_point", startPoint)
	return func(cfg *VroomClientConfig) {
		cfg.startPoint = startPoint
	}
}

func WithEndPoint(endPoint []float64) VroomClientOption {
	slog.Debug("use vroom client with end point", "end_point", endPoint)
	return func(cfg *VroomClientConfig) {
		cfg.endPoint = endPoint
	}
}

func WithWateringPoint(wateringPoint []float64) VroomClientOption {
	slog.Debug("use vroom client with watering point", "watering_point", wateringPoint)
	return func(cfg *VroomClientConfig) {
		cfg.wateringPoint = wateringPoint
	}
}

var defaultCfg = VroomClientConfig{
	client: http.DefaultClient,
}

func NewVroomClient(opts ...VroomClientOption) VroomClient {
	cfg := defaultCfg
	for _, opt := range opts {
		opt(&cfg)
	}
	return VroomClient{
		cfg: cfg,
	}
}

func (v *VroomClient) Send(ctx context.Context, reqBody *VroomReq) (*VroomResponse, error) {
	log := logger.GetLogger(ctx)
	var buf bytes.Buffer
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal vroom req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, v.cfg.url.String(), &buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := v.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to vroom service", "error", err)
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, err := io.ReadAll(resp.Body)
		if err == nil {
			log.Error("response from the vroom service with a not successful code", "status_code", resp.StatusCode, "body", body)
		} else {
			log.Error("response from the vroom service with a not successful code", "status_code", resp.StatusCode)
		}
		return nil, errors.New("failed to optimize route")
	}

	var vroomResp VroomResponse
	if err := json.NewDecoder(resp.Body).Decode(&vroomResp); err != nil {
		log.Error("failed to decode vroom response")
		return nil, errors.New("failed to optimize route")
	}

	if vroomResp.Code != 0 {
		log.Error("vroom returned error", "vroom_error", vroomResp.Error)
		return nil, errors.New("failed to optimize route")
	}

	return &vroomResp, nil
}

func (v *VroomClient) OptimizeRoute(ctx context.Context, vehicle *entities.Vehicle, cluster []*entities.TreeCluster) (*VroomResponse, error) {
	log := logger.GetLogger(ctx)
	vroomVehicle, err := v.toVroomVehicle(vehicle)
	if err != nil {
		if errors.Is(err, storage.ErrUnknownVehicleType) {
			log.Error("unknown vehicle type. please specify vehicle type", "error", err, "vehicle_type", vehicle.Type)
		}

		return nil, err
	}

	shipments := v.toVroomShipments(cluster)
	req := &VroomReq{
		Vehicles:  []VroomVehicle{*vroomVehicle},
		Shipments: shipments,
	}

	resp, err := v.Send(ctx, req)
	if err != nil {
		return nil, err
	}

	return resp, nil
}

func (v *VroomClient) toVroomShipments(cluster []*entities.TreeCluster) []VroomShipments {
	// ignore tree cluster with empty coordinates
	filteredClusters := utils.Filter(cluster, func(c *entities.TreeCluster) bool {
		return c.Longitude != nil && c.Latitude != nil
	})

	nextID := int32(0)
	return utils.Map(filteredClusters, func(c *entities.TreeCluster) VroomShipments {
		shipment := VroomShipments{
			Amount: []int32{int32(len(c.Trees) * treeScale)},
			Pickup: VroomShipmentStep{
				ID:       nextID,
				Location: v.cfg.wateringPoint,
			},
			Delivery: VroomShipmentStep{
				Description: c.Name,
				ID:          nextID + 1,
				Location:    []float64{*c.Longitude, *c.Latitude},
			},
		}

		nextID += 2
		return shipment
	})
}

func (v *VroomClient) toVroomVehicle(vehicle *entities.Vehicle) (*VroomVehicle, error) {
	vehicleType, err := v.toVehicleType(vehicle.Type)
	if err != nil {
		return nil, err
	}

	fmt.Println("vehicle capacity", int32(vehicle.WaterCapacity))
	return &VroomVehicle{
		ID:          vehicle.ID,
		Description: vehicle.Description,
		Profile:     vehicleType,
		Start:       v.cfg.startPoint,
		End:         v.cfg.endPoint,
		Capacity:    []int32{int32(vehicle.WaterCapacity)}, // vroom don't accept floats
	}, nil
}

func (v *VroomClient) toVehicleType(vehicle entities.VehicleType) (string, error) {
	if vehicle == entities.VehicleTypeUnknown {
		return "", storage.ErrUnknownVehicleType
	}

	return "auto", nil // It doesn't matter which type is used here
}
