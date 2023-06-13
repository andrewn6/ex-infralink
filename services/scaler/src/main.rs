use std::{env, task::Context};

use juniper::{EmptyMutation, EmptySubscription, RootNode, Context};
use warp::{http::Response, Filter};

struct Query;

#[juniper::graphql_object]
impl Query {
    fn hello() -> &str {
        "Hello, world!"
    }
}

type Schema = RootNode<'static, juniper::OperationType, EmptyMutation<Box<dyn juniper::Context>>, EmptySubscription<Box<dyn juniper::Context>>>;

fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), EmptySubscription::new())
}

#[tokio::main]
async fn main() {
	env::set_var("RUST_LOG", "warp_server");
	env_logger::init();

	let log = warp::log("warp_server");

	let homepage = warp::path::end().map(|| {
		Response::builder()
			.header("content-type", "text/html")
			.body(format!(
				"<h1>Infralink Scaler</h1> \
				 GraphQL API at /graphql."
			))
	});
	
	log("Listening on 127.0.0.1:8087");

	let state = warp::any().map(move || Context::new());
	let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

	warp::serve(
		warp::get()
		.and(warp::path("graphiql"))
		.and(juniper_warp::graphiql_filter("/graphql", None))
		.or(homepage)
		.or(warp::path("graphql").and(graphql_filter))
		.with(log),
	)
	.run(([127, 0, 0, 1], 8087))
	.await;
}