package utils

import (
	"net"
	"net/url"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestNetIPToString(t *testing.T) {
	t.Run("should convert IP to string", func(t *testing.T) {
		ip := net.ParseIP("192.168.0.1")
		result := NetIPToString(ip)
		assert.Equal(t, "192.168.0.1", result)
	})

	t.Run("should return empty string for nil IP", func(t *testing.T) {
		var ip net.IP
		result := NetIPToString(ip)
		assert.Equal(t, "", result)
	})
}

func TestTimeToString(t *testing.T) {
	t.Run("should format time to RFC3339 string", func(t *testing.T) {
		tm := time.Date(2024, 1, 1, 12, 0, 0, 0, time.UTC)
		result := TimeToString(tm)
		assert.Equal(t, "2024-01-01T12:00:00Z", result)
	})
}

func TestNetURLToString(t *testing.T) {
	t.Run("should convert URL to string", func(t *testing.T) {
		u, _ := url.Parse("http://example.com")
		result := NetURLToString(u)
		assert.Equal(t, "http://example.com", result)
	})

	t.Run("should return empty string for nil URL", func(t *testing.T) {
		var u *url.URL
		result := NetURLToString(u)
		assert.Equal(t, "", result)
	})
}

func TestTimeDurationToString(t *testing.T) {
	t.Run("should format duration to string", func(t *testing.T) {
		duration := 5 * time.Second
		result := TimeDurationToString(duration)
		assert.Equal(t, "5s", result)
	})
}

func TestTimePtrToTime(t *testing.T) {
	t.Run("should dereference time pointer", func(t *testing.T) {
		now := time.Now()
		result := TimePtrToTime(&now)
		assert.Equal(t, now, result)
	})

	t.Run("should return zero time for nil", func(t *testing.T) {
		result := TimePtrToTime(nil)
		assert.True(t, result.IsZero())
	})
}

func TestTimeToPtrTime(t *testing.T) {
	t.Run("should return pointer to time", func(t *testing.T) {
		now := time.Now()
		result := TimeToPtrTime(now)
		assert.NotNil(t, result)
		assert.Equal(t, now, *result)
	})

	t.Run("should return nil for zero time", func(t *testing.T) {
		result := TimeToPtrTime(time.Time{})
		assert.Nil(t, result)
	})
}

func TestFloat64ToDuration(t *testing.T) {
	t.Run("should convert float64 to duration", func(t *testing.T) {
		result := Float64ToDuration(float64(5 * time.Second))
		assert.Equal(t, 5*time.Second, result)
	})
}
