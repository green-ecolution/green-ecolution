package openrouteservice

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
	"github.com/green-ecolution/green-ecolution/backend/internal/storage"
	"github.com/green-ecolution/green-ecolution/backend/internal/storage/routing/openrouteservice/ors"
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

	testOrsGeoJSON = entities.GeoJSON{
		Type: entities.FeatureCollection,
		Features: []entities.GeoJSONFeature{
			{
				Type: entities.Feature,
				Geometry: entities.GeoJSONGeometry{
					Type:        entities.LineString,
					Coordinates: [][]float64{{9.0, 48.0}, {9.1, 48.1}},
				},
			},
		},
	}

	testOrsResponse = ors.OrsResponse{
		Routes: []ors.Route{
			{
				Summary: ors.Summary{Distance: 15000.0, Duration: 1800.0},
			},
		},
	}
)

func newTestOrsRouteRepo(t *testing.T, vroomHandler, orsHandler http.HandlerFunc) *RouteRepo {
	t.Helper()
	vroomServer := httptest.NewServer(vroomHandler)
	t.Cleanup(vroomServer.Close)
	orsServer := httptest.NewServer(orsHandler)
	t.Cleanup(orsServer.Close)

	cfg := &RouteRepoConfig{
		routing: config.RoutingConfig{
			StartPoint:    testStartPoint,
			EndPoint:      testEndPoint,
			WateringPoint: testWateringPoint,
			Ors: config.RoutingOrsConfig{
				Host: orsServer.URL,
				Optimization: config.RoutingOrsOptimizationConfig{
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

func TestToOrsVehicleType(t *testing.T) {
	t.Run("should return driving-car for transporter", func(t *testing.T) {
		// given
		repo := &RouteRepo{}

		// when
		result, err := repo.toOrsVehicleType(entities.VehicleTypeTransporter)

		// then
		require.NoError(t, err)
		assert.Equal(t, "driving-car", result)
	})

	t.Run("should return driving-car for trailer", func(t *testing.T) {
		// given
		repo := &RouteRepo{}

		// when
		result, err := repo.toOrsVehicleType(entities.VehicleTypeTrailer)

		// then
		require.NoError(t, err)
		assert.Equal(t, "driving-car", result)
	})

	t.Run("should return error for unknown type", func(t *testing.T) {
		// given
		repo := &RouteRepo{}

		// when
		_, err := repo.toOrsVehicleType(entities.VehicleTypeUnknown)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
	})
}

func TestPrepareOrsRoute(t *testing.T) {
	t.Run("should return OrsDirectionRequest with coordinates", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called in prepareOrsRoute")
			},
		)

		// when
		optimized, route, err := repo.prepareOrsRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.NotNil(t, optimized)
		assert.Equal(t, "m", route.Units)
		assert.Equal(t, "de-de", route.Language)
		assert.NotEmpty(t, route.Coordinates)
	})

	t.Run("should return error for empty routes", func(t *testing.T) {
		// given
		emptyResp := vroom.VroomResponse{Code: 0, Routes: []vroom.VroomRoutes{}}
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(emptyResp))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called")
			},
		)

		// when
		_, _, err := repo.prepareOrsRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "empty routes")
	})

	t.Run("should reduce consecutive pickups to fewer coordinates", func(t *testing.T) {
		// given
		vroomRespWithConsecutivePickups := vroom.VroomResponse{
			Code: 0,
			Routes: []vroom.VroomRoutes{
				{
					Vehicle: 1,
					Steps: []vroom.VroomRouteStep{
						{Type: "start", Location: []float64{9.0, 48.0}},
						{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{160}},
						{Type: "pickup", Location: []float64{9.05, 48.05}, Load: []int32{240}},
						{Type: "delivery", Location: []float64{9.2, 48.2}},
						{Type: "end", Location: []float64{9.1, 48.1}},
					},
				},
			},
		}

		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(vroomRespWithConsecutivePickups))
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called")
			},
		)

		// when
		_, route, err := repo.prepareOrsRoute(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		// 5 raw steps, but 2 consecutive pickups reduced to 1 → 4 coordinates
		assert.Len(t, route.Coordinates, 4)
	})
}

func TestOrsGenerateRoute(t *testing.T) {
	t.Run("should return GeoJSON with metadata on success", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testOrsGeoJSON))
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

	t.Run("should return ErrUnknownVehicleType for unknown vehicle", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("vroom should not be called")
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called")
			},
		)
		vehicle := &entities.Vehicle{Type: entities.VehicleTypeUnknown}

		// when
		result, err := repo.GenerateRoute(context.Background(), vehicle, testClusters)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
		assert.Nil(t, result)
	})

	t.Run("should return error when vroom fails", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.WriteHeader(http.StatusInternalServerError)
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called")
			},
		)

		// when
		result, err := repo.GenerateRoute(context.Background(), testVehicle, testClusters)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestOrsGenerateRawGpxRoute(t *testing.T) {
	t.Run("should return ReadCloser with GPX on success", func(t *testing.T) {
		// given
		gpxContent := `<?xml version="1.0" encoding="UTF-8"?><gpx></gpx>`
		repo := newTestOrsRouteRepo(t,
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

	t.Run("should return error for unknown vehicle type", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("vroom should not be called")
			},
			func(w http.ResponseWriter, r *http.Request) {
				t.Fatal("ORS should not be called")
			},
		)
		vehicle := &entities.Vehicle{Type: entities.VehicleTypeUnknown}

		// when
		result, err := repo.GenerateRawGpxRoute(context.Background(), vehicle, testClusters)

		// then
		assert.ErrorIs(t, err, storage.ErrUnknownVehicleType)
		assert.Nil(t, result)
	})
}

func TestOrsGenerateRouteInformation(t *testing.T) {
	t.Run("should return RouteMetadata on success", func(t *testing.T) {
		// given
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testOrsResponse))
			},
		)

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, 15000.0, result.Distance)
		assert.Equal(t, time.Duration(1800.0*float64(time.Second)), result.Time)
		assert.Equal(t, int32(2), result.Refills) // 2 pickups in testVroomResponse
	})

	t.Run("should return zero distance and duration for empty ORS routes", func(t *testing.T) {
		// given
		emptyOrsResp := ors.OrsResponse{Routes: []ors.Route{}}
		repo := newTestOrsRouteRepo(t,
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(testVroomResponse))
			},
			func(w http.ResponseWriter, r *http.Request) {
				w.Header().Set("Content-Type", "application/json")
				require.NoError(t, json.NewEncoder(w).Encode(emptyOrsResp))
			},
		)

		// when
		result, err := repo.GenerateRouteInformation(context.Background(), testVehicle, testClusters)

		// then
		require.NoError(t, err)
		assert.Equal(t, float64(0), result.Distance)
		assert.Equal(t, time.Duration(0), result.Time)
	})
}
