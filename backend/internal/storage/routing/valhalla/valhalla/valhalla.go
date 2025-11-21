package valhalla

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"math"
	"net/http"
	"net/url"
	"strings"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
	"github.com/green-ecolution/green-ecolution/backend/internal/utils"
)

type ValhallaClientConfig struct {
	url    *url.URL
	client *http.Client
}

type ValhallaClientOption func(*ValhallaClientConfig)

type ValhallaClient struct {
	cfg ValhallaClientConfig
}

func WithClient(client *http.Client) ValhallaClientOption {
	return func(cfg *ValhallaClientConfig) {
		cfg.client = client
	}
}

func WithHostURL(hostURL *url.URL) ValhallaClientOption {
	return func(cfg *ValhallaClientConfig) {
		cfg.url = hostURL
	}
}

var defaultCfg = ValhallaClientConfig{
	client: http.DefaultClient,
}

func NewValhallaClient(opts ...ValhallaClientOption) ValhallaClient {
	cfg := defaultCfg
	for _, opt := range opts {
		opt(&cfg)
	}
	return ValhallaClient{
		cfg: cfg,
	}
}

func (o *ValhallaClient) DirectionsJSON(ctx context.Context, reqBody *DirectionRequest) (*DirectionResponse, error) {
	log := logger.GetLogger(ctx)
	reqBody.Format = "json"
	var buf strings.Builder
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal valhalla req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	path := fmt.Sprintf("%s/route", o.cfg.url.String())
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, path, http.NoBody)
	if err != nil {
		return nil, err
	}

	query := req.URL.Query()
	query.Add("json", buf.String())
	req.URL.RawQuery = query.Encode()

	resp, err := o.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to ors service", "error", err)
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, err := io.ReadAll(resp.Body)
		if err == nil {
			log.Error("response from the Valhalla service with a not successful code", "status_code", resp.StatusCode, "body", body)
		} else {
			log.Error("response from the Valhalla service with a not successful code", "status_code", resp.StatusCode)
		}
		return nil, errors.New("response not successful")
	}

	var response DirectionResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		log.Error("failed to decode ors response")
	}

	return &response, nil
}

func (o *ValhallaClient) DirectionsGeoJSON(ctx context.Context, reqBody *DirectionRequest) (*entities.GeoJSON, error) {
	response, err := o.DirectionsJSON(ctx, reqBody)
	if err != nil {
		return nil, err
	}
	return o.toGeoJSON(response), nil
}

func (o *ValhallaClient) DirectionsRawGpx(ctx context.Context, reqBody *DirectionRequest) (io.ReadCloser, error) {
	log := logger.GetLogger(ctx)
	reqBody.Format = "gpx"
	var buf strings.Builder
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal valhalla req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	path := fmt.Sprintf("%s/route", o.cfg.url.String())
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, path, http.NoBody)
	if err != nil {
		return nil, err
	}

	query := req.URL.Query()
	query.Add("json", buf.String())
	req.URL.RawQuery = query.Encode()

	resp, err := o.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to valhalla service", "error", err)
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		defer resp.Body.Close()
		body, err := io.ReadAll(resp.Body)
		if err == nil {
			log.Error("response from the valhalla service with a not successful code", "status_code", resp.StatusCode, "body", body)
		} else {
			log.Error("response from the valhalla service with a not successful code", "status_code", resp.StatusCode)
		}
		return nil, errors.New("response not successful")
	}

	return resp.Body, nil
}

func (o *ValhallaClient) toGeoJSON(resp *DirectionResponse) *entities.GeoJSON {
	bboxRoot := []float64{resp.Trip.Summary.MinLat, resp.Trip.Summary.MinLon, resp.Trip.Summary.MaxLat, resp.Trip.Summary.MinLon}
	features := utils.Map(resp.Trip.Legs, func(leg LegResponse) entities.GeoJSONFeature {
		coords := o.decodePolyline(&leg.Shape)
		bbox := []float64{leg.Summary.MinLat, leg.Summary.MinLon, leg.Summary.MaxLat, leg.Summary.MaxLon}

		return entities.GeoJSONFeature{
			Type:       entities.Feature,
			Bbox:       bbox,
			Properties: make(map[string]any),
			Geometry: entities.GeoJSONGeometry{
				Type:        entities.LineString,
				Coordinates: coords,
			},
		}
	})

	return &entities.GeoJSON{
		Type:     entities.FeatureCollection,
		Bbox:     bboxRoot,
		Features: features,
	}
}

func (o *ValhallaClient) decodePolyline(encoded *string, precisionOptional ...int) [][]float64 {
	// default to 6 digits of precision
	precision := 6
	if len(precisionOptional) > 0 {
		precision = precisionOptional[0]
	}
	factor := math.Pow10(precision)

	lat, lng := 0, 0
	var coordinates [][]float64
	index := 0
	for index < len(*encoded) {
		// Consume varint bits for lat until we run out
		var b = 0x20
		shift, result := 0, 0
		for b >= 0x20 {
			b = int((*encoded)[index]) - 63
			result |= (b & 0x1f) << shift
			shift += 5
			index++
		}

		// check if we need to go negative or not
		if (result & 1) > 0 {
			lat += ^(result >> 1)
		} else {
			lat += result >> 1
		}

		// Consume varint bits for lng until we run out
		b = 0x20
		shift, result = 0, 0
		for b >= 0x20 {
			b = int((*encoded)[index]) - 63
			result |= (b & 0x1f) << shift
			shift += 5
			index++
		}

		// check if we need to go negative or not
		if (result & 1) > 0 {
			lng += ^(result >> 1)
		} else {
			lng += result >> 1
		}

		// scale the int back to floating point and store it
		// -> [long, lat]
		coordinates = append(coordinates, []float64{float64(lng) / factor, float64(lat) / factor})
	}

	return coordinates
}
