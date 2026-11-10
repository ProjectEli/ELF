//! 생성 파일 — `tools/earlier_files.py`가 git tag(v2.0~v2.3 generator 틀, v2.4.0 CLI manifest)에서 만듦. 손으로 고치지 않음.
//! `.elf/` 없는 ELF 2.0~2.4.0 프로젝트의 ELF 파일이 당시 원본 그대로인지 판정하는 지문(v3.0.2, S046).

/// 지문 — LF 정규화 바이트의 sha256. `Around` = 언어 한 줄만 바꿔 쓴 파일(EliRule)의 앞부분과 뒷부분(BOM 제외).
pub(crate) enum Print {
    Exact(&'static str),
    Around { head_len: usize, head: &'static str, tail_len: usize, tail: &'static str },
}

/// 알려진 원본 하나: 프로젝트 안 위치, 관리 방식(`managed`, `hybrid`), 처음 나온 버전, 지문.
pub(crate) struct Known {
    pub dest: &'static str,
    pub tier: &'static str,
    pub since: &'static str,
    pub print: Print,
}

pub(crate) const KNOWN: &[Known] = &[
    Known { dest: ".gitignore", tier: "hybrid", since: "v2.0", print: Print::Exact("4f451385aa231fa273ed7a00ada5dccf2eb9e19d8d5251eebea054c47754aff4") },
    Known { dest: "0_Meta/LogConvention.md", tier: "managed", since: "v2.0", print: Print::Exact("22c1d08872a483a4625ed493597c81231f746353fcc2e643e30a9e2b9bedca4f") },
    Known { dest: "0_Meta/AI_PARA_Framework.md", tier: "managed", since: "v2.0", print: Print::Exact("ca049107ef3132ed31a4e989402c0ac1d7ba835be9c7875ae72c0d67bf1c9b96") },
    Known { dest: "templates/sessionTemplate.md", tier: "managed", since: "v2.0", print: Print::Exact("bd59ba1e1afe1b568d7067af67cfb39b10f66ad5cafc19c75d9ba7ff33f5a7e4") },
    Known { dest: "templates/trialTemplate.md", tier: "managed", since: "v2.0", print: Print::Exact("ecfd1f51b85bfb86d3e23be9aee65b078ea6a4c5eeb8758bcd10bbd48da1cf1a") },
    Known { dest: "0_Meta/scripts/elf_gallery.ps1", tier: "managed", since: "v2.0", print: Print::Exact("4f4d1734da64af91bd0004f78ef4a62da59e58b9e497b17fb04372a1edf9652a") },
    Known { dest: "0_Meta/scripts/elf_gallery.sh", tier: "managed", since: "v2.0", print: Print::Exact("55dc14d60998102ade879a4728edde08fc6fc98d87f0dbf5a533660f4194cb3b") },
    Known { dest: "0_Meta/EliRule.md", tier: "managed", since: "v2.0", print: Print::Around { head_len: 5237, head: "ff5a6d13dd8af75547ec595ec9f90ba5073db6cba662bb9bc411e2462bc57f83", tail_len: 2474, tail: "b05fc9977dc89aafedd5b43743ecec72cebc03dfa7f4f2347fce88c141716053" } },
    Known { dest: "0_Meta/LogConvention.md", tier: "managed", since: "v2.1", print: Print::Exact("779ebfe74445ab1cc363f6dd8f63697524bec7a85660c026f2871886a184c5cb") },
    Known { dest: "0_Meta/AI_PARA_Framework.md", tier: "managed", since: "v2.1", print: Print::Exact("9261d74242aa43018f73d848526503c3fe53fb1af5a24ad45bac402112456511") },
    Known { dest: "0_Meta/LogConvention.md", tier: "managed", since: "v2.2", print: Print::Exact("80900668942dc4a45fa63a3ba807388f7c8b00cff3466dc74d59b568659cd48c") },
    Known { dest: "0_Meta/AI_PARA_Framework.md", tier: "managed", since: "v2.2", print: Print::Exact("a1829448d41de7c140f21feecfca1e176b4f999cef615db0318d8f5d8badfec8") },
    Known { dest: "0_Meta/scripts/elf_gallery.ps1", tier: "managed", since: "v2.2", print: Print::Exact("25afd2fad4bdf21ef7041d7d32713369eafc3c2819274eb32e6741219d32899a") },
    Known { dest: "0_Meta/scripts/elf_gallery.sh", tier: "managed", since: "v2.2", print: Print::Exact("7e2eb67f13db26c78794ac6f803fe73b43a47a91cdfd7459c5d99e373c06597e") },
    Known { dest: "0_Meta/EliRule.md", tier: "managed", since: "v2.2", print: Print::Around { head_len: 5237, head: "678ce5634489594a68f68f1252aba02d7b62b9d25f8c784672310d2890e90655", tail_len: 2474, tail: "b05fc9977dc89aafedd5b43743ecec72cebc03dfa7f4f2347fce88c141716053" } },
    Known { dest: "0_Meta/LogConvention.md", tier: "managed", since: "v2.3", print: Print::Exact("511bad1b1502c20d41611d57e65b9cb17e3c8c00f65dfbc1c4bd328f956e7895") },
    Known { dest: "0_Meta/AI_PARA_Framework.md", tier: "managed", since: "v2.3", print: Print::Exact("dd4ced8191daa07f2ed7368994c52deda96e3db2be9843dd53a843d74951c072") },
    Known { dest: "templates/sessionTemplate.md", tier: "managed", since: "v2.3", print: Print::Exact("ed7c79cb4a824f4499b5a8c37093c80fa3e772cf4a4a398ef98ac7026cb562e6") },
    Known { dest: "templates/trialTemplate.md", tier: "managed", since: "v2.3", print: Print::Exact("6fea493b17c8e505206fadd99cdd3c0b3dec076b8f59818daa0f2919aa126283") },
    Known { dest: "templates/Session_Registry.tsv", tier: "managed", since: "v2.3", print: Print::Exact("b4eed8d302ff9e87860c183a3fd0c98412ea10e07314e2d66d7bbd0b32deab9e") },
    Known { dest: "0_Meta/EliRule.md", tier: "managed", since: "v2.3", print: Print::Around { head_len: 5539, head: "27005962fac74ed67ee0f3b372cb2264a39a6cb67f5fb3ba794785728d0eea70", tail_len: 2474, tail: "b05fc9977dc89aafedd5b43743ecec72cebc03dfa7f4f2347fce88c141716053" } },
    Known { dest: "0_Meta/EliRule.md", tier: "managed", since: "v2.4.0", print: Print::Exact("6d5ef36f682519fb03e94827429966d831c0fc7b340c6f3d0c32c8eb09d56812") },
    Known { dest: "0_Meta/LogConvention.md", tier: "managed", since: "v2.4.0", print: Print::Exact("420c8cb84d1ae12ed0aeae33afdf4b455b619b0b283319c5a816d3634c3d1366") },
    Known { dest: "0_Meta/AI_PARA_Framework.md", tier: "managed", since: "v2.4.0", print: Print::Exact("7780a387dee3a920eaad23a8a9f1dac92a37839ff816e24436d27d3e6de25710") },
    Known { dest: "0_Meta/highIFjournals.md", tier: "managed", since: "v2.4.0", print: Print::Exact("923fe19ba7097b60cb6f9e4613737e3fab568edd564c87ed4d8acba658631d15") },
    Known { dest: ".gitignore", tier: "hybrid", since: "v2.4.0", print: Print::Exact("f463ee15c311e2138b44831375c728c28e182572aa9bde4f0223d08831fe1b33") },
];
