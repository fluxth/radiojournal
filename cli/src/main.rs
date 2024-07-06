use radiojournal::crud::logger::CRUDLogger;
use radiojournal::crud::station::CRUDStation;

mod mock;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().compact().init();

    let context = radiojournal::init::initialize()
        .await
        .expect("initialize radiojournal app");

    let crud_station = CRUDStation::new(context.clone());
    let crud_logger = CRUDLogger::new(context.clone());

    mock::mock_database(context, &crud_station, &crud_logger).await;
}
