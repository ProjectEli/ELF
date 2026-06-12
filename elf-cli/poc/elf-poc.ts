// t01 PoC: Bun 컴파일 자기완결 바이너리 + template embed 최소검증.
// 범위: --version, init 2개 명령만. update/migration/install 전부 제외.
import sample from "./assets/EliRule.sample.md" with { type: "text" };
import { writeFileSync, existsSync } from "node:fs";

const VERSION = "v2.4-dev";
const cmd = process.argv[2];

switch (cmd) {
  case "--version":
  case "-v":
    console.log(`elf-poc ${VERSION}`);
    break;
  case "init": {
    const out = "EliRule.sample.md";
    if (existsSync(out)) {
      console.error(`refuse: ${out} already exists`);
      process.exit(2);
    }
    // sample 은 빌드 시 바이너리에 embed 된 문자열 (파일시스템 의존 없음)
    writeFileSync(out, sample);
    console.log(`created ${out} (${Buffer.byteLength(sample, "utf8")} bytes)`);
    break;
  }
  default:
    console.log("usage: elf-poc <--version | init>");
    process.exit(cmd ? 1 : 0);
}
