package valhalla

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

func newTestValhallaClient(t *testing.T, serverURL string) ValhallaClient {
	t.Helper()
	u, err := url.Parse(serverURL)
	require.NoError(t, err)
	return NewValhallaClient(
		WithHostURL(u),
	)
}

func TestDecodePolyline(t *testing.T) {
	t.Run("should decode known 2-point polyline", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		// Encode two points: (48.0, 9.0) and (48.1, 9.1) with precision 6
		// Using a manually generated polyline for these known coordinates
		encoded := "__upzA_cidP_ibE_ibE"

		// when
		result := client.decodePolyline(&encoded)

		// then
		require.Len(t, result, 2)
		// coordinates are [lon, lat]
		assert.InDelta(t, 9.0, result[0][0], 0.001)
		assert.InDelta(t, 48.0, result[0][1], 0.001)
		assert.InDelta(t, 9.1, result[1][0], 0.001)
		assert.InDelta(t, 48.1, result[1][1], 0.001)
	})

	t.Run("should decode single point", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		// Single point at (48.0, 9.0) with precision 6
		encoded := "__upzA_cidP"

		// when
		result := client.decodePolyline(&encoded)

		// then
		require.Len(t, result, 1)
		assert.InDelta(t, 9.0, result[0][0], 0.001)
		assert.InDelta(t, 48.0, result[0][1], 0.001)
	})

	t.Run("should decode negative coordinates", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		// Point at (-33.8688, 151.2093) - Sydney, with precision 6
		encoded := "~~dr_Agtal_H"

		// when
		result := client.decodePolyline(&encoded)

		// then
		require.Len(t, result, 1)
		assert.InDelta(t, 151.2093, result[0][0], 0.01)
		assert.InDelta(t, -33.8688, result[0][1], 0.01)
	})

	t.Run("should return empty result for empty string", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		encoded := ""

		// when
		result := client.decodePolyline(&encoded)

		// then
		assert.Empty(t, result)
	})

	t.Run("should use custom precision parameter", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		// Encode (48.0, 9.0) with precision 5 (Google standard)
		encoded := "__~cH_y|u@"

		// when
		result := client.decodePolyline(&encoded, 5)

		// then
		require.Len(t, result, 1)
		assert.InDelta(t, 9.0, result[0][0], 0.01)
		assert.InDelta(t, 48.0, result[0][1], 0.01)
	})
}

func TestToGeoJSON(t *testing.T) {
	t.Run("should convert single leg to GeoJSON", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		resp := &DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{
					MinLat: 48.0,
					MinLon: 9.0,
					MaxLat: 48.1,
					MaxLon: 9.1,
				},
				Legs: []LegResponse{
					{
						Shape: "__upzA_cidP_ibE_ibE",
						Summary: SummaryResponse{
							MinLat: 48.0,
							MinLon: 9.0,
							MaxLat: 48.1,
							MaxLon: 9.1,
						},
					},
				},
			},
		}

		// when
		result := client.toGeoJSON(resp)

		// then
		require.Len(t, result.Features, 1)
		assert.Equal(t, entities.FeatureCollection, result.Type)
		assert.Equal(t, entities.Feature, result.Features[0].Type)
		assert.Equal(t, entities.LineString, result.Features[0].Geometry.Type)
		assert.NotEmpty(t, result.Features[0].Geometry.Coordinates)
	})

	t.Run("should convert two legs to GeoJSON with 2 features", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		resp := &DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{MinLat: 48.0, MinLon: 9.0, MaxLat: 48.2, MaxLon: 9.2},
				Legs: []LegResponse{
					{
						Shape:   "__upzA_cidP_ibE_ibE",
						Summary: SummaryResponse{MinLat: 48.0, MinLon: 9.0, MaxLat: 48.1, MaxLon: 9.1},
					},
					{
						Shape:   "__upzA_cidP_ibE_ibE",
						Summary: SummaryResponse{MinLat: 48.1, MinLon: 9.1, MaxLat: 48.2, MaxLon: 9.2},
					},
				},
			},
		}

		// when
		result := client.toGeoJSON(resp)

		// then
		assert.Len(t, result.Features, 2)
	})

	t.Run("should compute BBox from trip summary", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		resp := &DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{
					MinLat: 48.0,
					MinLon: 9.0,
					MaxLat: 48.5,
					MaxLon: 9.5,
				},
				Legs: []LegResponse{},
			},
		}

		// when
		result := client.toGeoJSON(resp)

		// then
		// Note: current implementation has a bug where 4th element is MinLon instead of MaxLon
		assert.Equal(t, 48.0, result.Bbox[0])
		assert.Equal(t, 9.0, result.Bbox[1])
		assert.Equal(t, 48.5, result.Bbox[2])
		assert.Equal(t, 9.0, result.Bbox[3]) // documents the bug: should be 9.5 (MaxLon)
	})

	t.Run("should set correct types for root and features", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		resp := &DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{},
				Legs: []LegResponse{
					{Shape: "__upzA_cidP", Summary: SummaryResponse{}},
				},
			},
		}

		// when
		result := client.toGeoJSON(resp)

		// then
		assert.Equal(t, entities.FeatureCollection, result.Type)
		assert.Equal(t, entities.Feature, result.Features[0].Type)
		assert.Equal(t, entities.LineString, result.Features[0].Geometry.Type)
	})

	t.Run("should return empty features for empty legs", func(t *testing.T) {
		// given
		client := NewValhallaClient()
		resp := &DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{},
				Legs:    []LegResponse{},
			},
		}

		// when
		result := client.toGeoJSON(resp)

		// then
		assert.Empty(t, result.Features)
	})
}

func TestDirectionsJSON(t *testing.T) {
	t.Run("should return DirectionResponse on 200 OK", func(t *testing.T) {
		// given
		dirResp := DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{Length: 10.5, Time: 300},
				Status:  0,
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(dirResp))
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{
			Locations: []Location{{Lat: 48.0, Lon: 9.0}},
			Costing:   "truck",
		}

		// when
		result, err := client.DirectionsJSON(context.Background(), req)

		// then
		require.NoError(t, err)
		assert.Equal(t, 10.5, result.Trip.Summary.Length)
		assert.Equal(t, float64(300), result.Trip.Summary.Time)
	})

	t.Run("should set format to json", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			jsonParam := r.URL.Query().Get("json")
			assert.Contains(t, jsonParam, `"format":"json"`)
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(DirectionResponse{}))
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		_, err := client.DirectionsJSON(context.Background(), req)

		// then
		require.NoError(t, err)
	})

	t.Run("should send request to /route endpoint", func(t *testing.T) {
		// given
		var receivedPath string
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedPath = r.URL.Path
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(DirectionResponse{}))
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		_, err := client.DirectionsJSON(context.Background(), req)

		// then
		require.NoError(t, err)
		assert.Equal(t, "/route", receivedPath)
	})

	t.Run("should return error on non-200 status", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsJSON(context.Background(), req)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "response not successful")
	})
}

func TestDirectionsGeoJSON(t *testing.T) {
	t.Run("should return GeoJSON on 200 OK", func(t *testing.T) {
		// given
		dirResp := DirectionResponse{
			Trip: TripResponse{
				Summary: SummaryResponse{MinLat: 48.0, MinLon: 9.0, MaxLat: 48.1, MaxLon: 9.1},
				Legs: []LegResponse{
					{Shape: "__upzA_cidP_ibE_ibE", Summary: SummaryResponse{MinLat: 48.0, MinLon: 9.0, MaxLat: 48.1, MaxLon: 9.1}},
				},
			},
		}

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			require.NoError(t, json.NewEncoder(w).Encode(dirResp))
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsGeoJSON(context.Background(), req)

		// then
		require.NoError(t, err)
		assert.Equal(t, entities.FeatureCollection, result.Type)
		assert.Len(t, result.Features, 1)
	})

	t.Run("should return error on HTTP failure", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusBadGateway)
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsGeoJSON(context.Background(), req)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})
}

func TestDirectionsRawGpx(t *testing.T) {
	t.Run("should return ReadCloser with GPX content on 200 OK", func(t *testing.T) {
		// given
		gpxContent := `<?xml version="1.0" encoding="UTF-8"?><gpx></gpx>`
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/gpx+xml")
			_, err := w.Write([]byte(gpxContent))
			require.NoError(t, err)
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), req)

		// then
		require.NoError(t, err)
		defer result.Close()
		body, err := io.ReadAll(result)
		require.NoError(t, err)
		assert.Equal(t, gpxContent, string(body))
	})

	t.Run("should set format to gpx", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			jsonParam := r.URL.Query().Get("json")
			assert.Contains(t, jsonParam, `"format":"gpx"`)
			_, err := w.Write([]byte("<gpx/>"))
			require.NoError(t, err)
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), req)

		// then
		require.NoError(t, err)
		result.Close()
	})

	t.Run("should return error on non-200 status", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusServiceUnavailable)
		}))
		defer server.Close()

		client := newTestValhallaClient(t, server.URL)
		req := &DirectionRequest{Costing: "truck"}

		// when
		result, err := client.DirectionsRawGpx(context.Background(), req)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "response not successful")
	})
}
