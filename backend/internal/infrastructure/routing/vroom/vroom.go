package vroom

import (
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"io"
	"log/slog"
	"net/http"
	"net/url"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
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

func (v *VroomClient) OptimizeRoute(ctx context.Context, waterCapacity shared.WaterCapacity, clusterCoordinates []shared.Coordinate, treeCounts []int) (*VroomResponse, error) {
	log := logger.GetLogger(ctx)
	vroomVehicle := v.toVroomVehicle(waterCapacity)
	shipments := v.toVroomShipments(clusterCoordinates, treeCounts)
	req := &VroomReq{
		Vehicles:  []VroomVehicle{vroomVehicle},
		Shipments: shipments,
	}

	resp, err := v.Send(ctx, req)
	if err != nil {
		log.Error("failed to optimize route", "error", err)
		return nil, err
	}

	return resp, nil
}

func (v *VroomClient) toVroomShipments(clusterCoordinates []shared.Coordinate, treeCounts []int) []VroomShipments {
	shipments := make([]VroomShipments, 0, len(clusterCoordinates))
	nextID := int32(0)
	for i, coord := range clusterCoordinates {
		treeCount := 0
		if i < len(treeCounts) {
			treeCount = treeCounts[i]
		}
		shipment := VroomShipments{
			Amount: []int32{int32(treeCount * treeScale)},
			Pickup: VroomShipmentStep{
				ID:       nextID,
				Location: v.cfg.wateringPoint,
			},
			Delivery: VroomShipmentStep{
				ID:       nextID + 1,
				Location: []float64{coord.Longitude(), coord.Latitude()},
			},
		}
		nextID += 2
		shipments = append(shipments, shipment)
	}
	return shipments
}

func (v *VroomClient) toVroomVehicle(waterCapacity shared.WaterCapacity) VroomVehicle {
	return VroomVehicle{
		ID:       1,
		Profile:  "auto",
		Start:    v.cfg.startPoint,
		End:      v.cfg.endPoint,
		Capacity: []int32{int32(waterCapacity.Liters())},
	}
}
