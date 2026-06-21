use async_trait::async_trait;
use loco_rs::prelude::*;

use crate::models::users::{Model, RegisterParams};

pub struct CreateUser;

#[async_trait]
impl Task for CreateUser {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "users.create".to_string(),
            detail: "Create a local NOVK user. Usage: cargo loco task users.create email:me@example.com password:secret name:Admin".to_string(),
        }
    }

    async fn run(&self, app_context: &AppContext, vars: &task::Vars) -> Result<()> {
        let email = vars.cli_arg("email")?.trim().to_string();
        let password = vars.cli_arg("password")?.to_string();
        let name = vars.cli_arg("name")?.trim().to_string();

        let user = Model::create_with_password(
            &app_context.db,
            &RegisterParams {
                email,
                password,
                name,
            },
        )
        .await?;

        println!("created user {} ({})", user.email, user.pid);

        Ok(())
    }
}
