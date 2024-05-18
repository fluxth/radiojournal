use radiojournal::crud::station::CRUDStation;

mod mock;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();

    let context = radiojournal::init::initialize()
        .await
        .expect("initialize radiojournal app");

    let crud_station = CRUDStation::new(context.clone());

    mock::mock_database(context, &crud_station).await;
}
