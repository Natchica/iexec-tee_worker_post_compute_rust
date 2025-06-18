use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, PartialEq, Clone, Error, Serialize, Deserialize)]
#[serde(rename_all(serialize = "SCREAMING_SNAKE_CASE"))]
#[allow(clippy::enum_variant_names)]
pub enum ReplicateStatusCause {
    #[error("computed.json file missing")]
    PostComputeComputedFileNotFound,
    #[error("Task ID related environment variable is missing")]
    PostComputeTaskIdMissing,
    #[error("Unexpected error occurred")]
    PostComputeFailedUnknownIssue,
    #[error("Invalid enclave challenge private key")]
    PostComputeInvalidEnclaveChallengePrivateKey,
    #[error("Invalid TEE signature")]
    PostComputeInvalidTeeSignature,
    #[error("Failed to upload to IPFS")]
    PostComputeIpfsUploadFailed,
    #[error("Failed to zip result folder")]
    PostComputeOutFolderZipFailed,
    #[error("Empty resultDigest")]
    PostComputeResultDigestComputationFailed,
    #[error("Result file not found")]
    PostComputeResultFileNotFound,
    #[error("Failed to send computed file")]
    PostComputeSendComputedFileFailed,
    #[error("Storage token related environment variable is missing")]
    PostComputeStorageTokenMissing,
    #[error("Tee challenge private key related environment variable is missing")]
    PostComputeTeeChallengePrivateKeyMissing,
    #[error("Result file name too long")]
    PostComputeTooLongResultFileName,
    #[error("Worker address related environment variable is missing")]
    PostComputeWorkerAddressMissing,
    #[error("Failed to encrypt result")]
    PostComputeEncryptionFailed,
    #[error("Malformed RSA public key for result encryption")]
    PostComputeMalformedEncryptionPublicKey,
    #[error("Result encryption public key is missing")]
    PostComputeEncryptionPublicKeyMissing,
}
