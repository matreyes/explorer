use crate::{ExDataFrame, ExLazyFrame, ExplorerError};
use polars::prelude::IntoLazy;
use polars::sql::SQLContext;
use rustler::{NifStruct, Resource, ResourceArc};
use std::sync::{Arc, Mutex};

// SQLContext resource wrapper for explicit context management
pub struct ExSQLContextRef(pub Arc<Mutex<SQLContext>>);

#[rustler::resource_impl]
impl Resource for ExSQLContextRef {}

#[derive(NifStruct)]
#[module = "Explorer.PolarsBackend.SQLContext"]
pub struct ExSQLContext {
    pub resource: ResourceArc<ExSQLContextRef>,
}

impl ExSQLContextRef {
    pub fn new(ctx: SQLContext) -> Self {
        Self(Arc::new(Mutex::new(ctx)))
    }
}

impl ExSQLContext {
    pub fn new(ctx: SQLContext) -> Self {
        Self {
            resource: ResourceArc::new(ExSQLContextRef::new(ctx)),
        }
    }

    pub fn lock_inner(&self) -> std::sync::MutexGuard<'_, SQLContext> {
        self.resource.0.lock().unwrap()
    }
}

// Functions for explicit SQLContext management (advanced API)

#[rustler::nif]
pub fn sql_context_new() -> ExSQLContext {
    let ctx = SQLContext::new();
    ExSQLContext::new(ctx)
}

#[rustler::nif]
pub fn sql_context_register(context: ExSQLContext, name: &str, df: ExDataFrame) {
    let mut ctx = context.lock_inner();
    let ldf = df.clone_inner().lazy();
    ctx.register(name, ldf);
}

#[rustler::nif]
pub fn sql_context_unregister(context: ExSQLContext, name: &str) {
    let mut ctx = context.lock_inner();
    ctx.unregister(name);
}

#[rustler::nif]
pub fn sql_context_execute(
    context: ExSQLContext,
    query: &str,
) -> Result<ExLazyFrame, ExplorerError> {
    let mut ctx = context.lock_inner();
    match ctx.execute(query) {
        Ok(lazy_frame) => Ok(ExLazyFrame::new(lazy_frame)),
        Err(e) => Err(ExplorerError::Other(format!(
            "Failed to execute query: {}",
            e
        ))),
    }
}

#[rustler::nif]
pub fn sql_context_get_tables(context: ExSQLContext) -> Vec<String> {
    let ctx = context.lock_inner();
    ctx.get_tables()
}

// Simplified function for multiple DataFrames (José Valim's proposed API)
// This is the primary API that takes a list of (table_name, dataframe) tuples
// and executes a SQL query against them.

#[rustler::nif]
pub fn sql_execute(
    tables: Vec<(String, ExDataFrame)>,
    query: String,
) -> Result<ExLazyFrame, ExplorerError> {
    let mut ctx = SQLContext::new();

    // Register each table
    for (name, df) in tables {
        let ldf = df.clone_inner().lazy();
        ctx.register(&name, ldf);
    }

    // Execute the query
    match ctx.execute(&query) {
        Ok(lazy_frame) => Ok(ExLazyFrame::new(lazy_frame)),
        Err(e) => Err(ExplorerError::Other(format!(
            "Failed to execute query: {}",
            e
        ))),
    }
}
