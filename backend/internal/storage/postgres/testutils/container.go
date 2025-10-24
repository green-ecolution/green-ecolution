package testutils

import (
	"context"
	"fmt"
	"log"
	"time"

	"github.com/docker/go-connections/nat"
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
	startupTimeout := time.Second * 30
	pollInterval := time.Microsecond * 100

	postgis, err := postgres.Run(ctx,
		"postgis/postgis",
		postgres.WithDatabase(dbName),
		postgres.WithUsername(dbUsername),
		postgres.WithPassword(dbPassword),
		postgres.WithSQLDriver(dbDriver),
		testcontainers.WithWaitStrategy(
			wait.ForSQL(nat.Port("5432/tcp"), dbDriver, func(host string, port nat.Port) string {
				return fmt.Sprintf("postgresql://%s:%s@%s:%s/%s", dbUsername, dbPassword, host, port.Port(), dbName)
			}).
				WithStartupTimeout(startupTimeout).
				WithPollInterval(pollInterval).
				WithQuery("SELECT 1"),
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
