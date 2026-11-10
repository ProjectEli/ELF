//! elf-cli 코어 라이브러리.
//! 순수 로직(manifest, hash, plan)은 FS 무부작용 — 통합 테스트(tests/)에서 직접 검증 가능.

pub mod autoread;
pub mod check;
pub mod consent;
pub mod deprecate;
pub mod dispatch;
pub mod doctor;
pub mod earlier;
mod earlier_files;
pub mod embed;
pub mod gallery;
pub mod hash;
pub mod header;
pub mod init;
pub mod legacy;
pub mod logcmd;
pub mod logs;
pub mod manifest;
pub mod migrate;
pub mod pin;
pub mod plan;
pub mod sections;
pub mod selfupdate;
pub mod session;
pub mod status;
pub mod store;
pub mod trial;
pub mod tsa;
pub mod update;
pub mod validate;
pub mod version;
