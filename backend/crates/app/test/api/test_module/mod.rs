use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
};

use axum::{extract::State, http::StatusCode};
use domain::{
    Id, RepositoryError,
    authorization::{Action, Permission, ResourceKey},
    event_bus::{EventHandler, EventHandlerError},
    events::DomainEvent,
    module::ModuleKey,
    organization::Organization,
    shared::coordinates::Coordinate,
    target::{TargetCatalog, TargetError, TargetKindSpec, TargetPurpose, TargetRef},
};
use serde::{Deserialize, Serialize};
use server::{
    http::{AppState, auth::extractor::AuthUserExtractor, extractors::Json},
    module::{Module, ModuleBuildError, ModuleContext, ModuleRoutes, ModuleRuntime},
    service::{ServiceError, module_access::ModuleAccess},
};
use sqlx::{PgPool, migrate::Migrator};
use utoipa_axum::{router::OpenApiRouter, routes};
use uuid::Uuid;

pub const KEY: &str = "test_module";
static MIGRATOR: Migrator = sqlx::migrate!("test/api/test_module/migrations");

/// What the tests observe from outside.
#[derive(Default)]
pub struct Probe {
    pub organizations_created: AtomicUsize,
    pub task_ran: AtomicBool,
}

pub struct TestModule {
    pub probe: Arc<Probe>,
}

impl TestModule {
    pub fn new(probe: Arc<Probe>) -> Self {
        Self { probe }
    }
}

struct TestState {
    pool: PgPool,
    access: ModuleAccess,
}

#[derive(Deserialize, utoipa::ToSchema)]
struct CreateThing {
    organization_id: Uuid,
    #[serde(default)]
    label: Option<String>,
    #[serde(default)]
    latitude: Option<f64>,
    #[serde(default)]
    longitude: Option<f64>,
}

#[derive(Serialize, utoipa::ToSchema)]
struct Thing {
    id: Uuid,
    organization_id: Uuid,
}

pub const THING: ResourceKey = ResourceKey::declare("test_module_thing");

fn permission() -> Permission {
    Permission::new(ResourceKey::ORGANIZATION, Action::Read)
}

#[utoipa::path(post, path = "/test-module/things", request_body = CreateThing, responses((status = 201, body = Thing)))]
async fn create_thing(
    State(state): State<Arc<TestState>>,
    user: AuthUserExtractor,
    Json(req): Json<CreateThing>,
) -> Result<(StatusCode, Json<Thing>), ServiceError> {
    state
        .access
        .require_write(user.id, permission(), Id::new(req.organization_id))
        .await?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO test_module_things (id, organization_id, label, latitude, longitude)
         VALUES ($1, $2, COALESCE($3, 'thing'), $4, $5)",
    )
    .bind(id)
    .bind(req.organization_id)
    .bind(req.label)
    .bind(req.latitude)
    .bind(req.longitude)
    .execute(&state.pool)
    .await
    .map_err(|e| ServiceError::from(domain::RepositoryError::from(e)))?;
    Ok((
        StatusCode::CREATED,
        Json(Thing {
            id,
            organization_id: req.organization_id,
        }),
    ))
}

#[utoipa::path(get, path = "/test-module/things", responses((status = 200, body = Vec<Thing>)))]
async fn list_things(
    State(state): State<Arc<TestState>>,
    user: AuthUserExtractor,
) -> Result<Json<Vec<Thing>>, ServiceError> {
    let visible = state.access.visible_orgs(user.id, permission()).await?;
    let orgs = visible.into_raw_ids().unwrap_or_default();
    let rows: Vec<(Uuid, Uuid)> = sqlx::query_as(
        "SELECT id, organization_id FROM test_module_things WHERE organization_id = ANY($1)",
    )
    .bind(&orgs)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ServiceError::from(domain::RepositoryError::from(e)))?;
    Ok(Json(
        rows.into_iter()
            .map(|(id, organization_id)| Thing {
                id,
                organization_id,
            })
            .collect(),
    ))
}

fn routes() -> OpenApiRouter<Arc<TestState>> {
    OpenApiRouter::new().routes(routes!(create_thing, list_things))
}

pub const THING_KIND: &str = "thing";

const THING_KINDS: [TargetKindSpec; 1] = [TargetKindSpec {
    kind: THING_KIND,
    accepts_sensors: true,
    commentable: true,
    resource: THING,
}];

/// Things as targets, so tests can attach core objects to another module's rows.
struct ThingCatalog {
    pool: PgPool,
}

impl ThingCatalog {
    fn ids(targets: &[TargetRef]) -> Vec<Uuid> {
        targets
            .iter()
            .filter(|t| t.module.as_str() == KEY && t.kind.as_str() == THING_KIND)
            .map(|t| t.id)
            .collect()
    }

    fn target(id: Uuid) -> TargetRef {
        TargetRef::new(KEY, THING_KIND, id).expect("static keys are valid")
    }
}

#[async_trait::async_trait]
impl TargetCatalog for ThingCatalog {
    fn kinds(&self) -> &'static [TargetKindSpec] {
        &THING_KINDS
    }

    async fn admit(
        &self,
        target: &TargetRef,
        _purpose: TargetPurpose,
    ) -> Result<Id<Organization>, TargetError> {
        if target.kind.as_str() != THING_KIND {
            return Err(TargetError::NotFound);
        }
        let org: Option<Uuid> =
            sqlx::query_scalar("SELECT organization_id FROM test_module_things WHERE id = $1")
                .bind(target.id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| TargetError::Repository(e.into()))?;
        org.map(Id::new).ok_or(TargetError::NotFound)
    }

    async fn locate(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, Coordinate>, RepositoryError> {
        let rows: Vec<(Uuid, f64, f64)> = sqlx::query_as(
            "SELECT id, latitude, longitude FROM test_module_things
             WHERE id = ANY($1) AND latitude IS NOT NULL AND longitude IS NOT NULL",
        )
        .bind(Self::ids(targets))
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(id, lat, lng)| Ok((Self::target(id), Coordinate::new(lat, lng)?)))
            .collect()
    }

    async fn label(
        &self,
        targets: &[TargetRef],
    ) -> Result<HashMap<TargetRef, String>, RepositoryError> {
        let rows: Vec<(Uuid, String)> =
            sqlx::query_as("SELECT id, label FROM test_module_things WHERE id = ANY($1)")
                .bind(Self::ids(targets))
                .fetch_all(&self.pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(id, label)| (Self::target(id), label))
            .collect())
    }
}

struct CountOrganizations(Arc<Probe>);

#[async_trait::async_trait]
impl EventHandler for CountOrganizations {
    fn name(&self) -> &str {
        "test_module.count_organizations"
    }

    async fn handle(&self, event: &DomainEvent) -> Result<Vec<DomainEvent>, EventHandlerError> {
        if matches!(event, DomainEvent::OrganizationCreated { .. }) {
            self.0.organizations_created.fetch_add(1, Ordering::SeqCst);
        }
        Ok(vec![])
    }
}

impl Module for TestModule {
    fn key(&self) -> ModuleKey {
        ModuleKey::new(KEY).expect("the test module key is valid")
    }

    fn resources(&self) -> &'static [ResourceKey] {
        &[THING]
    }

    fn default_enabled(&self) -> bool {
        false
    }

    fn openapi(&self) -> utoipa::openapi::OpenApi {
        routes().into_openapi()
    }

    fn migrations(&self) -> Vec<sqlx::migrate::Migration> {
        MIGRATOR.iter().cloned().collect()
    }

    fn build(&self, ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
        let state = Arc::new(TestState {
            pool: ctx.pool.clone(),
            access: ctx.access.clone(),
        });
        let mut runtime = ModuleRuntime::new(ModuleRoutes {
            public: OpenApiRouter::new(),
            protected: routes().with_state::<Arc<AppState>>(state),
        });
        runtime.targets = Some(Arc::new(ThingCatalog {
            pool: ctx.pool.clone(),
        }));
        runtime
            .event_handlers
            .push(Arc::new(CountOrganizations(self.probe.clone())));
        let probe = self.probe.clone();
        runtime.background_tasks.push(Box::pin(async move {
            probe.task_ran.store(true, Ordering::SeqCst);
        }));
        Ok(runtime)
    }
}

/// Claims a path the core already serves; only used to prove startup refuses it.
pub struct CollidingModule;

#[utoipa::path(get, path = "/organizations")]
async fn shadow() {}

impl Module for CollidingModule {
    fn key(&self) -> ModuleKey {
        ModuleKey::new("colliding").expect("valid key")
    }
    fn default_enabled(&self) -> bool {
        false
    }
    fn openapi(&self) -> utoipa::openapi::OpenApi {
        OpenApiRouter::<()>::new()
            .routes(routes!(shadow))
            .into_openapi()
    }
    fn build(&self, _ctx: ModuleContext) -> Result<ModuleRuntime, ModuleBuildError> {
        Ok(ModuleRuntime::new(ModuleRoutes::empty()))
    }
}
