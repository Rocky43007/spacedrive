use crate::{
	library::LibraryConfigError,
	location::{indexer, LocationManagerError},
};

use sd_p2p::IdentityOrRemoteIdentityErr;
use sd_utils::{
	db::{self, MissingFieldError},
	error::{FileIOError, NonUtf8PathError},
};

use thiserror::Error;
use tracing::error;

#[derive(Error, Debug)]
pub enum LibraryManagerError {
	#[error("error serializing or deserializing the JSON in the config file: {0}")]
	Json(#[from] serde_json::Error),
	#[error("database error: {0}")]
	Database(#[from] prisma_client_rust::QueryError),
	#[error("library not found error")]
	LibraryNotFound,
	#[error("failed to parse uuid: {0}")]
	Uuid(#[from] uuid::Error),
	#[error("failed to run indexer rules seeder: {0}")]
	IndexerRulesSeeder(#[from] indexer::rules::seed::SeederError),
	// #[error("failed to initialize the key manager: {0}")]
	// KeyManager(#[from] sd_crypto::Error),
	#[error("error migrating the library: {0}")]
	MigrationError(#[from] db::MigrationError),
	#[error("invalid library configuration: {0}")]
	InvalidConfig(String),
	#[error(transparent)]
	NonUtf8Path(#[from] NonUtf8PathError),
	#[error("failed to watch locations: {0}")]
	LocationWatcher(#[from] LocationManagerError),
	#[error("failed to parse library p2p identity: {0}")]
	Identity(#[from] IdentityOrRemoteIdentityErr),
	#[error("failed to load private key for instance p2p identity")]
	InvalidIdentity,
	#[error("current instance with id '{0}' was not found in the database")]
	CurrentInstanceNotFound(String),
	#[error("missing-field: {0}")]
	MissingField(#[from] MissingFieldError),

	#[error(transparent)]
	FileIO(#[from] FileIOError),
	#[error(transparent)]
	LibraryConfig(#[from] LibraryConfigError),
}

impl From<LibraryManagerError> for rspc::Error {
	fn from(error: LibraryManagerError) -> Self {
		rspc::Error::with_cause(
			rspc::ErrorCode::InternalServerError,
			error.to_string(),
			error,
		)
	}
}
