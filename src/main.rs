#[macro_use]
extern crate rocket;
use std::str::FromStr;

use rocket::{
    http::Status,
    request::{self, FromRequest},
    Request
};
use rocket_db_pools::{deadpool_redis::{self, redis::AsyncCommands}, Database, Connection};
use uuid::Uuid;

#[derive(Database)]
#[database("store")]
struct Store(deadpool_redis::Pool);

#[get("/private")]
async fn private(user: User) -> String {
    user.id.to_string()
}

#[post("/register")]
async fn register(mut store: Connection<Store>) -> String {
    let token_uuid = Uuid::new_v4().to_string();
    let user_uuid = Uuid::new_v4().to_string();

    let _: bool = store.set(format!("api_token:{token_uuid}"), &user_uuid).await.unwrap();

    token_uuid 
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
               private,
               register
        ])
        .attach(Store::init())
}

struct User {
    id: Uuid,
}

impl User {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let maybe_api_key = request.headers().get_one("Authorization");

        if maybe_api_key.is_none() {
            return request::Outcome::Failure((Status::Unauthorized, ()));
        }

        let api_key = maybe_api_key.unwrap();
        if !api_key.starts_with("Bearer ") {
            return request::Outcome::Failure((Status::Unauthorized, ()));
        }

        let api_key = api_key.replace("Bearer ", "");
        let mut store = request.guard::<Connection<Store>>().await.unwrap();

        let maybe_user_id = get_id_from_token(&mut store, api_key).await;
        if maybe_user_id.is_none() {
            return request::Outcome::Failure((Status::Unauthorized, ()));
        }

        return request::Outcome::Success(User::new(maybe_user_id.unwrap()));
    }
}

async fn get_id_from_token(store: &mut Connection<Store>, api_key: String) -> Option<Uuid> {
    let maybe_id: Result<String, _> = store.get(format!("api_token:{}", &api_key)).await;

    let id = match maybe_id {
        Ok(id) => Some(id),
        Err(_) => None,
    }?;

    match Uuid::from_str(&id.as_ref()) {
        Err(err) => {
            info!("Could not parse uuid \"{id}\"! ({})", err.to_string());

            return None;
        },
        Ok(uuid) => Some(uuid),
    }
}
