package utils

import (
	"io"
	"net/http"
	"net/url"
	"strings"
	"testing"

	"github.com/google/uuid"
	"github.com/stretchr/testify/assert"
)

func TestP(t *testing.T) {
	t.Run("should return pointer of string value", func(t *testing.T) {
		// given
		value := "value"

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of int value", func(t *testing.T) {
		// given
		value := 10

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of bool value", func(t *testing.T) {
		// given
		value := true

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of float64 value", func(t *testing.T) {
		// given
		value := 10.5

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of struct value", func(t *testing.T) {
		// given
		value := struct{ Name string }{Name: "name"}

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of slice value", func(t *testing.T) {
		// given
		value := []string{"value"}

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of map value", func(t *testing.T) {
		// given
		value := map[string]string{"key": "value"}

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of interface value", func(t *testing.T) {
		// given
		value := interface{}("value")

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of channel value", func(t *testing.T) {
		// given
		value := make(chan string)

		// when
		result := P(value)

		// then
		assert.Equal(t, &value, result)
	})

	t.Run("should return pointer of pointer value", func(t *testing.T) {
		// given
		value := "value"
		pointer := &value

		// when
		result := P(pointer)

		// then
		assert.Equal(t, &pointer, result)
	})
}

func TestRootDir(t *testing.T) {
	t.Run("should return root directory of the project", func(t *testing.T) {
		// when
		result := RootDir()

		// then
		assert.NotEmpty(t, result)
	})
}

func TestParseJSONResponse(t *testing.T) {
	t.Run("should successfully decode JSON response", func(t *testing.T) {
		// given
		responseBody := `{"name":"Toni","age":30}`
		response := &http.Response{
			Body: io.NopCloser(strings.NewReader(responseBody)),
		}
		var result map[string]any

		// when
		err := ParseJSONResponse(response, &result)

		// then
		assert.NoError(t, err)
		assert.Equal(t, "Toni", result["name"])
		assert.Equal(t, 30.0, result["age"])
	})

	t.Run("should return error for invalid JSON", func(t *testing.T) {
		// given
		responseBody := `{"name": "Toni", "age": }` // Invalid JSON
		response := &http.Response{
			Body: io.NopCloser(strings.NewReader(responseBody)),
		}
		var result map[string]any

		// when
		err := ParseJSONResponse(response, &result)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "invalid character")
	})

	t.Run("should handle empty body", func(t *testing.T) {
		// given
		response := &http.Response{
			Body: io.NopCloser(strings.NewReader("")),
		}
		var result map[string]any

		// when
		err := ParseJSONResponse(response, &result)

		// then
		assert.Error(t, err)
		assert.Contains(t, err.Error(), "EOF")
	})

	t.Run("should handle JSON array response", func(t *testing.T) {
		// given
		responseBody := `[{"name":"Toni"},{"name":"Tester"}]`
		response := &http.Response{
			Body: io.NopCloser(strings.NewReader(responseBody)),
		}
		var result []map[string]any

		// when
		err := ParseJSONResponse(response, &result)

		// then
		assert.NoError(t, err)
		assert.Len(t, result, 2)
		assert.Equal(t, "Toni", result[0]["name"])
		assert.Equal(t, "Tester", result[1]["name"])
	})
}

func TestStringPtrToString(t *testing.T) {
	t.Run("should return empty string when source is nil", func(t *testing.T) {
		// when
		result := StringPtrToString(nil)

		// then
		assert.Empty(t, result)
	})

	t.Run("should return string value when source is not nil", func(t *testing.T) {
		// given
		source := "source"

		// when
		result := StringPtrToString(&source)

		// then
		assert.Equal(t, source, result)
	})
}

func TestUUIDToString(t *testing.T) {
	t.Run("should return string representation of UUID", func(t *testing.T) {
		// given
		testUUID := uuid.New()

		// when
		result := UUIDToString(testUUID)

		// then
		assert.Equal(t, testUUID.String(), result)
	})

	t.Run("should return empty string for nil UUID", func(t *testing.T) {
		// given
		var nilUUID uuid.UUID

		// when
		result := UUIDToString(nilUUID)

		// then
		assert.Equal(t, "", result)
	})
}

func TestURLToString(t *testing.T) {
	t.Run("should return empty string when URL is nil", func(t *testing.T) {
		// given
		var u *url.URL

		// when
		result := URLToString(u)

		// then
		assert.Equal(t, "", result)
	})

	t.Run("should return string representation of URL when URL is valid", func(t *testing.T) {
		// given
		u, err := url.Parse("https://example.com/path?query=value")
		if err != nil {
			t.Fatalf("Failed to parse URL: %v", err)
		}

		// when
		result := URLToString(u)

		// then
		assert.Equal(t, "https://example.com/path?query=value", result)
	})

	t.Run("should return empty string when URL is an empty URL", func(t *testing.T) {
		// given
		u, err := url.Parse("")
		if err != nil {
			t.Fatalf("Failed to parse URL: %v", err)
		}

		// when
		result := URLToString(u)

		// then
		assert.Equal(t, "", result)
	})
}
