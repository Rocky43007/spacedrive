use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

use crate::cloud::sync::err_break;

// Responsible for taking sync operations received from the cloud,
// and applying them to the local database via the sync system's ingest actor.

pub async fn run_actor(sync: Arc<sd_core_sync::Manager>, notify: Arc<Notify>) {
	loop {
		{
			let mut rx = sync.ingest.req_rx.lock().await;

			if sync
				.ingest
				.event_tx
				.send(sd_core_sync::Event::Notification)
				.await
				.is_ok()
			{
				while let Some(req) = rx.recv().await {
					const OPS_PER_REQUEST: u32 = 1000;

					use sd_core_sync::*;

					let timestamps = match req {
						Request::FinishedIngesting => break,
						Request::Messages { timestamps, .. } => timestamps,
						_ => continue,
					};

					let ops = err_break!(
						sync.get_cloud_ops(GetOpsArgs {
							clocks: timestamps,
							count: OPS_PER_REQUEST,
						})
						.await
					);

					info!("Got {} cloud ops to ingest", ops.len());

					err_break!(
						sync.ingest
							.event_tx
							.send(sd_core_sync::Event::Messages(MessagesEvent {
								instance_id: sync.instance,
								has_more: ops.len() == 1000,
								messages: ops,
							}))
							.await
					);
				}
			}
		}

		notify.notified().await;
	}
}
