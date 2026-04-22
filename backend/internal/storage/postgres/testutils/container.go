package testutils

import (
	"context"
	"log"
	"time"

	"github.com/testcontainers/testcontainers-go"
	"github.com/testcontainers/testcontainers-go/modules/postgres"
	"github.com/testcontainers/testcontainers-go/wait"
)

var (
	dbUsername = "user"
	dbPassword = "geheim"
	dbName     = "ge-test"
	dbDriver   = "pgx"
)

// SetupPostgres starts a postgres container
func SetupPostgres(ctx context.Context) *postgres.PostgresContainer {
	postgis, err := postgres.Run(ctx,
		"postgis/postgis",
		postgres.WithDatabase(dbName),
		postgres.WithUsername(dbUsername),
		postgres.WithPassword(dbPassword),
		postgres.WithSQLDriver(dbDriver),
		testcontainers.WithWaitStrategy(
			wait.ForLog("database system is ready to accept connections").
				WithOccurrence(2).
				WithStartupTimeout(60*time.Second),
		),
	)
	if err != nil {
		log.Fatalf("Could not start postgres container: %s", err)
	}

	return postgis
}

func (s *PostgresTestSuite) Terminate(ctx context.Context) {
	if err := s.Container.Terminate(ctx); err != nil {
		log.Fatalf("Could not terminate container: %s", err)
	}
}
