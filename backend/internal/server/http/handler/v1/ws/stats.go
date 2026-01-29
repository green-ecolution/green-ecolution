package ws

import (
	"encoding/json"
	"os"
	"runtime"
	"time"

	"github.com/gofiber/contrib/websocket"
	"github.com/green-ecolution/green-ecolution/backend/internal/logger"
)

// RuntimeStats represents Go runtime metrics
type RuntimeStats struct {
	// Memory
	Alloc      uint64 `json:"alloc"`
	TotalAlloc uint64 `json:"totalAlloc"`
	Sys        uint64 `json:"sys"`
	HeapAlloc  uint64 `json:"heapAlloc"`
	HeapSys    uint64 `json:"heapSys"`
	HeapInuse  uint64 `json:"heapInuse"`

	// Goroutines
	NumGoroutine int `json:"numGoroutine"`

	// GC
	NumGC        uint32 `json:"numGC"`
	PauseTotalNs uint64 `json:"pauseTotalNs"`

	// CPU
	NumCPU int `json:"numCPU"`

	// Timestamp
	Timestamp int64 `json:"timestamp"`
}

func collectRuntimeStats() RuntimeStats {
	var m runtime.MemStats
	runtime.ReadMemStats(&m)

	return RuntimeStats{
		Alloc:        m.Alloc,
		TotalAlloc:   m.TotalAlloc,
		Sys:          m.Sys,
		HeapAlloc:    m.HeapAlloc,
		HeapSys:      m.HeapSys,
		HeapInuse:    m.HeapInuse,
		NumGoroutine: runtime.NumGoroutine(),
		NumGC:        m.NumGC,
		PauseTotalNs: m.PauseTotalNs,
		NumCPU:       runtime.NumCPU(),
		Timestamp:    time.Now().UnixMilli(),
	}
}

// StatsHandler handles WebSocket connections for runtime stats
func StatsHandler(c *websocket.Conn) {
	log := logger.CreateLogger(os.Stdout, logger.Console, logger.Debug)()
	log.Debug("WebSocket connection established for runtime stats")

	ticker := time.NewTicker(2 * time.Second)
	defer ticker.Stop()
	defer c.Close()

	// Send initial stats
	stats := collectRuntimeStats()
	if err := sendStats(c, &stats); err != nil {
		log.Debug("failed to send initial stats", "error", err)
		return
	}

	for range ticker.C {
		stats := collectRuntimeStats()
		if err := sendStats(c, &stats); err != nil {
			log.Debug("failed to send stats, closing connection", "error", err)
			return
		}
	}
}

func sendStats(c *websocket.Conn, stats *RuntimeStats) error {
	data, err := json.Marshal(stats)
	if err != nil {
		return err
	}
	return c.WriteMessage(websocket.TextMessage, data)
}
