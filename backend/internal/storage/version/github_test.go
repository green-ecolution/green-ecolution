package version

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

func TestNewGitHubVersionRepository(t *testing.T) {
	t.Run("should create a new repository with default URL", func(t *testing.T) {
		// when
		repo := NewGitHubVersionRepository()

		// then
		assert.NotNil(t, repo)
		assert.NotNil(t, repo.httpClient)
		assert.Equal(t, defaultGitHubReleaseURL, repo.apiURL)
	})
}

func TestGitHubVersionRepository_GetLatestVersion(t *testing.T) {
	t.Run("should fetch latest version from GitHub API", func(t *testing.T) {
		// given
		publishedAt := time.Date(2025, 1, 15, 10, 0, 0, 0, time.UTC)
		mockRelease := githubRelease{
			TagName:     "v0.1.2",
			HTMLURL:     "https://github.com/green-ecolution/green-ecolution/releases/tag/v0.1.2",
			PublishedAt: publishedAt,
		}
		responseBody, err := json.Marshal(mockRelease)
		require.NoError(t, err)

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			assert.Equal(t, "GET", r.Method)
			assert.Equal(t, "application/vnd.github.v3+json", r.Header.Get("Accept"))
			assert.Equal(t, "green-ecolution-backend", r.Header.Get("User-Agent"))

			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, responseBody)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.GetLatestVersion(context.Background())

		// then
		require.NoError(t, err)
		assert.Equal(t, "v0.1.2", result.Version)
		assert.Equal(t, "https://github.com/green-ecolution/green-ecolution/releases/tag/v0.1.2", result.ReleaseURL)
		assert.Equal(t, publishedAt, result.PublishedAt)
	})

	t.Run("should return cached result on second call", func(t *testing.T) {
		// given
		callCount := 0
		mockRelease := githubRelease{
			TagName:     "v0.1.2",
			HTMLURL:     "https://example.com/release",
			PublishedAt: time.Now(),
		}
		responseBody, err := json.Marshal(mockRelease)
		require.NoError(t, err)

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			callCount++
			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, responseBody)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result1, err1 := repo.GetLatestVersion(context.Background())
		result2, err2 := repo.GetLatestVersion(context.Background())

		// then
		require.NoError(t, err1)
		require.NoError(t, err2)
		assert.Equal(t, 1, callCount, "API should only be called once")
		assert.Equal(t, result1.Version, result2.Version)
	})

	t.Run("should return error on HTTP failure", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusInternalServerError)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.GetLatestVersion(context.Background())

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "500")
	})

	t.Run("should return error on invalid JSON", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, []byte("invalid json"))
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.GetLatestVersion(context.Background())

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "decode")
	})

	t.Run("should return stale cache on API failure", func(t *testing.T) {
		// given
		callCount := 0
		mockRelease := githubRelease{
			TagName:     "v0.1.2",
			HTMLURL:     "https://example.com/release",
			PublishedAt: time.Now(),
		}
		responseBody, err := json.Marshal(mockRelease)
		require.NoError(t, err)

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			callCount++
			if callCount == 1 {
				w.Header().Set("Content-Type", "application/json")
				mustWrite(t, w, responseBody)
			} else {
				w.WriteHeader(http.StatusInternalServerError)
			}
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// First call to populate cache
		result1, err1 := repo.GetLatestVersion(context.Background())
		require.NoError(t, err1)

		// Expire the cache manually
		repo.cacheTime = time.Now().Add(-2 * time.Hour)

		// when - second call should fail but return stale cache
		result2, err2 := repo.GetLatestVersion(context.Background())

		// then
		require.NoError(t, err2)
		assert.Equal(t, result1.Version, result2.Version)
	})

	t.Run("should handle context cancellation", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			time.Sleep(100 * time.Millisecond)
			w.WriteHeader(http.StatusOK)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		ctx, cancel := context.WithCancel(context.Background())
		cancel()

		// when
		result, err := repo.GetLatestVersion(ctx)

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
	})

	t.Run("should handle rate limit (403)", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusForbidden)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.GetLatestVersion(context.Background())

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "403")
	})

	t.Run("should handle not found (404)", func(t *testing.T) {
		// given
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.WriteHeader(http.StatusNotFound)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.GetLatestVersion(context.Background())

		// then
		assert.Error(t, err)
		assert.Nil(t, result)
		assert.Contains(t, err.Error(), "404")
	})

	t.Run("should refetch when cache expires", func(t *testing.T) {
		// given
		callCount := 0
		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			callCount++
			mockRelease := githubRelease{
				TagName:     "v0.1." + string(rune('0'+callCount)),
				HTMLURL:     "https://example.com/release",
				PublishedAt: time.Now(),
			}
			responseBody, _ := json.Marshal(mockRelease)
			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, responseBody)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// First call
		result1, err1 := repo.GetLatestVersion(context.Background())
		require.NoError(t, err1)

		// Expire the cache
		repo.cacheTime = time.Now().Add(-2 * time.Hour)

		// when - second call should refetch
		result2, err2 := repo.GetLatestVersion(context.Background())

		// then
		require.NoError(t, err2)
		assert.Equal(t, 2, callCount, "API should be called twice after cache expiry")
		assert.NotEqual(t, result1.Version, result2.Version)
	})
}

func TestGitHubVersionRepository_FetchFromGitHub(t *testing.T) {
	t.Run("should set correct headers", func(t *testing.T) {
		// given
		var receivedHeaders http.Header
		mockRelease := githubRelease{
			TagName:     "v0.1.2",
			HTMLURL:     "https://example.com/release",
			PublishedAt: time.Now(),
		}
		responseBody, err := json.Marshal(mockRelease)
		require.NoError(t, err)

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			receivedHeaders = r.Header
			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, responseBody)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		_, err = repo.fetchFromGitHub(context.Background())

		// then
		require.NoError(t, err)
		assert.Equal(t, "application/vnd.github.v3+json", receivedHeaders.Get("Accept"))
		assert.Equal(t, "green-ecolution-backend", receivedHeaders.Get("User-Agent"))
	})

	t.Run("should parse all fields from response", func(t *testing.T) {
		// given
		publishedAt := time.Date(2025, 1, 20, 14, 30, 0, 0, time.UTC)
		mockRelease := githubRelease{
			TagName:     "v1.0.0",
			HTMLURL:     "https://github.com/org/repo/releases/tag/v1.0.0",
			PublishedAt: publishedAt,
		}
		responseBody, err := json.Marshal(mockRelease)
		require.NoError(t, err)

		server := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
			w.Header().Set("Content-Type", "application/json")
			mustWrite(t, w, responseBody)
		}))
		defer server.Close()

		repo := newTestRepository(server.URL)

		// when
		result, err := repo.fetchFromGitHub(context.Background())

		// then
		require.NoError(t, err)
		assert.Equal(t, "v1.0.0", result.Version)
		assert.Equal(t, "https://github.com/org/repo/releases/tag/v1.0.0", result.ReleaseURL)
		assert.Equal(t, publishedAt, result.PublishedAt)
	})
}

// newTestRepository creates a test repository with a custom URL for the GitHub API
func newTestRepository(url string) *GitHubVersionRepository {
	repo := NewGitHubVersionRepository()
	repo.apiURL = url
	return repo
}

// mustWrite writes data to w and fails the test if an error occurs
func mustWrite(t *testing.T, w http.ResponseWriter, data []byte) {
	t.Helper()
	if _, err := w.Write(data); err != nil {
		t.Fatalf("failed to write response: %v", err)
	}
}
