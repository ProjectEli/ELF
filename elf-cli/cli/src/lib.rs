//! elf-cli 코어 라이브러리.
//! 순수 로직(manifest·hash·plan)은 FS 무부작용 — 통합 테스트(tests/)에서 직접 검증 가능.

pub mod embed;
pub mod hash;
pub mod init;
pub mod manifest;
pub mod plan;
pub mod selfupdate;
pub mod status;
pub mod update;
