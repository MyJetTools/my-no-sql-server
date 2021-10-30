use my_no_sql_tcp_shared::ReadingTcpContractFail;

use super::SendPackageError;

#[derive(Debug)]
pub enum ReadSocketError {
    ReadingTcpContractFail(ReadingTcpContractFail),
    SendPackageError(SendPackageError),
}

impl From<SendPackageError> for ReadSocketError {
    fn from(src: SendPackageError) -> Self {
        Self::SendPackageError(src)
    }
}

impl From<ReadingTcpContractFail> for ReadSocketError {
    fn from(src: ReadingTcpContractFail) -> Self {
        Self::ReadingTcpContractFail(src)
    }
}
