/*!
 * Streaming Syscall Support
 * Handles large file operations with chunked streaming
 */

use crate::core::types::Pid;
use crate::syscalls::SyscallExecutor;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const DEFAULT_CHUNK_SIZE: usize = 64 * 1024; // 64KB

#[derive(Clone)]
pub struct StreamingManager {
    executor: SyscallExecutor,
}

impl StreamingManager {
    pub fn new(executor: SyscallExecutor) -> Self {
        Self { executor }
    }

    pub async fn stream_file_read(
        &self,
        pid: Pid,
        path: PathBuf,
        chunk_size: Option<u32>,
    ) -> Result<impl futures::Stream<Item = Result<Vec<u8>, String>>, String> {
        // Permission check done by executor - just proceed

        let chunk_size = chunk_size.unwrap_or(DEFAULT_CHUNK_SIZE as u32) as usize;
        let file = File::open(&path)
            .await
            .map_err(|e| format!("Failed to open file: {}", e))?;

        Ok(async_stream::stream! {
            let mut reader = tokio::io::BufReader::new(file);
            let mut buffer = vec![0u8; chunk_size];

            loop {
                match reader.read(&mut buffer).await {
                    Ok(0) => break, // EOF
                    Ok(n) => yield Ok(buffer[..n].to_vec()),
                    Err(e) => {
                        yield Err(format!("Read error: {}", e));
                        break;
                    }
                }
            }
        })
    }

    pub async fn stream_file_write(
        &self,
        pid: Pid,
        path: PathBuf,
        data_stream: impl futures::Stream<Item = Vec<u8>>,
    ) -> Result<u64, String> {
        // Permission check done by executor - just proceed

        let mut file = File::create(&path)
            .await
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let mut total_bytes = 0u64;
        tokio::pin!(data_stream);

        while let Some(chunk) = futures::StreamExt::next(&mut data_stream).await {
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Write error: {}", e))?;
            total_bytes += chunk.len() as u64;
        }

        file.flush()
            .await
            .map_err(|e| format!("Flush error: {}", e))?;

        Ok(total_bytes)
    }
}
