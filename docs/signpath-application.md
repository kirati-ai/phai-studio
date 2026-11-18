# ใบสมัคร SignPath Foundation (เซ็นโค้ดฟรีสำหรับโอเพนซอร์ส)

เอกสารนี้เตรียมทุกอย่างไว้สำหรับสมัครที่ <https://signpath.org/apply>
คำตอบในแบบฟอร์มเขียนเป็นภาษาอังกฤษไว้ให้คัดลอกได้เลย

> ข้อกำหนดอ่านจาก <https://signpath.org/terms> เมื่อ 2026-11-18 (หน้านั้นระบุว่ายังเป็นฉบับร่าง)
> ก่อนส่ง ควรเปิดดูอีกครั้งว่ามีอะไรเปลี่ยนไหม SignPath ไม่ได้รับประกันว่าจะรับทุกโปรเจกต์

> **สถานะ:** ส่งใบสมัครแล้วเมื่อ 2026-11-18 รอ SignPath ตอบทางอีเมล

## สิ่งที่ต้องทำเอง ก่อนกดส่ง

1. **เปิด 2FA (การยืนยันตัวตนสองขั้นตอน) ใน GitHub** ที่ GitHub > Settings > Password and authentication
   ข้อกำหนดบังคับให้ทุกคนในทีมใช้ MFA ทั้ง GitHub และ SignPath
2. **ออก release แรก** ข้อกำหนดบอกว่าโปรเจกต์ต้อง "ออกให้ใช้แล้ว ในรูปแบบที่จะเซ็น"
   สั่ง:
   ```
   git tag v0.1.0
   git push origin v0.1.0
   ```
   GitHub Actions จะ build ตัวติดตั้งจากซอร์สแล้วแนบไว้ในหน้า Releases ให้เอง (ยังไม่เซ็น)
3. **ตรวจหน้า Releases** ว่ามี `PhaiStudio-0.1.0-setup.exe` และลองติดตั้งได้
4. **กรอกแบบฟอร์ม** ที่ <https://signpath.org/apply> ด้วยคำตอบข้างล่าง แล้วส่งเอง
   (เป็นการสมัครในนามของคุณ จึงต้องเป็นคุณที่กดส่ง)
   แบบฟอร์มเป็น HubSpot มี reCAPTCHA หัวข้อในแบบฟอร์มอาจไม่ตรงกับข้างล่างทุกข้อ
   ให้เลือกคำตอบที่ตรงกับคำถาม ถ้าถามสิ่งที่ไม่มีข้างล่าง ตอบตามจริงได้เลย
5. หลังได้รับอนุมัติ: ทำตามหัวข้อ "เมื่อได้รับอนุมัติ" ท้ายเอกสาร

## ตรวจตามข้อกำหนด

| ข้อกำหนด | สถานะ | ที่อยู่ |
|---|---|---|
| สัญญาอนุญาต OSI ไม่มี dual-licensing เชิงพาณิชย์ | ✅ MIT | `LICENSE` |
| ไม่มีโค้ดปิด (proprietary) | ✅ ทุกส่วนเปิดซอร์ส crate ที่ใช้ทั้ง 130 ตัวเป็นสัญญาอนุญาตเปิด (ส่วนใหญ่ MIT หรือ Apache-2.0 มี BSD, Zlib, Unicode บ้าง) | `Cargo.toml` |
| ไม่มีมัลแวร์ / โปรแกรมไม่พึงประสงค์ / เครื่องมือแฮก | ✅ | - |
| ดูแลอยู่ (actively maintained) | ✅ commit สม่ำเสมอ | ประวัติ git |
| ออก release แล้ว ในรูปแบบที่จะเซ็น | ✅ v0.1.0 (2026-11-18) | หน้า Releases |
| อธิบายความสามารถในหน้าดาวน์โหลด | ✅ | `README.md` |
| หน้าแรกมีหัวข้อ "Code signing policy" | ✅ | `README.md` → `docs/code-signing-policy.md` |
| ระบุบทบาทในทีม (committer, reviewer, approver) | ✅ | `docs/code-signing-policy.md` |
| ข้อความความเป็นส่วนตัว | ✅ ใช้ข้อความมาตรฐานของ SignPath และบอกว่าส่งอะไรเมื่อผู้ใช้สั่ง | `docs/code-signing-policy.md` |
| build จากซอร์สแบบตรวจสอบได้ (CI) | ✅ GitHub Actions | `.github/workflows/release.yml` |
| ทุกการเซ็นต้องอนุมัติด้วยมือ | ✅ ตั้งใน signing policy ของ SignPath ตอนได้รับอนุมัติ | - |
| ไฟล์ที่เซ็นมีชื่อผลิตภัณฑ์และเวอร์ชัน | ✅ ProductName "Phai Studio", เวอร์ชันจาก Cargo.toml | `apps/*/build.rs` |
| มีตัวถอนการติดตั้ง | ✅ ถอนผ่าน Settings > Apps | `apps/setup` |
| แจ้งก่อนเปลี่ยนแปลงระบบ | ✅ ตัวติดตั้งบอกที่ติดตั้งและทางลัดก่อนทำ | `apps/setup` |
| MFA ทั้ง GitHub และ SignPath | ✅ GitHub (authenticator app) - SignPath เปิดตอนได้บัญชี | - |

## คำตอบสำหรับแบบฟอร์ม (คัดลอกได้)

**Project name**
Phai Studio

**Repository**
https://github.com/kirati-ai/phai-studio

**Homepage / download page**
https://github.com/kirati-ai/phai-studio (README), installers on https://github.com/kirati-ai/phai-studio/releases

**License**
MIT

**Short description**
A free, open-source IDE for building Windows desktop programs in Rust, in the spirit of Visual
Studio: a form designer, a code editor with rust-analyzer, build and publish, its own debugger
(breakpoints, stepping, locals, threads, memory, disassembly), tests with code coverage, Git and
database tools. The interface is in English and Thai, with a Thai guide for beginners.

**What will be signed**
Three Windows PE files built by GitHub Actions from tagged commits of the repository:
`phai-studio.exe` (the IDE), `phai-setup.exe` (its setup and uninstall program) and
`PhaiStudio-<version>-setup.exe` (the installer: `phai-setup.exe` with the program files packed
after it). The programs are signed first, then packed, then the installer is signed.

**Build system**
GitHub Actions on `windows-latest`, workflow `.github/workflows/release.yml`, `cargo build --release --locked`.

**Team**
One maintainer, kirati-ai (author, reviewer and approver). Outside contributions come as pull
requests reviewed by the maintainer. Multi-factor authentication is on for GitHub and will be for
SignPath.

**Code signing policy URL**
https://github.com/kirati-ai/phai-studio/blob/main/docs/code-signing-policy.md

**Privacy**
No telemetry. The program sends data only when the user asks (crates.io search, Cargo downloads,
Git remotes, database servers the user adds); details in the code signing policy.

**Why signing helps**
The installer is downloaded by beginners learning Rust; unsigned, Windows SmartScreen stops it with
a warning many of them will not get past.

## เมื่อได้รับอนุมัติ

1. ใน SignPath สร้าง project ชื่อ `phai-studio` ที่ผูกกับ GitHub repository
   - **Artifact configuration** `programs`: zip ที่มี `phai-studio.exe` และ `phai-setup.exe` เซ็นแบบ Authenticode บังคับ ProductName = "Phai Studio" และ ProductVersion
   - **Artifact configuration** `installer`: zip ที่มี `PhaiStudio-*-setup.exe` เงื่อนไขเดียวกัน
   - **Signing policy** `release-signing`: ต้องมีผู้อนุมัติ (approval) ทุกครั้ง
2. ใน GitHub repository > Settings > Secrets and variables > Actions
   - Variable `SIGNPATH_ENABLED` = `true`
   - Variable `SIGNPATH_ORGANIZATION_ID` = รหัสองค์กรจาก SignPath
   - Secret `SIGNPATH_API_TOKEN` = token จาก SignPath
   - ตรวจชื่อ input ของ `signpath/github-action-submit-signing-request` กับเอกสารของ SignPath อีกครั้ง
3. ใน `docs/code-signing-policy.md` เอาเครื่องหมาย comment ออกจากบรรทัด
   "Free code signing provided by SignPath.io, certificate by SignPath Foundation"
   และแก้ประโยคแรกว่าได้รับอนุมัติแล้ว
4. ออก release ใหม่ (`git tag v0.1.1` แล้ว push) ไฟล์ที่แนบจะเป็นไฟล์ที่เซ็นแล้ว

## หมายเหตุทางเทคนิค

- ตัวติดตั้งของ Phai Studio เก็บไฟล์ต่อท้าย `phai-setup.exe` การเซ็นจะเพิ่มตารางใบรับรองไว้ท้ายไฟล์
  `phai-pack` จึงอ่านตำแหน่งตารางใบรับรองจาก PE header แล้วหาไฟล์ที่เก็บไว้ก่อนหน้านั้น
  มีเทสต์ `a_signed_setup_still_finds_its_files` ตรวจไว้
- ตัวติดตั้งของโปรแกรมที่ผู้ใช้สร้างด้วย Publish ใช้ `phai-setup.exe` ตัวเดียวกัน ตอนแพ็กไฟล์ `phai-pack` ถอดลายเซ็นเดิมออกก่อน
  ตัวติดตั้งของผู้ใช้จึงเป็นแบบไม่ได้เซ็น (ไม่ใช่ลายเซ็นเสีย) ใบรับรองของ SignPath Foundation ใช้ได้เฉพาะโปรแกรมของโปรเจกต์นี้
