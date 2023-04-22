use serde::Deserialize;
use surrealdb::engine::local::{Mem, Db};
use surrealdb::opt::auth::Root;
use surrealdb::sql::Thing;
use surrealdb::Surreal;

const ACCOUNT: &str = "account";

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Account {
	id: Thing,
	balance: String,
}

static DB: Surreal<Db> = Surreal::init();

#[tokio::main]
async fn main() -> surrealdb::Result<()> {
    Db.connect::<Mem>(()).await?;

	DB.signin(Root {
		username: "root",
		password: "root",
	})
	.await?;

	DB.use_ns("namespace").use_db("database").await?;

	let accounts: Vec<Account> = DB.select(ACCOUNT).range("one".."two").await?;

	println!("{accounts:?}");

	Ok(())
}
