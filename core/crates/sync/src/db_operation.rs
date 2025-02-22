use rmp_serde::to_vec;
use sd_prisma::prisma::{cloud_crdt_operation, crdt_operation, instance, PrismaClient};
use sd_sync::CRDTOperation;
use uhlc::NTP64;
use uuid::Uuid;

crdt_operation::include!(crdt_include {
	instance: select { pub_id }
});

cloud_crdt_operation::include!(cloud_crdt_include {
	instance: select { pub_id }
});

impl crdt_include::Data {
	pub fn timestamp(&self) -> NTP64 {
		NTP64(self.timestamp as u64)
	}

	pub fn id(&self) -> Uuid {
		Uuid::from_slice(&self.id).unwrap()
	}

	pub fn instance(&self) -> Uuid {
		Uuid::from_slice(&self.instance.pub_id).unwrap()
	}

	pub fn into_operation(self) -> CRDTOperation {
		CRDTOperation {
			id: self.id(),
			instance: self.instance(),
			timestamp: self.timestamp(),
			record_id: rmp_serde::from_slice(&self.record_id).unwrap(),
			model: self.model,
			data: rmp_serde::from_slice(&self.data).unwrap(),
			// match self {
			// 	Self::Shared(op) => CRDTOperationType::Shared(SharedOperation {
			// 		record_id: serde_json::from_slice(&op.record_id).unwrap(),
			// 		model: op.model,
			// 		data: serde_json::from_slice(&op.data).unwrap(),
			// 	}),
			// 	Self::Relation(op) => CRDTOperationType::Relation(RelationOperation {
			// 		relation: op.relation,
			// 		data: serde_json::from_slice(&op.data).unwrap(),
			// 		relation_item: serde_json::from_slice(&op.item_id).unwrap(),
			// 		relation_group: serde_json::from_slice(&op.group_id).unwrap(),
			// 	}),
			// },
		}
	}
}

impl cloud_crdt_include::Data {
	pub fn timestamp(&self) -> NTP64 {
		NTP64(self.timestamp as u64)
	}

	pub fn id(&self) -> Uuid {
		Uuid::from_slice(&self.id).unwrap()
	}

	pub fn instance(&self) -> Uuid {
		Uuid::from_slice(&self.instance.pub_id).unwrap()
	}

	pub fn into_operation(self) -> CRDTOperation {
		CRDTOperation {
			id: self.id(),
			instance: self.instance(),
			timestamp: self.timestamp(),
			record_id: rmp_serde::from_slice(&self.record_id).unwrap(),
			model: self.model,
			data: serde_json::from_slice(&self.data).unwrap(),
			// match self {
			// 	Self::Shared(op) => ,
			// 	Self::Relation(op) => CRDTOperationType::Relation(RelationOperation {
			// 		relation: op.relation,
			// 		data: serde_json::from_slice(&op.data).unwrap(),
			// 		relation_item: serde_json::from_slice(&op.item_id).unwrap(),
			// 		relation_group: serde_json::from_slice(&op.group_id).unwrap(),
			// 	}),
			// },
		}
	}
}

pub async fn write_crdt_op_to_db(
	op: &CRDTOperation,
	db: &PrismaClient,
) -> Result<(), prisma_client_rust::QueryError> {
	crdt_op_db(op).to_query(db).exec().await?;

	Ok(())
}

fn crdt_op_db(op: &CRDTOperation) -> crdt_operation::Create {
	crdt_operation::Create {
		id: op.id.as_bytes().to_vec(),
		timestamp: op.timestamp.0 as i64,
		instance: instance::pub_id::equals(op.instance.as_bytes().to_vec()),
		kind: op.kind().to_string(),
		data: to_vec(&op.data).unwrap(),
		model: op.model.to_string(),
		record_id: rmp_serde::to_vec(&op.record_id).unwrap(),
		_params: vec![],
	}
}
