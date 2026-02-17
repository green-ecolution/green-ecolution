package valhalla

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/config"
	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/valhalla/valhalla"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/vroom"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var (
	testStartPoint    = []float64{9.0, 48.0}
	testEndPoint      = []float64{9.1, 48.1}
	testWateringPoint = []float64{9.05, 48.05}

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

	testVroomResponse = vroom.VroomResponse{
		Code: 0,
		Routes: []vroom.VroomRoutes{
			{
				Vehicle: 1,
				Steps: []vroom.VroomRouteStep{
					{Type: "start", Location: []float64{9.0, 48.0}, Load: []int32{0}},
					{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{160}},
					{Type: "delivery", Location: []float64{9.2, 48.2}, Load: []int32{0}},
					{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{80}},
					{Type: "delivery", Location: []float64{9.3, 48.3}, Load: []int32{0}},
					{Type: "end", Location: []float64{9.1, 48.1}, Load: []int32{0}},
				},
			},
		},
	}

	testValhallaResponse = valhalla.DirectionResponse{
		Trip: valhalla.TripResponse{
			Summary: valhalla.SummaryResponse{
				MinLat: 48.0, MinLon: 9.0, MaxLat: 48.3, MaxLon: 9.3,
				Length: 15.5,
				Time:   1200,
			},
			Legs: []valhalla.LegResponse{
				{
					Shape:   "__upzA_cidP_ibE_ibE",
					Summary: valhalla.SummaryResponse{MinLat: 48.0, MinLon: 9.0, MaxLat: 48.1, MaxLon: 9.1},
				},
			},
		},
	}
)

func newTestRouteRepo(t *testing.T, vroomHandler, valhallaHandler http.HandlerFunc) *RouteRepo {
	t.Helper()
	vroomServer := httptest.NewServer(vroomHandler)
	t.Cleanup(vroomServer.Close)
	valhallaServer := httptest.NewServer(valhallaHandler)
	t.Cleanup(valhallaServer.Close)

	cfg := &RouteRepoConfig{
		routing: config.RoutingConfig{
			StartPoint:    testStartPoint,
			EndPoint:      testEndPoint,
			WateringPoint: testWateringPoint,
			Valhalla: config.RoutingValhallaConfig{
				Host: valhallaServer.URL,
				Optimization: config.RoutingValhallaOptimizationConfig{
					Vroom: config.RoutingVroomConfig{
						Host: vroomServer.URL,
					},
				},
			},
		},
	}

	repo, err := NewRouteRepo(cfg)
	require.NoError(t, err)
	return repo
}

func TestPrepareRoute(t *testing.T) {
	t.Run("should return DirectionRequest with reduced locations", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called in prepareRoute")
			},
		)

		// when
		optimized, route, err := repo.prepareRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.NotNil(t, optimized)
		assert.Equal(t, "truck", route.Costing)
		// Steps: start, pickup, delivery, pickup, delivery, end → reduced: start, pickup, delivery, pickup, delivery, end (no consecutive pickups here)
		assert.Len(t, route.Locations, 6)
	})

	t.Run("should return error when vroom fails", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusInternalServerError)
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called")
			},
		)

		// when
		_, _, err := repo.prepareRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
	})

	t.Run("should return error for empty routes", func(t *testing.T) {
		// given
		emptyResp := vroom.VroomResponse{Code: 0, Routes: []vroom.VroomRoutes{}}
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(emptyResp))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called")
			},
		)

		// when
		_, _, err := repo.prepareRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "empty routes")
	})

	t.Run("should set vehicle dimensions in costing options", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called")
			},
		)

		// when
		_, route, err := repo.prepareRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		costingOpts := route.CostingOptions["truck"]
		assert.Equal(t, 2.5, costingOpts.Width)
		assert.Equal(t, 3.0, costingOpts.Height)
		assert.Equal(t, 6.0, costingOpts.Length)
		assert.Equal(t, 7.5, costingOpts.Weight)
	})
}

func TestGenerateRoute(t *testing.T) {
	t.Run("should return GeoJSON with metadata on success", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testValhallaResponse))
			},
		)

		// when
		result, err := repo.GenerateRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, entities.FeatureCollection, result.Type)
		assert.Equal(t, 9.0, result.Metadata.StartPoint.Longitude)
		assert.Equal(t, 48.0, result.Metadata.StartPoint.Latitude)
		assert.Equal(t, 9.1, result.Metadata.EndPoint.Longitude)
		assert.Equal(t, 48.1, result.Metadata.EndPoint.Latitude)
	})

	t.Run("should return error when vroom fails", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusInternalServerError)
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called")
			},
		)

		// when
		result, err := repo.GenerateRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should return error when valhalla fails", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusInternalServerError)
			},
		)

		// when
		result, err := repo.GenerateRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestGenerateRawGpxRoute(t *testing.T) {
	t.Run("should return ReadCloser with GPX on success", func(t *testing.T) {
		// given
		gpxContent := `<?xml version="1.0" encoding="UTF-8"?><gpx></gpx>`
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				_, err := w.Write([]byte(gpxContent))
				require.NoError(t, err)
			},
		)

		// when
		result, err := repo.GenerateRawGpxRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		defer result.Close()
		body, err := io.ReadAll(result)
		require.NoError(t, err)
		assert.Equal(t, gpxContent, string(body))
	})

	t.Run("should return error when vroom fails", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusInternalServerError)
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("valhalla should not be called")
			},
		)

		// when
		result, err := repo.GenerateRawGpxRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestGenerateRouteInformation(t *testing.T) {
	t.Run("should return RouteMetadata on success", func(t *testing.T) {
		// given
		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testValhallaResponse))
			},
		)

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, 15.5, result.Distance)
		assert.Equal(t, time.Duration(1200*float64(time.Second)), result.Time)
	})

	t.Run("should calculate refill count from reduced steps", func(t *testing.T) {
		// given
		vroomRespWithPickups := vroom.VroomResponse{
			Code: 0,
			Routes: []vroom.VroomRoutes{
				{
					Vehicle: 1,
					Steps: []vroom.VroomRouteStep{
						{Type: "start", Location: []float64{9.0, 48.0}},
						{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{160}},
						{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{240}},
						{Type: "delivery", Location: []float64{9.2, 48.2}, Load: []int32{0}},
						{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{80}},
						{Type: "delivery", Location: []float64{9.3, 48.3}, Load: []int32{0}},
						{Type: "end", Location: []float64{9.1, 48.1}},
					},
				},
			},
		}

		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(vroomRespWithPickups))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testValhallaResponse))
			},
		)

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		// Two consecutive pickups get reduced to 1, plus another pickup = 2 refills
		assert.Equal(t, int32(2), result.Refills)
	})

	t.Run("should convert time from seconds to duration", func(t *testing.T) {
		// given
		valhallaResp := valhalla.DirectionResponse{
			Trip: valhalla.TripResponse{
				Summary: valhalla.SummaryResponse{Time: 3600, Length: 50.0},
				Legs:    []valhalla.LegResponse{},
			},
		}

		repo := newTestRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(valhallaResp))
			},
		)

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, time.Hour, result.Time)
	})
}
