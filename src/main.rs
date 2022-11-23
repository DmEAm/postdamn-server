extern crate postdamn;

#[macro_use]
extern crate diesel;

use std::default::Default;
use diesel::{Connection, PgConnection,
             dsl::*, pg::Pg,
             debug_query, insert_into, select};
use dotenvy::dotenv;
use std::env;
use uuid::Uuid;

use postdamn::{
    models::{User, Role},
    service::{Example}};


pub fn establish_connection() -> PgConnection {
    let url = env::var("DATABASE_URL").expect("Test");
    PgConnection::establish(&url).ok().unwrap()
}

fn users_list(c: &mut PgConnection) -> Vec<User>
{
    use crate::diesel::{QueryDsl, RunQueryDsl, ExpressionMethods};
    use std::iter::Iterator;
    use postdamn::schema::security::users::dsl::*;
    let q = users
        .filter(name.eq("test"))
        .order_by(name)
        .then_order_by(id)
        .limit(5);
    println!("Run SQL {}", debug_query::<Pg, _>(&q));
    let r = q.load::<User>(c).expect("Error loading posts");

    assert!(!r.is_empty());
    let r = vec![];
    r
}

fn main() {
    dotenv().ok();


    let mut connection = establish_connection();

    {
        let mut results = users_list(&mut connection);
        println!("Displaying {}", results.len());
        println!("-----------");
        for r in results.iter_mut() {
            println!("{}", r);
            r.id = Default::default();
            println!("{}", r);
            println!("-----------");
        }

        for r in results {
            println!("{}", r);
            println!("-----------");
        }
    }
}
