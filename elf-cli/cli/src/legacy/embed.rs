//! [동결] ELF v2.27.4 로그 틀(trialTemplate, sessionTemplate, Session_Registry.tsv) — 이전 형식 저장소 전용.
//! 배포 대상 아님(manifest 밖, `templates/`와 분리).

use include_dir::{Dir, include_dir};

pub static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/legacy/tpl");
