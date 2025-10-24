package logger

import (
	"context"
	"log/slog"
)

// GetLogger retrieves a logger instance from the given context.
//
// This function attempts to extract a logger from the context using the key "logger".
// If the key is not present or the value is not of type *slog.Logger, it falls back
// to returning the default logger provided by slog.
//
// Usage:
//  1. To use this function effectively, ensure that the context has a logger stored
//     using context.WithValue, with the key "logger" and a value of type *slog.Logger.
//  2. If no logger is found in the context, slog.Default() is returned.
//
// Parameters:
// - ctx (context.Context): The context from which the logger will be extracted.
//
// Returns:
// - *slog.Logger: The logger instance extracted from the context or the default logger.
//
// Example:
// ```go
// logger := slog.New(slog.WithLevel(slog.LevelInfo))
// ctx := context.WithValue(context.Background(), "logger", logger)
// retrievedLogger := GetLogger(ctx)
// retrievedLogger.Info("Logger successfully retrieved!")
// ```
func GetLogger(ctx context.Context) *slog.Logger {
	log, ok := ctx.Value("logger").(*slog.Logger)
	if !ok {
		log = slog.Default()
	}
	return log
}
