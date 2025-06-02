use reqwest::blocking::Client;
use serde::Serialize;
use std::{fs, path::Path};

use crate::compute::{computed_file::ComputedFile, errors::ReplicateStatusCause, result_model::ResultModel};

const RESULTS_ENDPOINT: &str = "/v1/results";

pub fn upload_to_ipfs_with_iexec_proxy(
    computed_file: &ComputedFile,
    base_url: &str,
    token: &str,
    file_to_upload_path: &str,
) -> Result<String, ReplicateStatusCause> {
    // 1. Read the ZIP file
    let file_data = fs::read(file_to_upload_path)
        .map_err(|_| ReplicateStatusCause::PostComputeResultFileNotFound)?;

    let file_name = Path::new(file_to_upload_path)
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ReplicateStatusCause::PostComputeResultFileNotFound)? // Should not happen if fs::read worked
        .to_string();

    // 2. Create a ResultModel instance (adjust based on actual ResultModel definition)
    // Assuming ComputedFile has task_id, enclave_signature, and result_digest as Option<String>
    let task_id = computed_file.task_id.clone().ok_or(ReplicateStatusCause::PostComputeIpfsUploadFailed)?; // Or a more specific error
    let enclave_signature = computed_file.enclave_signature.clone().ok_or(ReplicateStatusCause::PostComputeIpfsUploadFailed)?;
    let result_digest = computed_file.result_digest.clone().ok_or(ReplicateStatusCause::PostComputeIpfsUploadFailed)?;

    // The ResultModel struct itself is now defined.
    let result_model = ResultModel::new(
        task_id,
        enclave_signature,
        result_digest,
        "ipfs".to_string(), // Assuming storage is always "ipfs" for this function
        file_name,
        file_data,
    );

    // 3. Make a POST request
    let client = Client::new();
    let url = format!("{}{}", base_url, RESULTS_ENDPOINT);

    let response = client
        .post(&url)
        .header("Authorization", token)
        .json(&result_model) // Serialize ResultModel to JSON
        .send()
        .map_err(|e| {
            log::error!("IPFS upload request failed: {:?}", e);
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        })?;

    // 4. Handle HTTP errors
    if !response.status().is_success() {
        log::error!(
            "IPFS upload failed with status: {} and body: {:?}",
            response.status(),
            response.text() // Consumes body, careful if you need to re-read
        );
        return Err(ReplicateStatusCause::PostComputeIpfsUploadFailed);
    }

    // 5. Return the IPFS hash/link
    response
        .text() // Assuming the body is plain text IPFS hash/link
        .map_err(|e| {
            log::error!("Failed to read IPFS upload response body: {:?}", e);
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compute::computed_file::ComputedFile;
    use crate::compute::errors::ReplicateStatusCause;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use wiremock::{
        matchers::{body_json, header, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    fn create_dummy_computed_file() -> ComputedFile {
        ComputedFile {
            deterministic_output_path: Some("/path/to/output".to_string()),
            task_id: Some("0xtaskid".to_string()),
            result_digest: Some("0xresultdigest".to_string()),
            enclave_signature: Some("0xenclavesignature".to_string()),
            callback_data: None,
            error_message: None,
        }
    }

    fn create_dummy_zip_file(dir: &tempfile::TempDir, content: &[u8]) -> String {
        let zip_path = dir.path().join("test_upload.zip");
        let mut file = File::create(&zip_path).unwrap();
        file.write_all(content).unwrap();
        zip_path.to_str().unwrap().to_string()
    }

    #[tokio::test]
    async fn test_upload_success() {
        let server = MockServer::start().await;
        let computed_file = create_dummy_computed_file();
        let temp_dir = tempdir().unwrap();
        let zip_content = b"zip file content";
        let zip_path = create_dummy_zip_file(&temp_dir, zip_content);

        let expected_ipfs_hash = "QmTestHash";

        let result_model_payload = ResultModel::new(
            computed_file.task_id.clone().unwrap(),
            computed_file.enclave_signature.clone().unwrap(),
            computed_file.result_digest.clone().unwrap(),
            "ipfs".to_string(),
            "test_upload.zip".to_string(),
            zip_content.to_vec(),
        );

        Mock::given(method("POST"))
            .and(path(RESULTS_ENDPOINT))
            .and(header("Authorization", "test_token"))
            .and(body_json(&result_model_payload))
            .respond_with(ResponseTemplate::new(200).set_body_string(expected_ipfs_hash))
            .mount(&server)
            .await;

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            &server.uri(),
            "test_token",
            &zip_path,
        );

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_ipfs_hash);
        // Attempting explicit async close, though MockServer does not have close_async()
        // server.reset().await; // Resetting mocks
        // Forcing the server to drop before the runtime of the test finishes.
        // This is a bit of a guess, hoping it influences the drop order favorably.
        let _ = server;
    }

    // This test does not involve async Wiremock, so no #[tokio::test] needed
    #[test]
    fn test_upload_file_not_found() {
        //let server = MockServer::start().await; // Not actually used but fn needs it
        let computed_file = create_dummy_computed_file();

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            "http://localhost:1234", // Dummy URL, server not started
            "test_token",
            "/non/existent/path.zip",
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeResultFileNotFound
        );
    }

    #[tokio::test]
    async fn test_upload_server_error() {
        let server = MockServer::start().await;
        let computed_file = create_dummy_computed_file();
        let temp_dir = tempdir().unwrap();
        let zip_path = create_dummy_zip_file(&temp_dir, b"content");

        Mock::given(method("POST"))
            .and(path(RESULTS_ENDPOINT))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            &server.uri(),
            "test_token",
            &zip_path,
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        );
        let _ = server;
    }

    #[tokio::test]
    async fn test_upload_auth_error() {
        let server = MockServer::start().await;
        let computed_file = create_dummy_computed_file();
        let temp_dir = tempdir().unwrap();
        let zip_path = create_dummy_zip_file(&temp_dir, b"content");

        Mock::given(method("POST"))
            .and(path(RESULTS_ENDPOINT))
            .and(header("Authorization", "wrong_token")) // Mock expects "test_token"
            .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
            .mount(&server)
            .await;

        // This second mock for the same path might cause issues if not ordered or if it's too broad.
        // For a specific auth error test, it's better to make the first mock more specific
        // or ensure only one relevant mock is active.
        // Let's rely on the first mock with "wrong_token" to simulate the auth failure.
        // The server should return 401 if the token is "wrong_token".
        // If we send "test_token" (as in success test), it would pass if not for the specific header check.

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            &server.uri(),
            "wrong_token", // Use the token that the mock is set to reject
            &zip_path,
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        );
        let _ = server;
    }

    // These tests do not involve async Wiremock, so no #[tokio::test] needed
    #[test]
    fn test_upload_missing_task_id_in_computed_file() {
        //let server = MockServer::start().await; // Not needed
        let mut computed_file = create_dummy_computed_file();
        computed_file.task_id = None;
        let temp_dir = tempdir().unwrap();
        let zip_path = create_dummy_zip_file(&temp_dir, b"content");

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            "http://dummyurl", // server not used
            "test_token",
            &zip_path,
        );

        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        );
    }

    #[test]
    fn test_upload_missing_enclave_signature_in_computed_file() {
        //let server = MockServer::start().await; // Not needed
        let mut computed_file = create_dummy_computed_file();
        computed_file.enclave_signature = None;
        let temp_dir = tempdir().unwrap();
        let zip_path = create_dummy_zip_file(&temp_dir, b"content");

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            "http://dummyurl", // server not used
            "test_token",
            &zip_path,
        );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        );
    }

    #[test]
    fn test_upload_missing_result_digest_in_computed_file() {
        //let server = MockServer::start().await; // Not needed
        let mut computed_file = create_dummy_computed_file();
        computed_file.result_digest = None;
        let temp_dir = tempdir().unwrap();
        let zip_path = create_dummy_zip_file(&temp_dir, b"content");

        let result = upload_to_ipfs_with_iexec_proxy(
            &computed_file,
            "http://dummyurl", // server not used
            "test_token",
            &zip_path,
        );
        assert!(result.is_err());
        assert_eq!(
            result.err().unwrap(),
            ReplicateStatusCause::PostComputeIpfsUploadFailed
        );
    }
}
