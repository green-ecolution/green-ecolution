package treecluster

import (
	"github.com/gofiber/fiber/v2"

	"github.com/green-ecolution/green-ecolution/backend/internal/application/ports"
)

func RegisterRoutes(r fiber.Router, svc ports.TreeClusterService) {
	r.Get("/", GetAllTreeClusters(svc))
	r.Get("/:treecluster_id", GetTreeClusterByID(svc))
	r.Post("/", CreateTreeCluster(svc))
	r.Put("/:treecluster_id", UpdateTreeCluster(svc))
	r.Delete("/:treecluster_id", DeleteTreeCluster(svc))
}
