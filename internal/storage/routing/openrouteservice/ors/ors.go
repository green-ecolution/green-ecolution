package ors

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

	"github.com/green-ecolution/backend/internal/entities"
	"github.com/green-ecolution/backend/internal/logger"
)

type OrsClientConfig struct {
	url    *url.URL
	client *http.Client
}

type OrsClientOption func(*OrsClientConfig)

type OrsClient struct {
	cfg OrsClientConfig
}

func WithClient(client *http.Client) OrsClientOption {
	return func(cfg *OrsClientConfig) {
		cfg.client = client
	}
}

func WithHostURL(hostURL *url.URL) OrsClientOption {
	slog.Debug("use ors client with host url", "host_url", hostURL)
	return func(cfg *OrsClientConfig) {
		cfg.url = hostURL
	}
}

var defaultCfg = OrsClientConfig{
	client: http.DefaultClient,
}

func NewOrsClient(opts ...OrsClientOption) OrsClient {
	cfg := defaultCfg
	for _, opt := range opts {
		opt(&cfg)
	}
	return OrsClient{
		cfg: cfg,
	}
}

func (o *OrsClient) DirectionsGeoJSON(ctx context.Context, profile string, reqBody *OrsDirectionRequest) (*entities.GeoJSON, error) {
	log := logger.GetLogger(ctx)
	var buf bytes.Buffer
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal ors req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	path := fmt.Sprintf("%s/v2/directions/%s/geojson", o.cfg.url.String(), profile)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, path, &buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := o.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to ors service", "error", err)
		return nil, err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, err := io.ReadAll(resp.Body)
		if err == nil {
			log.Error("response from the ORS service with a not successful code", "status_code", resp.StatusCode, "body", body)
		} else {
			log.Error("response from the ORS service with a not successful code", "status_code", resp.StatusCode)
		}
		return nil, errors.New("response not successful")
	}

	var geoJSON entities.GeoJSON
	if err := json.NewDecoder(resp.Body).Decode(&geoJSON); err != nil {
		log.Error("failed to decode ors response")
	}

	return &geoJSON, nil
}

func (o *OrsClient) DirectionsRawGpx(ctx context.Context, profile string, reqBody *OrsDirectionRequest) (io.ReadCloser, error) {
	log := logger.GetLogger(ctx)
	var buf bytes.Buffer
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal ors req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	path := fmt.Sprintf("%s/v2/directions/%s/gpx", o.cfg.url.String(), profile)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, path, &buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := o.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to ors service", "error", err)
		return nil, err
	}

	if resp.StatusCode != http.StatusOK {
		defer resp.Body.Close()
		body, err := io.ReadAll(resp.Body)
		if err == nil {
			log.Error("response from the ORS service with a not successful code", "status_code", resp.StatusCode, "body", body)
		} else {
			log.Error("response from the ORS service with a not successful code", "status_code", resp.StatusCode)
		}
		return nil, errors.New("response not successful")
	}

	return resp.Body, nil
}

func (o *OrsClient) DirectionsJSON(ctx context.Context, profile string, reqBody *OrsDirectionRequest) (*OrsResponse, error) {
	log := logger.GetLogger(ctx)
	var buf bytes.Buffer
	if err := json.NewEncoder(&buf).Encode(reqBody); err != nil {
		log.Error("failed to marshal ors req body", "error", err, "req_body", reqBody)
		return nil, err
	}

	path := fmt.Sprintf("%s/v2/directions/%s/json", o.cfg.url.String(), profile)
	req, err := http.NewRequestWithContext(ctx, http.MethodPost, path, &buf)
	if err != nil {
		return nil, err
	}

	req.Header.Set("Content-Type", "application/json")
	resp, err := o.cfg.client.Do(req)
	if err != nil {
		log.Error("failed to send request to ors service", "error", err)
		return nil, err
	}
	defer resp.Body.Close()

	var response OrsResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		log.Error("failed to decode ors response")
	}

	return &response, nil
}
