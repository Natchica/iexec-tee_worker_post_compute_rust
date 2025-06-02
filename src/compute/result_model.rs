use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct ResultModel {
    #[serde(rename = "chainTaskId")]
    pub chain_task_id: String,
    #[serde(rename = "enclaveSignature")]
    pub enclave_signature: String,
    #[serde(rename = "resultDigest")]
    pub result_digest: String,
    pub storage: String, // e.g., "ipfs"
    pub filename: String,
    pub data: Vec<u8>, // File content as bytes
}

impl ResultModel {
    pub fn new(
        chain_task_id: String,
        enclave_signature: String,
        result_digest: String,
        storage: String,
        filename: String,
        data: Vec<u8>,
    ) -> Self {
        ResultModel {
            chain_task_id,
            enclave_signature,
            result_digest,
            storage,
            filename,
            data,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_result_model_serialization() {
        let result_model = ResultModel {
            chain_task_id: "0xtaskid".to_string(),
            enclave_signature: "0xsignature".to_string(),
            result_digest: "0xhash".to_string(),
            storage: "ipfs".to_string(),
            filename: "results.zip".to_string(),
            data: vec![1, 2, 3, 4, 5],
        };

        let json_string = serde_json::to_string(&result_model).unwrap();

        // Expected JSON structure based on serde rename attributes
        let expected_json = r#"{"chainTaskId":"0xtaskid","enclaveSignature":"0xsignature","resultDigest":"0xhash","storage":"ipfs","filename":"results.zip","data":[1,2,3,4,5]}"#;

        assert_eq!(json_string, expected_json);
    }

    #[test]
    fn test_result_model_serialization_empty_data() {
        let result_model = ResultModel {
            chain_task_id: "0xtaskid_empty".to_string(),
            enclave_signature: "0xsignature_empty".to_string(),
            result_digest: "0xhash_empty".to_string(),
            storage: "s3".to_string(),
            filename: "data.bin".to_string(),
            data: vec![],
        };

        let json_string = serde_json::to_string(&result_model).unwrap();

        let expected_json = r#"{"chainTaskId":"0xtaskid_empty","enclaveSignature":"0xsignature_empty","resultDigest":"0xhash_empty","storage":"s3","filename":"data.bin","data":[]}"#;
        assert_eq!(json_string, expected_json);
    }
}
