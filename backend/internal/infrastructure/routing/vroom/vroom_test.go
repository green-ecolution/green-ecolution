package vroom

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/green-ecolution/green-ecolution/backend/internal/domain/shared"
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
	testWaterCapacity      = shared.MustNewWaterCapacity(5000.0)
	testClusterCoordinates = []shared.Coordinate{
		shared.MustNewCoordinate(48.2, 9.2),
		shared.MustNewCoordinate(48.3, 9.3),
	}
	testTreeCounts = []int{2, 1}
)

func TestToVroomVehicle(t *testing.T) {
	t.Run("should convert water capacity to vroom vehicle", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithStartPoint([]float64{9.0, 48.0}),
			WithEndPoint([]float64{9.1, 48.1}),
		)

		// when
		result := client.toVroomVehicle(testWaterCapacity)

		// then
		assert.Equal(t, int32(1), result.ID)
		assert.Equal(t, "auto", result.Profile)
		assert.Equal(t, []float64{9.0, 48.0}, []float64(result.Start))
		assert.Equal(t, []float64{9.1, 48.1}, []float64(result.End))
		assert.Equal(t, []int32{5000}, result.Capacity)
	})

	t.Run("should convert float water capacity to int32", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithStartPoint([]float64{9.0, 48.0}),
			WithEndPoint([]float64{9.1, 48.1}),
		)
		wc := shared.MustNewWaterCapacity(3500.7)

		// when
		result := client.toVroomVehicle(wc)

		// then
		assert.Equal(t, []int32{3500}, result.Capacity)
	})
}

func TestToVroomShipments(t *testing.T) {
	t.Run("should convert coordinates and tree counts to shipments", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments(testClusterCoordinates, testTreeCounts)

		// then
		require.Len(t, result, 2)
		assert.Equal(t, []int32{160}, result[0].Amount) // 2 trees * 80
		assert.Equal(t, []int32{80}, result[1].Amount)  // 1 tree * 80
	})

	t.Run("should set pickup location to watering point", func(t *testing.T) {
		// given
		wateringPoint := []float64{9.05, 48.05}
		client := NewVroomClient(
			WithWateringPoint(wateringPoint),
		)

		// when
		result := client.toVroomShipments(testClusterCoordinates, testTreeCounts)

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
		result := client.toVroomShipments(testClusterCoordinates, testTreeCounts)

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
		result := client.toVroomShipments(testClusterCoordinates, testTreeCounts)

		// then
		assert.Equal(t, int32(0), result[0].Pickup.ID)
		assert.Equal(t, int32(1), result[0].Delivery.ID)
		assert.Equal(t, int32(2), result[1].Pickup.ID)
		assert.Equal(t, int32(3), result[1].Delivery.ID)
	})

	t.Run("should return empty shipments for empty coordinates", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when
		result := client.toVroomShipments([]shared.Coordinate{}, []int{})

		// then
		assert.Empty(t, result)
	})

	t.Run("should handle missing tree counts gracefully", func(t *testing.T) {
		// given
		client := NewVroomClient(
			WithWateringPoint([]float64{9.05, 48.05}),
		)

		// when - more coordinates than tree counts
		result := client.toVroomShipments(testClusterCoordinates, []int{2})

		// then
		require.Len(t, result, 2)
		assert.Equal(t, []int32{160}, result[0].Amount) // 2 trees * 80
		assert.Equal(t, []int32{0}, result[1].Amount)   // 0 trees (no count provided)
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
		result, err := client.OptimizeRoute(context.Background(), testWaterCapacity, testClusterCoordinates, testTreeCounts)

		// then
		require.NoError(t, err)
		assert.Equal(t, int32(0), result.Code)
		assert.Len(t, result.Routes, 1)
	})

	t.Run("should return error when vroom returns error", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		client := newTestVroomClient(t, server.URL)

		// when
		result, err := client.OptimizeRoute(context.Background(), testWaterCapacity, testClusterCoordinates, testTreeCounts)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}
