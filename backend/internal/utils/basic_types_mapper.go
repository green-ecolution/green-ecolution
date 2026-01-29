package utils

import (
	"net"
	"net/url"
	"time"
)

func TimeToTime(t time.Time) time.Time {
	return t
}

func TimePtrToTime(t *time.Time) time.Time {
	if t == nil {
		return time.Time{}
	}
	return *t
}

func TimeToPtrTime(t time.Time) *time.Time {
	if t.IsZero() {
		return nil
	}
	return &t
}

func TimeToTimePtr(t *time.Time) *time.Time {
	return t
}

func URLToURL(u *url.URL) *url.URL {
	return u
}

func TimeDurationToTimeDuration(t time.Duration) time.Duration {
	return t
}

func StringToTime(s string) time.Time {
	t, _ := time.Parse(time.RFC3339, s)
	return t
}

func StringToURL(s string) *url.URL {
	u, _ := url.Parse(s)
	return u
}

func StringToNetIP(s string) net.IP {
	ip := net.ParseIP(s)
	return ip
}

func NetIPToString(ip net.IP) string {
	if ip == nil {
		return ""
	}

	return ip.String()
}

func StringToDuration(s string) time.Duration {
	d, _ := time.ParseDuration(s)
	return d
}

func TimeToString(t time.Time) string {
	return t.Format(time.RFC3339)
}

func NetURLToString(u *url.URL) string {
	if u == nil {
		return ""
	}
	return u.String()
}

func TimeDurationToString(t time.Duration) string {
	if t == 0 {
		return ""
	}
	return t.String()
}

func StringPtrToString(source *string) string {
	if source == nil {
		return ""
	}
	return *source
}

func Float64ToDuration(source float64) time.Duration {
	return time.Duration(source)
}

func DurationToPtrFloat64(source time.Duration) *float64 {
	return P(float64(source))
}

func MapKeyValueInterface(src map[string]interface{}) map[string]interface{} {
	return src
}
