use sd_core_sync::*;
use sd_prisma::{prisma, prisma_sync};
use sd_sync::*;
use sd_utils::uuid_to_bytes;

use prisma_client_rust::chrono::Utc;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::broadcast;
use uuid::Uuid;

fn db_path(id: Uuid) -> String {
	format!("/tmp/test-{id}.db")
}

#[derive(Clone)]
struct Instance {
	id: Uuid,
	db: Arc<prisma::PrismaClient>,
	sync: Arc<sd_core_sync::Manager>,
}

impl Instance {
	async fn new(id: Uuid) -> (Arc<Self>, broadcast::Receiver<SyncMessage>) {
		let url = format!("file:{}", db_path(id));

		println!("new -1: {url}");

		let db = Arc::new(
			prisma::PrismaClient::_builder()
				.with_url(url.to_string())
				.build()
				.await
				.unwrap(),
		);

		println!("new 0: {url}");

		db._db_push().await.unwrap();

		println!("new 1");

		db.instance()
			.create(
				uuid_to_bytes(id),
				vec![],
				vec![],
				format!("Instace {id}"),
				0,
				Utc::now().into(),
				Utc::now().into(),
				vec![],
			)
			.exec()
			.await
			.unwrap();

		println!("new 2");

		let sync = sd_core_sync::Manager::new(
			&db,
			id,
			&Arc::new(AtomicBool::new(true)),
			Default::default(),
		);

		(
			Arc::new(Self {
				id,
				db,
				sync: Arc::new(sync.manager),
			}),
			sync.rx,
		)
	}

	async fn teardown(&self) {
		tokio::fs::remove_file(db_path(self.id)).await.unwrap();
	}

	async fn pair(left: &Self, right: &Self) {
		left.db
			.instance()
			.create(
				uuid_to_bytes(right.id),
				vec![],
				vec![],
				String::new(),
				0,
				Utc::now().into(),
				Utc::now().into(),
				vec![],
			)
			.exec()
			.await
			.unwrap();

		right
			.db
			.instance()
			.create(
				uuid_to_bytes(left.id),
				vec![],
				vec![],
				String::new(),
				0,
				Utc::now().into(),
				Utc::now().into(),
				vec![],
			)
			.exec()
			.await
			.unwrap();
	}
}

#[tokio::test]
async fn bruh() -> Result<(), Box<dyn std::error::Error>> {
	let (instance1, mut sync_rx1) = Instance::new(Uuid::new_v4()).await;
	let (instance2, mut sync_rx2) = Instance::new(Uuid::new_v4()).await;

	Instance::pair(&instance1, &instance2).await;

	let task_1 = tokio::spawn({
		let _instance1 = instance1.clone();
		let instance2 = instance2.clone();

		async move {
			while let Ok(msg) = sync_rx1.recv().await {
				if matches!(msg, SyncMessage::Created) {
					instance2
						.sync
						.ingest
						.event_tx
						.send(ingest::Event::Notification)
						.await
						.unwrap();
				}
			}
		}
	});

	let task_2 = tokio::spawn({
		let instance1 = instance1.clone();
		let instance2 = instance2.clone();

		async move {
			while let Some(msg) = instance2.sync.ingest.req_rx.lock().await.recv().await {
				match msg {
					ingest::Request::Messages { timestamps, .. } => {
						let messages = instance1
							.sync
							.get_ops(GetOpsArgs {
								clocks: timestamps,
								count: 100,
							})
							.await
							.unwrap();

						let ingest = &instance2.sync.ingest;

						ingest
							.event_tx
							.send(ingest::Event::Messages(ingest::MessagesEvent {
								messages,
								has_more: false,
								instance_id: instance1.id,
							}))
							.await
							.unwrap();
					}
					ingest::Request::Ingested => {
						instance2.sync.tx.send(SyncMessage::Ingested).ok();
					}
					_ => todo!(),
				}
			}
		}
	});

	instance1
		.sync
		.write_ops(&instance1.db, {
			let id = Uuid::new_v4();

			use prisma::location;

			let (sync_ops, db_ops): (Vec<_>, Vec<_>) = [
				sync_db_entry!("Location 0".to_string(), location::name),
				sync_db_entry!("/User/Brendan/Documents".to_string(), location::path),
			]
			.into_iter()
			.unzip();

			(
				instance1.sync.shared_create(
					prisma_sync::location::SyncId {
						pub_id: uuid_to_bytes(id),
					},
					sync_ops,
				),
				instance1.db.location().create(uuid_to_bytes(id), db_ops),
			)
		})
		.await?;

	assert!(matches!(sync_rx2.recv().await?, SyncMessage::Ingested));

	let out = instance2
		.sync
		.get_ops(GetOpsArgs {
			clocks: vec![],
			count: 100,
		})
		.await?;

	assert_eq!(out.len(), 3);

	instance1.teardown().await;
	instance2.teardown().await;

	task_1.abort();
	task_2.abort();

	Ok(())
}
