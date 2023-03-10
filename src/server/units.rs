use actix_web::{get, post, delete, patch, Responder, HttpResponse, web, Scope};
use sqlx::{
    FromRow,
};

use super::AppState;
use serde::{Serialize, Deserialize};
use serde_json::json;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Unit {
    pub id: i32,
    pub title: String,
    pub content: String,
}

#[get("/get/{id}")]
pub async fn get(app_state: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();

    match sqlx::query_as::<_, Unit>("SELECT id, title, content FROM units WHERE id = $1")
        .bind(id as i32)
        .fetch_one(&app_state.db).await
    {
        Ok(units) => HttpResponse::Ok().json(units),
        Err(_) => HttpResponse::NotFound().json("Unit not found")
    }
}

#[derive(Deserialize, Clone)]
pub struct CreateRequest {
    pub title: String,
    pub content: String,
}

#[post("/create")]
pub async fn create(app_state: web::Data<AppState>, body: web::Json<CreateRequest>) -> impl Responder {
    let response: Result<(i32,), _> = sqlx::query_as(
        "INSERT INTO units (title, content) VALUES ($1, $2) RETURNING id"
    )
    .bind(body.title.clone())
    .bind(body.content.clone())
    .fetch_one(&app_state.db).await;

    match response {
        Ok(id) => HttpResponse::Ok().json(id),
        Err(err) => {
            dbg!(err);
            HttpResponse::InternalServerError().json("failed to create unit")
        },
    }
}

#[delete("/delete/{id}")]
pub async fn del(app_state: web::Data<AppState>, path: web::Path<i32>) -> impl Responder {
    let id = path.into_inner();

    let response: Result<(i32,), _> = sqlx::query_as(
        "DELETE FROM units WHERE id = $1 RETURNING id"
    )
    .bind(id as i32)
    .fetch_one(&app_state.db)
    .await;

    match response {
        Ok((id,)) => HttpResponse::Ok().json(json!({
            "message": format!("successfully deleted unit with id: {}", id)
        })),

        Err(error) => {
            dbg!(error);
            HttpResponse::InternalServerError().json("failed to delete unit")
        }
    }
}

#[derive(Deserialize, Clone)]
pub struct UpdateRequest {
    pub title: Option<String>,
    pub content: Option<String>
}

#[patch("/update/{id}")]
pub async fn update(app_state: web::Data<AppState>, path: web::Path<i32>, body: web::Json<UpdateRequest>) -> impl Responder {
    let id = path.into_inner();

    let result: Result<(), HttpResponse> = match body.title.clone() {
        Some(title) => {
            let result: Result<Unit, _> = sqlx::query_as(
                "UPDATE units SET title = $1 WHERE id = $2 RETURNING title, content, id"
            )
            .bind(title)
            .bind(id)
            .fetch_one(&app_state.db)
            .await;

            match result {
                Ok(_unit) => Ok(()),
                Err(error) => {
                    dbg!(error);
                    Err(HttpResponse::InternalServerError().json(json!({
                        "message": "error updating title"
                    })))
                }
            }
        },
        None => {Ok(())}
    };

    if let Err(response) = result {
        return response;
    }

    let result: Result<(), HttpResponse> = match body.content.clone() {
        Some(content) => {
            let result: Result<Unit, _> = sqlx::query_as(
                "UPDATE units SET content = $1 WHERE id = $2 RETURNING title, content, id"
            )
            .bind(content)
            .bind(id)
            .fetch_one(&app_state.db)
            .await;

            match result {
                Ok(_unit) => Ok(()),
                Err(error) => {
                    dbg!(error);
                    Err(HttpResponse::InternalServerError().json(json!({
                        "message": "error updating content"
                    })))
                }
            }
        },
        None => {Ok(())}
    };

    if let Err(response) = result {
        return response;
    }

    HttpResponse::Ok().json(json!({
        "message": "successfully updated!"
    }))
}

pub fn service() -> Scope {
    web::scope("/units")
        .service(get)
        .service(create)
        .service(del)
        .service(update)
}