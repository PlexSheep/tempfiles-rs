pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250507_105126_file_tracking;
mod m20250528_144714_user_apitok;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250507_105126_file_tracking::Migration),
            Box::new(m20250528_144714_user_apitok::Migration),
        ]
    }
}
