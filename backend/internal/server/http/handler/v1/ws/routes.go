package ws

import (
	"github.com/gofiber/contrib/websocket"
	"github.com/gofiber/fiber/v2"
)

// RegisterRoutes registers WebSocket routes
// Runtime stats are public as they only contain non-sensitive Go runtime metrics
func RegisterRoutes(router fiber.Router) {
	router.Use("/stats", upgradeMiddleware)
	router.Get("/stats", websocket.New(StatsHandler))
}

// upgradeMiddleware checks if the request is a WebSocket upgrade request
func upgradeMiddleware(c *fiber.Ctx) error {
	if websocket.IsWebSocketUpgrade(c) {
		c.Locals("allowed", true)
		return c.Next()
	}
	return fiber.ErrUpgradeRequired
}
