package ors

import (
	"context"
	"encoding/json"
	"io"
	"net/http"
	"net/http/httptest"
	"net/url"
	"testing"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func newTestOrsClient(t *testing.T, serverURL string) OrsClient {
	t.Helper()
	u, err := url.Parse(serverURL)
	require.NoError(t, err)
	return NewOrsClient(
		WithHostURL(u),
	)
}

func TestDirectionsGeoJSON(t *testing.T) {
	t.Run("should return GeoJSON on 200 OK", func(t *testing.T) {
		// given
		geoJSON := entities.GeoJSON{
			Type: entities.FeatureCollection,
			Features: []entities.GeoJSONFeature{
				{Type: entities.Feature},
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(geoJSON))
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{
			Coordinates: [][]float64{{9.0, 48.0}, {9.1, 48.1}},
		}

		// when
		result, err := client.DirectionsGeoJSON(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		assert.Equal(t, entities.FeatureCollection, result.Type)
		assert.Len(t, result.Features, 1)
	})

	t.Run("should send request to correct profile path", func(t *testing.T) {
		// given
		var receivedPath string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedPath = r.URL.Path
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(entities.GeoJSON{}))
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		_, err := client.DirectionsGeoJSON(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		assert.Equal(t, "/v2/directions/driving-car/geojson", receivedPath)
	})

	t.Run("should set Content-Type header", func(t *testing.T) {
		// given
		var receivedContentType string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedContentType = r.Header.Get("Content-Type")
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(entities.GeoJSON{}))
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		_, err := client.DirectionsGeoJSON(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		assert.Equal(t, "application/json", receivedContentType)
	})

	t.Run("should return error on non-200 status", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		result, err := client.DirectionsGeoJSON(context.Background(), "driving-car", req)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "response not successful")
	})
}

func TestDirectionsRawGpx(t *testing.T) {
	t.Run("should return ReadCloser on 200 OK", func(t *testing.T) {
		// given
		gpxContent := `<?xml version="1.0" encoding="UTF-8"?><gpx></gpx>`
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/gpx+xml")
			_, err := w.Write([]byte(gpxContent))
			require.NoError(t, err)
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}, {9.1, 48.1}}}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		defer result.Close()
		body, err := io.ReadAll(result)
		require.NoError(t, err)
		assert.Equal(t, gpxContent, string(body))
	})

	t.Run("should send request to correct gpx endpoint", func(t *testing.T) {
		// given
		var receivedPath string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedPath = r.URL.Path
			_, err := w.Write([]byte("<gpx/>"))
			require.NoError(t, err)
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		result.Close()
		assert.Equal(t, "/v2/directions/driving-car/gpx", receivedPath)
	})

	t.Run("should return error on non-200 status", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusBadGateway)
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), "driving-car", req)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "response not successful")
	})
}

func TestDirectionsJSON(t *testing.T) {
	t.Run("should return OrsResponse on 200 OK", func(t *testing.T) {
		// given
		orsResp := OrsResponse{
			Routes: []Route{
				{Summary: Summary{Distance: 1500.0, Duration: 600.0}},
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(orsResp))
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}, {9.1, 48.1}}}

		// when
		result, err := client.DirectionsJSON(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		require.Len(t, result.Routes, 1)
		assert.Equal(t, 1500.0, result.Routes[0].Summary.Distance)
		assert.Equal(t, 600.0, result.Routes[0].Summary.Duration)
	})

	t.Run("should send request to correct json endpoint", func(t *testing.T) {
		// given
		var receivedPath string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedPath = r.URL.Path
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(OrsResponse{}))
		}))
		defer server.Close()

		client := newTestOrsClient(t, server.URL)
		req := &OrsDirectionRequest{Coordinates: [][]float64{{9.0, 48.0}}}

		// when
		_, err := client.DirectionsJSON(context.Background(), "driving-car", req)

		// then
		require.NoError(t, err)
		assert.Equal(t, "/v2/directions/driving-car/json", receivedPath)
	})
}
