# Contributing to Phai Studio

Thank you for helping. Bug reports, ideas and pull requests are all welcome.
ขอบคุณที่ช่วยครับ แจ้งบั๊ก เสนอความคิด หรือส่ง pull request ได้ทั้งหมด (ภาษาไทยได้)

## Reporting a bug / แจ้งบั๊ก

Open an issue with the **Bug report** form: what you did, what you expected, what happened, and
Phai Studio's version (Help > About > Copy). A small project or a `.phf` file that shows the
problem helps most.

## Building / build เอง

You need Rust with the MSVC toolchain on Windows.

```
cargo run -p phai-studio                    # Studio
cargo test --workspace                      # every test
cargo run -p phai-studio -- apps/sample/src/main_window.phf
```

Phai Studio runs on Windows only for now; the programs it makes also run in any browser as a PWA.

## Pull requests

- One change per pull request, with a test where the change can be tested
  (windows are checked by their picture: `PHAI_SHOT=out.png`).
- `cargo test --workspace` passes, with no new warnings.
- Every sentence a user sees has its Thai beside it (`apps/studio/src/text.rs`); a test fails otherwise.
- Comments and commit messages say what the code does for the user, in plain English.

## The terms of a contribution / เงื่อนไขของโค้ดที่ส่งมา

Phai Studio is in two parts under two licenses (see [LICENSE](LICENSE)). By opening a pull request
you agree that:

1. **A contribution to the libraries** (`crates/phai-ui`, `crates/phai-data`, `crates/phai-web`,
   `crates/phai-pack`, `apps/setup`, the examples) is given under the **MIT License**, as the
   libraries are.
2. **A contribution to anything else** (Phai Studio itself, the guide, the website) is given to
   Monrak Net (Kirati Kittikhun) under the **MIT License**, so that Monrak Net may release it as
   part of Phai Studio under Phai Studio's license now and under any license later.
   Everyone else gets it under Phai Studio's license, as the rest of Phai Studio.
3. The contribution is **your own work**, or you have the right to give it on these terms, and
   it carries no code under a license that does not allow this (GPL code, for example).
4. You keep the copyright in what you wrote.

The pull request form has a box to tick for this. Without it a pull request cannot be merged.

สรุปภาษาไทย: โค้ดของไลบรารีส่งมาภายใต้ MIT เหมือนไลบรารี ส่วนโค้ดของตัว Studio ส่งมาให้ Monrak Net
ภายใต้ MIT เพื่อให้รวมเข้า Phai Studio ได้ทั้งตอนนี้และถ้าวันหน้าเปลี่ยน license ผู้ส่งยังเป็นเจ้าของลิขสิทธิ์
ในสิ่งที่เขียนเอง และยืนยันว่าเป็นงานของตัวเอง ไม่มีโค้ดจาก license ที่ไม่ยอมให้ทำแบบนี้

## Forks

You may fork Phai Studio to try it, change it for yourself, or send changes back. What a fork may
not do - offer a product that competes with Phai Studio, or use the name "Phai Studio" and the
bamboo logo - is in [LICENSE](LICENSE) and [docs/license-faq.md](docs/license-faq.md).
