package middleware

import (
	"strconv"

	"github.com/gofiber/fiber/v2"
)

func PaginationMiddleware() fiber.Handler {
	return func(c *fiber.Ctx) error {
		defaultPage := 1
		defaultLimit := -1

		pageParam := c.Query("page", strconv.Itoa(defaultPage))
		page, err := strconv.Atoi(pageParam)
		if err != nil || page < 1 {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "invalid page format",
			})
		}

		limitParam := c.Query("limit", strconv.Itoa(defaultLimit))
		limit, err := strconv.Atoi(limitParam)
		if err != nil || (limit != -1 && limit <= 0) {
			return c.Status(fiber.StatusBadRequest).JSON(fiber.Map{
				"error": "invalid limit format",
			})
		}

		c.Locals("page", int32(page))
		c.Locals("limit", int32(limit))

		return c.Next()
	}
}
