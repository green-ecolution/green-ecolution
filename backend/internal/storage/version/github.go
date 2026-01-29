package version

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"sync"
	"time"
)

const (
	githubReleaseURL = "https://api.github.com/repos/green-ecolution/green-ecolution/releases/latest"
	cacheTTL         = 1 * time.Hour
)

type githubRelease struct {
	TagName     string    `json:"tag_name"`
	HTMLURL     string    `json:"html_url"`
	PublishedAt time.Time `json:"published_at"`
}

type GitHubVersionRepository struct {
	httpClient *http.Client

	cacheMu    sync.RWMutex
	cachedInfo *LatestVersionInfo
	cacheTime  time.Time
}

func NewGitHubVersionRepository() *GitHubVersionRepository {
	return &GitHubVersionRepository{
		httpClient: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

func (r *GitHubVersionRepository) GetLatestVersion(ctx context.Context) (*LatestVersionInfo, error) {
	r.cacheMu.RLock()
	if r.cachedInfo != nil && time.Since(r.cacheTime) < cacheTTL {
		info := r.cachedInfo
		r.cacheMu.RUnlock()
		return info, nil
	}
	r.cacheMu.RUnlock()

	r.cacheMu.Lock()
	defer r.cacheMu.Unlock()

	// Double-check after acquiring write lock
	if r.cachedInfo != nil && time.Since(r.cacheTime) < cacheTTL {
		return r.cachedInfo, nil
	}

	info, err := r.fetchFromGitHub(ctx)
	if err != nil {
		slog.Warn("failed to fetch latest version from GitHub", "error", err)
		// Return cached info if available, even if stale
		if r.cachedInfo != nil {
			return r.cachedInfo, nil
		}
		return nil, err
	}

	r.cachedInfo = info
	r.cacheTime = time.Now()

	return info, nil
}

func (r *GitHubVersionRepository) fetchFromGitHub(ctx context.Context) (*LatestVersionInfo, error) {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, githubReleaseURL, nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create request: %w", err)
	}

	req.Header.Set("Accept", "application/vnd.github.v3+json")
	req.Header.Set("User-Agent", "green-ecolution-backend")

	resp, err := r.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("failed to fetch from GitHub: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("GitHub API returned status %d", resp.StatusCode)
	}

	var release githubRelease
	if err := json.NewDecoder(resp.Body).Decode(&release); err != nil {
		return nil, fmt.Errorf("failed to decode GitHub response: %w", err)
	}

	return &LatestVersionInfo{
		Version:     release.TagName,
		ReleaseURL:  release.HTMLURL,
		PublishedAt: release.PublishedAt,
	}, nil
}
