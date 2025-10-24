package utils

import (
	"net"
	"net/url"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
)

func TestTimeToTime(t *testing.T) {
	t.Run("should return same time value", func(t *testing.T) {
		now := time.Now()
		result := TimeToTime(now)
		assert.Equal(t, now, result)
	})
}

func TestTimeToTimePtr(t *testing.T) {
	t.Run("should return pointer to time", func(t *testing.T) {
		now := time.Now()
		result := TimeToTimePtr(&now)
		assert.NotNil(t, result)
		assert.Equal(t, now, *result)
	})

	t.Run("should return nil when nil is passed", func(t *testing.T) {
		result := TimeToTimePtr(nil)
		assert.Nil(t, result)
	})
}

func TestURLToURL(t *testing.T) {
	t.Run("should return same URL pointer", func(t *testing.T) {
		u, _ := url.Parse("http://example.com")
		result := URLToURL(u)
		assert.Equal(t, u, result)
	})

	t.Run("should return nil when nil is passed", func(t *testing.T) {
		result := URLToURL(nil)
		assert.Nil(t, result)
	})
}

func TestTimeDurationToTimeDuration(t *testing.T) {
	t.Run("should return same duration", func(t *testing.T) {
		duration := 5 * time.Second
		result := TimeDurationToTimeDuration(duration)
		assert.Equal(t, duration, result)
	})
}

func TestStringToTime(t *testing.T) {
	t.Run("should parse valid RFC3339 string to time", func(t *testing.T) {
		timeStr := "2023-01-01T12:00:00Z"
		expectedTime, _ := time.Parse(time.RFC3339, timeStr)
		result := StringToTime(timeStr)
		assert.Equal(t, expectedTime, result)
	})

	t.Run("should return zero time for invalid string", func(t *testing.T) {
		result := StringToTime("invalid")
		assert.True(t, result.IsZero())
	})
}

func TestStringToURL(t *testing.T) {
	t.Run("should parse valid URL string", func(t *testing.T) {
		urlStr := "http://example.com"
		expectedURL, _ := url.Parse(urlStr)
		result := StringToURL(urlStr)
		assert.Equal(t, expectedURL, result)
	})

	t.Run("should return nil for invalid URL string", func(t *testing.T) {
		result := StringToURL("://invalid-url")
		assert.Nil(t, result)
	})
}

func TestStringToNetIP(t *testing.T) {
	t.Run("should parse valid IP string", func(t *testing.T) {
		ipStr := "192.168.0.1"
		expectedIP := net.ParseIP(ipStr)
		result := StringToNetIP(ipStr)
		assert.Equal(t, expectedIP, result)
	})

	t.Run("should return nil for invalid IP string", func(t *testing.T) {
		result := StringToNetIP("invalid-ip")
		assert.Nil(t, result)
	})
}

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

func TestStringToDuration(t *testing.T) {
	t.Run("should parse valid duration string", func(t *testing.T) {
		durationStr := "5s"
		expectedDuration, _ := time.ParseDuration(durationStr)
		result := StringToDuration(durationStr)
		assert.Equal(t, expectedDuration, result)
	})

	t.Run("should return zero duration for invalid string", func(t *testing.T) {
		result := StringToDuration("invalid")
		assert.Equal(t, time.Duration(0), result)
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
