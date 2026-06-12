//! 콘텐츠 해시 — CRLF→LF 정규화 후 sha256.
//! autocrlf 환경에서 dest가 CRLF로 checkout되어도 정본(LF)과 동일 해시를 보장
//! (Git_Hook_Governance §5의 "git hash-object" 교훈을 git 비의존으로 이식).

use sha2::{Digest, Sha256};

/// CRLF 쌍만 LF로 정규화(단독 \r은 보존) 후 sha256 hex(소문자) 반환.
pub fn sha256_lf(bytes: &[u8]) -> String {
    let mut norm = Vec::with_capacity(bytes.len());
    let mut iter = bytes.iter().peekable();
    while let Some(&b) = iter.next() {
        if b == b'\r' && iter.peek() == Some(&&b'\n') {
            continue; // \r\n → \n (\r 제거, \n은 다음 루프에서 push)
        }
        norm.push(b);
    }
    let mut hasher = Sha256::new();
    hasher.update(&norm);
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lf_and_crlf_content_hash_identically() {
        assert_eq!(sha256_lf(b"a\nb\n"), sha256_lf(b"a\r\nb\r\n"));
    }

    #[test]
    fn different_content_hashes_differently() {
        assert_ne!(sha256_lf(b"a\n"), sha256_lf(b"b\n"));
    }

    #[test]
    fn lone_cr_is_preserved_not_stripped() {
        // 단독 \r(구식 Mac/바이너리 잔재)은 의미 변경 방지를 위해 정규화하지 않음
        assert_ne!(sha256_lf(b"a\rb"), sha256_lf(b"ab"));
    }

    #[test]
    fn hex_is_lowercase_64_chars() {
        let h = sha256_lf(b"x");
        assert_eq!(h.len(), 64);
        assert_eq!(h, h.to_lowercase());
    }
}
