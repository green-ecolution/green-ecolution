package vroom

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestVroomClient(t *testing.T, serverURL string) VroomClient {
	t.Helper()
	u, err := url.Parse(serverURL)
	require.NoError(t, err)
	return NewVroomClient(
		WithHostURL(u),
		WithStartPoint([]float64{9.0, 48.0}),
		WithEndPoint([]float64{9.1, 48.1}),
		WithWateringPoint([]float64{9.05, 48.05}),
	)
}

var (
	testVehicle = &entities.Vehicle{
		ID:            1,
		Description:   "Test Vehicle",
		WaterCapacity: 5000.0,
		Type:          entities.VehicleTypeTransporter,
		Width:         2.5,
		Height:        3.0,
		Length:        6.0,
		Weight:        7.5,
	}

	testClusters = []*entities.TreeCluster{
		{
			ID:        1,
			Name:      "Cluster A",
			Longitude: utils.P(9.2),
			Latitude:  utils.P(48.2),
			Trees:     []*entities.Tree{{}, {}},
		},
		{
			ID:        2,
			Name:      "Cluster B",
			Longitude: utils.P(9.3),
			Latitude:  utils.P(48.3),
			Trees:     []*entities.Tree{{}},
		},
	}
)

func TestToVehicleType(t *testing.T) {
	t.Run("should return auto for transporter", func(t *testing.T) {
		// given
		client := NewVroomClient()

		// when
		result, err := client.toVehicleType(entities.VehicleTypeTransporter)

		// then
		require.NoError(t, err)
		assert.Equal(t, "auto", result)
	})

	t.Run("should return auto for trailer", func(t *testing.T) {
		// given
		client := NewVroomClient()

		// when
		result, err := client.toVehicleType(entities.VehicleTypeTrailer)

		// then
		require.NoError(t, err)
		assert.Equal(t, "auto", result)
	})

	t.Run("should return error for unknown type", func(t *testing.T) {
		// given
		client := NewVroomClient()

		// when
		_, err := client.toVehicleType(entities.VehicleTypeUnknown)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
	})
}

func TestToVroomVehicle(t *testing.T) {
	t.Run("should convert valid vehicle", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithStartPoint([]float64{9.0, 48.0}),
			WithEndPoint([]float64{9.1, 48.1}),
		)

		// when
		result, err := client.toVroomVehicle(testVehicle)

		// then
		require.NoError(t, err)
		assert.Equal(t, int32(1), result.ID)
		assert.Equal(t, "Test Vehicle", result.Description)
		assert.Equal(t, "auto", result.Profile)
		assert.Equal(t, []float64{9.0, 48.0}, []float64(result.Start))
		assert.Equal(t, []float64{9.1, 48.1}, []float64(result.End))
		assert.Equal(t, []int32{5000}, result.Capacity)
	})

	t.Run("should return error for unknown vehicle type", func(t *testing.T) {
		// given
		client := NewVroomClient()
		vehicle := &entities.Vehicle{Type: entities.VehicleTypeUnknown}

		// when
		_, err := client.toVroomVehicle(vehicle)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
	})

	t.Run("should convert float water capacity to int32", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithStartPoint([]float64{9.0, 48.0}),
			WithEndPoint([]float64{9.1, 48.1}),
		)
		vehicle := &entities.Vehicle{
			ID:            2,
			WaterCapacity: 3500.7,
			Type:          entities.VehicleTypeTransporter,
		}

		// when
		result, err := client.toVroomVehicle(vehicle)

		// then
		require.NoError(t, err)
		assert.Equal(t, []int32{3500}, result.Capacity)
	})
}

func TestToVroomShipments(t *testing.T) {
	t.Run("should convert clusters to shipments", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments(testClusters)

		// then
		require.Len(t, result, 2)
		assert.Equal(t, []int32{160}, result[0].Amount) // 2 trees * 80
		assert.Equal(t, []int32{80}, result[1].Amount)  // 1 tree * 80
	})

	t.Run("should filter clusters with nil coordinates", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)
		clusters := []*entities.TreeCluster{
			{ID: 1, Name: "A", Longitude: utils.P(9.2), Latitude: utils.P(48.2), Trees: []*entities.Tree{{}}},
			{ID: 2, Name: "B", Longitude: nil, Latitude: nil, Trees: []*entities.Tree{{}}},
			{ID: 3, Name: "C", Longitude: utils.P(9.3), Latitude: nil, Trees: []*entities.Tree{{}}},
		}

		// when
		result := client.toVroomShipments(clusters)

		// then
		assert.Len(t, result, 1)
		assert.Equal(t, "A", result[0].Delivery.Description)
	})

	t.Run("should set pickup location to watering point", func(t *testing.T) {
		// given
		wateringPoint := []float64{9.05, 48.05}
		client := NewVroomClient(
			WithWateringPoint(wateringPoint),
		)

		// when
		result := client.toVroomShipments(testClusters)

		// then
		assert.Equal(t, wateringPoint, result[0].Pickup.Location)
		assert.Equal(t, wateringPoint, result[1].Pickup.Location)
	})

	t.Run("should set delivery location to cluster coordinates", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments(testClusters)

		// then
		assert.Equal(t, []float64{9.2, 48.2}, result[0].Delivery.Location)
		assert.Equal(t, []float64{9.3, 48.3}, result[1].Delivery.Location)
	})

	t.Run("should assign incrementing IDs starting at 0", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments(testClusters)

		// then
		assert.Equal(t, int32(0), result[0].Pickup.ID)
		assert.Equal(t, int32(1), result[0].Delivery.ID)
		assert.Equal(t, int32(2), result[1].Pickup.ID)
		assert.Equal(t, int32(3), result[1].Delivery.ID)
	})

	t.Run("should return empty shipments for empty clusters", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments([]*entities.TreeCluster{})

		// then
		assert.Empty(t, result)
	})

	t.Run("should set delivery description to cluster name", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments(testClusters)

		// then
		assert.Equal(t, "Cluster A", result[0].Delivery.Description)
		assert.Equal(t, "Cluster B", result[1].Delivery.Description)
	})
}

func TestVroomClient_Send(t *testing.T) {
	t.Run("should return VroomResponse on 200 OK", func(t *testing.T) {
		// given
		vroomResp := VroomResponse{
			Code: 0,
			Routes: []VroomRoutes{
				{Vehicle: 1, Steps: []VroomRouteStep{{Type: "start", Location: []float64{9.0, 48.0}}}},
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			assert.Equal(t, http.MethodPost, r.Method)
			assert.Equal(t, "application/json", r.Header.Get("Content-Type"))
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(vroomResp))
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.Send(context.Background(), &VroomReq{})

		// then
		require.NoError(t, err)
		assert.Equal(t, int32(0), result.Code)
		assert.Len(t, result.Routes, 1)
	})

	t.Run("should return error on non-200 status", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.Send(context.Background(), &VroomReq{})

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to optimize route")
	})

	t.Run("should return error on invalid JSON response", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			_, err := w.Write([]byte("invalid json"))
			require.NoError(t, err)
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.Send(context.Background(), &VroomReq{})

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when vroom code is not 0", func(t *testing.T) {
		// given
		errMsg := "internal error"
		vroomResp := VroomResponse{
			Code:  1,
			Error: &errMsg,
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(vroomResp))
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.Send(context.Background(), &VroomReq{})

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "failed to optimize route")
	})

	t.Run("should set Content-Type header", func(t *testing.T) {
		// given
		var receivedContentType string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedContentType = r.Header.Get("Content-Type")
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(VroomResponse{Code: 0}))
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		_, err := client.Send(context.Background(), &VroomReq{})

		// then
		require.NoError(t, err)
		assert.Equal(t, "application/json", receivedContentType)
	})
}

func TestVroomClient_OptimizeRoute(t *testing.T) {
	t.Run("should return VroomResponse for valid input", func(t *testing.T) {
		// given
		vroomResp := VroomResponse{
			Code: 0,
			Routes: []VroomRoutes{
				{Vehicle: 1, Steps: []VroomRouteStep{{Type: "start"}}},
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			var req VroomReq
			require.NoError(t, json.NewDecoder(r.Body).Decode(&req))
			assert.Len(t, req.Vehicles, 1)
			assert.Equal(t, "auto", req.Vehicles[0].Profile)
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(vroomResp))
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.OptimizeRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, int32(0), result.Code)
		assert.Len(t, result.Routes, 1)
	})

	t.Run("should return ErrUnknownVehicleType for unknown vehicle type", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			t.Fatal("should not reach HTTP call")
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)
		vehicle := &entities.Vehicle{Type: entities.VehicleTypeUnknown}

		// when
		result, err := client.OptimizeRoute(context.Background(), vehicle, testClusters)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
		assert.Nil(t, result)
	})

	t.Run("should return error when vroom returns error", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.OptimizeRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should filter clusters with nil coordinates in shipments", func(t *testing.T) {
		// given
		vroomResp := VroomResponse{Code: 0, Routes: []VroomRoutes{{Vehicle: 1}}}

		var receivedReq VroomReq
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			require.NoError(t, json.NewDecoder(r.Body).Decode(&receivedReq))
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(vroomResp))
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)
		clusters := []*entities.TreeCluster{
			{ID: 1, Longitude: utils.P(9.2), Latitude: utils.P(48.2), Trees: []*entities.Tree{{}}},
			{ID: 2, Longitude: nil, Latitude: nil, Trees: []*entities.Tree{{}}},
		}

		// when
		_, err := client.OptimizeRoute(context.Background(), testVehicle, clusters)

		// then
		require.NoError(t, err)
		assert.Len(t, receivedReq.Shipments, 1)
	})
}
