package testutils

import (
	"context"
	"database/sql"
	"log"
	"log/slog"
	"testing"

	sqlc "github.com/green-ecolution/backend/internal/storage/postgres/_sqlc"
	"github.com/green-ecolution/backend/internal/storage/postgres/store"
	"github.com/jackc/pgx/v5"
	"github.com/jackc/pgx/v5/pgxpool"
	_ "github.com/jackc/pgx/v5/stdlib"
	"github.com/testcontainers/testcontainers-go/modules/postgres"
	"github.com/twpayne/go-geos"
	pgxgeos "github.com/twpayne/pgx-geos"
)

type PostgresTestSuite struct {
	Container *postgres.PostgresContainer
	URL       string
	Store     *store.Store
	snapshot  string
}

func SetupPostgresTestSuite(ctx context.Context) *PostgresTestSuite {
	log.Println("Setting up test suite...")
	testSuite := PostgresTestSuite{
		snapshot: "test_db",
	}
	container := SetupPostgres(ctx)
	testSuite.Container = container

	dbURL, err := container.ConnectionString(ctx)
	if err != nil {
		log.Fatalf("Could not get connection string: %s", err)
	}
	testSuite.URL = dbURL

	// Migrate database
	db, err := sql.Open("pgx", dbURL)
	if err != nil {
		log.Fatalf("Could not connect to postgres: %s", err)
	}

	ExecMigration(db)

	err = db.Close()
	if err != nil {
		log.Fatalf("Could not close database connection: %s", err)
	}

	// Take a snapshot without data
	slog.Info("Taking snapshot...")
	if err = container.Snapshot(ctx, postgres.WithSnapshotName(testSuite.snapshot)); err != nil {
		log.Fatalf("Could not create snapshot: %s", err)
	}
	testSuite.Store = initStore(dbURL)

	return &testSuite
}

// ResetDB resets the database to the state of the snapshot without data
func (s *PostgresTestSuite) ResetDB(t testing.TB) {
	t.Helper()
	t.Log("Resetting database...")

	if err := s.Container.Restore(context.Background(), postgres.WithSnapshotName(s.snapshot)); err != nil {
		t.Fatalf("Could not restore snapshot: %s", err)
	}
	s.Store = initStore(s.URL)
}

func (s *PostgresTestSuite) TakeSnapshot(t testing.TB, snapshotName string) {
	t.Helper()
	t.Log("Taking snapshot...")

	s.CloseConnTemporary(t)
	if err := s.Container.Snapshot(context.Background(), postgres.WithSnapshotName(snapshotName)); err != nil {
		t.Fatalf("Could not create snapshot: %s", err)
	}
}

func initStore(dbURL string) *store.Store {
	pgxConfig, err := pgxpool.ParseConfig(dbURL)
	if err != nil {
		log.Fatalf("Error while parsing PostgreSQL connection string: %s", err)
	}

	pgxConfig.AfterConnect = func(ctx context.Context, conn *pgx.Conn) error {
		return pgxgeos.Register(ctx, conn, geos.NewContext())
	}

	pool, err := pgxpool.NewWithConfig(context.Background(), pgxConfig)
	if err != nil {
		log.Fatalf("Error while connecting to PostgreSQL: %s", err)
	}

	s := store.NewStore(pool, sqlc.New(pool))

	return s
}

// CloseConn closes the connection. After the test, the connection will be re-established.
func (s *PostgresTestSuite) CloseConnTemporary(t testing.TB) {
	t.Helper()
	t.Log("Closing connection...")
	s.Store.Close()

	t.Cleanup(func() {
		s.Store = initStore(s.URL)
	})
}

func (s *PostgresTestSuite) ExecQuery(t testing.TB, query string, args ...any) (pgx.Rows, error) {
	t.Helper()
	t.Log("Executing query...")

	result, err := s.Store.DB().Query(context.Background(), query, args...)
	if err != nil {
		return nil, err
	}

	return result, nil
}

func (s *PostgresTestSuite) SwitchQuerier(t testing.TB, querier *sqlc.Queries) {
	t.Helper()
	t.Log("Switching querier...")

	oldQuerier := s.Store.Queries
	s.Store.Queries = querier

	t.Cleanup(func() {
		t.Log("Restoring querier...")
		s.Store.Queries = oldQuerier
	})
}
