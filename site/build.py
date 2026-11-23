"""Builds phai-studio.monrak.net into target/site: this folder, the Thai guide from docs/help as
/help/, and release.json describing the newest GitHub release (version, file, size, SHA-256).

    python site/build.py            # then upload target/site to the web server
    python site/build.py --serve    # and preview it on http://localhost:8790/

Uploading keeps the server's storage/ folder (the download and visitor counts): copy over the
top, never delete the folder first.
"""

import json
import shutil
import subprocess
import sys
from datetime import datetime, timedelta
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent
OUT = ROOT / "target" / "site"
SKIP = {"build.py", "router.php", "deploy.sh", "storage", "__pycache__"}


def release():
    """The newest release, read from GitHub with gh."""
    try:
        raw = subprocess.run(
            ["gh", "release", "view", "-R", "kirati-ai/phai-studio", "--json", "tagName,publishedAt,assets"],
            capture_output=True, text=True, check=True,
        ).stdout
    except (OSError, subprocess.CalledProcessError) as e:
        sys.exit(f"could not read the release from GitHub: {e}")
    r = json.loads(raw)
    asset = next(a for a in r["assets"] if a["name"].endswith("-setup.exe"))
    return {
        "version": r["tagName"].lstrip("v"),
        "file": asset["name"],
        "size": asset["size"],
        "sha256": (asset.get("digest") or "").removeprefix("sha256:"),
        # The day in Thailand (UTC+7), where the site's readers are: a release at 23:22 UTC is the next day there.
        "date": (datetime.fromisoformat(r["publishedAt"].replace("Z", "+00:00")) + timedelta(hours=7)).strftime("%Y-%m-%d"),
    }


def main():
    if OUT.exists():
        shutil.rmtree(OUT)
    OUT.mkdir(parents=True)
    for p in HERE.iterdir():
        if p.name in SKIP:
            continue
        if p.is_dir():
            shutil.copytree(p, OUT / p.name)
        else:
            shutil.copy2(p, OUT / p.name)
    shutil.copytree(ROOT / "docs" / "help", OUT / "help")
    (OUT / "release.json").write_text(json.dumps(release(), indent=2), encoding="utf-8")
    print(f"built {OUT}")
    if "--serve" in sys.argv:
        php = shutil.which("php") or r"D:\DesktopServer\php82\php.exe"
        subprocess.run([php, "-S", "localhost:8790", "-t", str(OUT), str(HERE / "router.php")])


if __name__ == "__main__":
    main()
