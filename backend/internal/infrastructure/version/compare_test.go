package version

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestCompareVersions(t *testing.T) {
	t.Run("should detect development version with git hash", func(t *testing.T) {
		// given
		current := "v0.1.1-49-g0b049dce"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.Equal(t, current, result.Current)
		assert.Equal(t, latest, result.Latest)
		assert.True(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should detect development version with dirty flag", func(t *testing.T) {
		// given
		current := "v0.1.1-49-g0b049dce-dirty"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should detect 'development' string as development version", func(t *testing.T) {
		// given
		current := "development"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should detect 'develop' string as development version", func(t *testing.T) {
		// given
		current := "develop"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should detect stage version", func(t *testing.T) {
		// given
		current := "v0.1.2-608bd4e-stage"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.False(t, result.IsDevelopment)
		assert.True(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should detect update available when latest is greater", func(t *testing.T) {
		// given
		current := "v0.1.1"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.False(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.True(t, result.UpdateAvailable)
	})

	t.Run("should not detect update when versions are equal", func(t *testing.T) {
		// given
		current := "v0.1.2"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.False(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should not detect update when current is greater", func(t *testing.T) {
		// given
		current := "v0.1.3"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.False(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should handle major version updates", func(t *testing.T) {
		// given
		current := "v1.0.0"
		latest := "v2.0.0"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.UpdateAvailable)
	})

	t.Run("should handle minor version updates", func(t *testing.T) {
		// given
		current := "v1.0.0"
		latest := "v1.1.0"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.UpdateAvailable)
	})

	t.Run("should handle patch version updates", func(t *testing.T) {
		// given
		current := "v1.0.0"
		latest := "v1.0.1"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.UpdateAvailable)
	})

	t.Run("should handle invalid current version gracefully", func(t *testing.T) {
		// given
		current := "invalid-version"
		latest := "v0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.Equal(t, current, result.Current)
		assert.Equal(t, latest, result.Latest)
		assert.False(t, result.UpdateAvailable)
		assert.False(t, result.IsDevelopment)
		assert.False(t, result.IsStage)
	})

	t.Run("should handle invalid latest version gracefully", func(t *testing.T) {
		// given
		current := "v0.1.1"
		latest := "invalid-version"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should handle empty latest version", func(t *testing.T) {
		// given
		current := "v0.1.1"
		latest := ""

		// when
		result := CompareVersions(current, latest)

		// then
		assert.Equal(t, current, result.Current)
		assert.Equal(t, "", result.Latest)
		assert.False(t, result.UpdateAvailable)
	})

	t.Run("should handle versions without v prefix", func(t *testing.T) {
		// given
		current := "0.1.1"
		latest := "0.1.2"

		// when
		result := CompareVersions(current, latest)

		// then
		assert.True(t, result.UpdateAvailable)
	})
}

func TestIsDevelopmentVersion(t *testing.T) {
	tests := []struct {
		name     string
		version  string
		expected bool
	}{
		{"git hash suffix", "v0.1.1-49-g0b049dce", true},
		{"dirty suffix", "v0.1.1-dirty", true},
		{"git hash with dirty", "v0.1.1-49-g0b049dce-dirty", true},
		{"development string", "development", true},
		{"develop string", "develop", true},
		{"clean release", "v0.1.1", false},
		{"stage version", "v0.1.2-stage", false},
		{"stage with hash", "v0.1.2-608bd4e-stage", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// when
			result := isDevelopmentVersion(tt.version)

			// then
			assert.Equal(t, tt.expected, result)
		})
	}
}

func TestIsStageVersion(t *testing.T) {
	tests := []struct {
		name     string
		version  string
		expected bool
	}{
		{"stage suffix", "v0.1.2-stage", true},
		{"stage with hash", "v0.1.2-608bd4e-stage", true},
		{"clean release", "v0.1.1", false},
		{"development version", "v0.1.1-49-g0b049dce", false},
		{"development string", "development", false},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			// when
			result := isStageVersion(tt.version)

			// then
			assert.Equal(t, tt.expected, result)
		})
	}
}
