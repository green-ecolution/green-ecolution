package user

import (
	"github.com/gofiber/fiber/v2"
	"github.com/green-ecolution/backend/internal/service"
)

func RegisterRoutes(r fiber.Router, svc service.AuthService) {
	r.Post("/", Register(svc))
	r.Get("/", GetAllUsers(svc))
	r.Get("/role/:role", GetUsersByRole(svc))
}

func RegisterPublicRoutes(r fiber.Router, svc service.AuthService) {
	r.Get("/auth/dummy", GetAuthDummyCode())
	r.Post("/logout", Logout(svc))
	r.Get("/login", Login(svc))
	r.Post("/login/token", RequestToken(svc))
	r.Post("/token/refresh", RefreshToken(svc))
}
