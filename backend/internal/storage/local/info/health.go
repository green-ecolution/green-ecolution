package info

import (
	"context"
	"fmt"
	"log/slog"
	"net/http"
	"time"

	"github.com/green-ecolution/green-ecolution/backend/internal/entities"
)

const (
	msgDisabled      = "Deaktiviert"
	msgConnected     = "Verbunden"
	msgNoConnection  = "Keine Verbindung"
	msgURLNotConfig  = "URL nicht konfiguriert"
	msgNotConfigured = "Nicht konfiguriert"
)

type S3HealthChecker interface {
	BucketExists(ctx context.Context) (bool, error)
}

type healthChecker struct {
	cfg        healthConfig
	dbPool     DBPool
	s3Repo     S3HealthChecker
	httpClient *http.Client
}

type healthConfig struct {
	dbEnabled      bool
	authEnabled    bool
	authHealthURL  string
	mqttEnabled    bool
	s3Enabled      bool
	routingEnabled bool
	routingURL     string
	vroomEnabled   bool
	vroomURL       string
}

func newHealthChecker(r *InfoRepository) *healthChecker {
	vroomEnabled := r.cfg.Routing.Enable && r.cfg.Routing.Valhalla.Optimization.Vroom.Host != ""
	var vroomURL string
	if vroomEnabled {
		vroomURL = fmt.Sprintf("%s/health", r.cfg.Routing.Valhalla.Optimization.Vroom.Host)
	}

	var routingURL string
	if r.cfg.Routing.Enable && r.cfg.Routing.Valhalla.Host != "" {
		routingURL = fmt.Sprintf("%s/status", r.cfg.Routing.Valhalla.Host)
	}

	var authHealthURL string
	if r.cfg.IdentityAuth.Enable {
		if r.cfg.IdentityAuth.OidcProvider.HealthURL != "" {
			authHealthURL = r.cfg.IdentityAuth.OidcProvider.HealthURL
		} else if r.cfg.IdentityAuth.OidcProvider.BaseURL != "" {
			authHealthURL = fmt.Sprintf("%s/health/ready", r.cfg.IdentityAuth.OidcProvider.BaseURL)
		}
	}

	slog.Debug("health checker initialized",
		"authEnabled", r.cfg.IdentityAuth.Enable,
		"authHealthURL", authHealthURL,
		"routingEnabled", r.cfg.Routing.Enable,
		"routingURL", routingURL,
		"vroomEnabled", vroomEnabled,
		"vroomURL", vroomURL,
	)

	return &healthChecker{
		cfg: healthConfig{
			dbEnabled:      true,
			authEnabled:    r.cfg.IdentityAuth.Enable,
			authHealthURL:  authHealthURL,
			mqttEnabled:    r.cfg.MQTT.Enable,
			s3Enabled:      r.cfg.S3.Enable,
			routingEnabled: r.cfg.Routing.Enable,
			routingURL:     routingURL,
			vroomEnabled:   vroomEnabled,
			vroomURL:       vroomURL,
		},
		dbPool:     r.dbPool,
		s3Repo:     r.s3Repo,
		httpClient: &http.Client{Timeout: 5 * time.Second},
	}
}

func (h *healthChecker) checkAll(ctx context.Context) []entities.ServiceStatus {
	now := time.Now()
	services := []entities.ServiceStatus{
		h.checkDatabase(ctx, now),
		h.checkAuth(ctx, now),
		h.checkMQTT(now),
		h.checkS3(ctx, now),
		h.checkRouting(ctx, now),
		h.checkVroom(ctx, now),
	}
	return services
}

func (h *healthChecker) checkDatabase(ctx context.Context, now time.Time) entities.ServiceStatus {
	status := entities.ServiceStatus{
		Name:        "database",
		Enabled:     h.cfg.dbEnabled,
		LastChecked: now,
	}

	if !h.cfg.dbEnabled || h.dbPool == nil {
		status.Healthy = false
		status.Message = msgDisabled
		return status
	}

	start := time.Now()
	err := h.dbPool.Ping(ctx)
	status.ResponseTime = time.Since(start)

	if err != nil {
		status.Healthy = false
		status.Message = msgNoConnection
		return status
	}

	status.Healthy = true
	status.Message = msgConnected
	return status
}

func (h *healthChecker) checkAuth(ctx context.Context, now time.Time) entities.ServiceStatus {
	status := entities.ServiceStatus{
		Name:        "auth",
		Enabled:     h.cfg.authEnabled,
		LastChecked: now,
	}

	if !h.cfg.authEnabled {
		status.Healthy = false
		status.Message = msgDisabled
		return status
	}

	if h.cfg.authHealthURL == "" {
		status.Healthy = false
		status.Message = msgURLNotConfig
		return status
	}

	healthy, responseTime := h.httpHealthCheck(ctx, h.cfg.authHealthURL)
	status.Healthy = healthy
	status.ResponseTime = responseTime
	if healthy {
		status.Message = msgConnected
	} else {
		status.Message = msgNoConnection
	}
	return status
}

func (h *healthChecker) checkMQTT(now time.Time) entities.ServiceStatus {
	// MQTT health check would require MQTT client reference
	// For now, just check if enabled
	status := entities.ServiceStatus{
		Name:        "mqtt",
		Enabled:     h.cfg.mqttEnabled,
		Healthy:     h.cfg.mqttEnabled,
		LastChecked: now,
	}
	if h.cfg.mqttEnabled {
		status.Message = "Aktiviert"
	} else {
		status.Message = msgDisabled
	}
	return status
}

func (h *healthChecker) checkS3(ctx context.Context, now time.Time) entities.ServiceStatus {
	status := entities.ServiceStatus{
		Name:        "s3",
		Enabled:     h.cfg.s3Enabled,
		LastChecked: now,
	}

	if !h.cfg.s3Enabled {
		status.Healthy = false
		status.Message = msgDisabled
		return status
	}

	if h.s3Repo == nil {
		status.Healthy = false
		status.Message = msgNotConfigured
		return status
	}

	checkCtx, cancel := context.WithTimeout(ctx, 5*time.Second)
	defer cancel()

	start := time.Now()
	exists, err := h.s3Repo.BucketExists(checkCtx)
	status.ResponseTime = time.Since(start)

	if err != nil {
		slog.Debug("s3 health check: bucket exists failed", "error", err, "responseTime", status.ResponseTime)
		status.Healthy = false
		status.Message = "Verbindungsfehler"
		return status
	}

	if !exists {
		slog.Debug("s3 health check: bucket does not exist", "responseTime", status.ResponseTime)
		status.Healthy = false
		status.Message = "Bucket nicht gefunden"
		return status
	}

	slog.Debug("s3 health check: success", "exists", exists, "responseTime", status.ResponseTime)
	status.Healthy = true
	status.Message = msgConnected
	return status
}

func (h *healthChecker) checkRouting(ctx context.Context, now time.Time) entities.ServiceStatus {
	status := entities.ServiceStatus{
		Name:        "routing",
		Enabled:     h.cfg.routingEnabled,
		LastChecked: now,
	}

	if !h.cfg.routingEnabled {
		status.Healthy = false
		status.Message = msgDisabled
		return status
	}

	if h.cfg.routingURL == "" {
		status.Healthy = false
		status.Message = msgURLNotConfig
		return status
	}

	healthy, responseTime := h.httpHealthCheck(ctx, h.cfg.routingURL)
	status.Healthy = healthy
	status.ResponseTime = responseTime
	if healthy {
		status.Message = msgConnected
	} else {
		status.Message = msgNoConnection
	}
	return status
}

func (h *healthChecker) checkVroom(ctx context.Context, now time.Time) entities.ServiceStatus {
	status := entities.ServiceStatus{
		Name:        "vroom",
		Enabled:     h.cfg.vroomEnabled,
		LastChecked: now,
	}

	if !h.cfg.vroomEnabled {
		status.Healthy = false
		status.Message = msgDisabled
		return status
	}

	if h.cfg.vroomURL == "" {
		status.Healthy = false
		status.Message = msgURLNotConfig
		return status
	}

	healthy, responseTime := h.httpHealthCheck(ctx, h.cfg.vroomURL)
	status.Healthy = healthy
	status.ResponseTime = responseTime
	if healthy {
		status.Message = msgConnected
	} else {
		status.Message = msgNoConnection
	}
	return status
}

func (h *healthChecker) httpHealthCheck(ctx context.Context, url string) (bool, time.Duration) {
	if url == "" {
		return false, 0
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, http.NoBody)
	if err != nil {
		slog.Debug("health check: failed to create request", "url", url, "error", err)
		return false, 0
	}

	start := time.Now()
	resp, err := h.httpClient.Do(req)
	responseTime := time.Since(start)

	if err != nil {
		slog.Debug("health check: request failed", "url", url, "error", err, "responseTime", responseTime)
		return false, responseTime
	}
	defer resp.Body.Close()

	healthy := resp.StatusCode >= 200 && resp.StatusCode < 300
	if !healthy {
		slog.Debug("health check: unhealthy status", "url", url, "statusCode", resp.StatusCode, "responseTime", responseTime)
	}

	return healthy, responseTime
}
