"""Prepare a fresh, offline fixture; starting a paid agent session is separate."""

import argparse
import json
from pathlib import Path
import shutil
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("scenario", choices=("simple", "enterprise"))
    parser.add_argument("destination", type=Path)
    parser.add_argument("--client", choices=("codex", "claude", "generic"), default="codex")
    parser.add_argument("--oracle", default="oracle", help="Built Oracle binary or command")
    args = parser.parse_args()

    binary = shutil.which(args.oracle)
    if binary is None:
        parser.error("Oracle binary not found; build it and pass --oracle PATH")
    binary = str(Path(binary).resolve())
    destination = args.destination.resolve()
    if destination.exists():
        parser.error("destination must not exist; use a new directory for every trial")
    if shutil.which("git") is None:
        parser.error("Git is required to create the isolated fixture")

    destination.mkdir(parents=True)
    subprocess.run(["git", "init", "-q", "-b", "trial", str(destination)], check=True)
    (destination / "AGENTS.md").write_text(
        "# Workspace scope\n\n"
        "Read project content and write files only within this workspace. "
        "Do not inspect sibling projects, temporary files from other work, "
        "or prior agent transcripts. Installed tools and runtime dependencies "
        "may be used for verification. Keep test artifacts here.\n",
        encoding="utf-8",
    )
    (destination / "index.html").write_text(
        '<!doctype html>\n<html lang="en"><head><meta charset="utf-8">'
        '<meta name="viewport" content="width=device-width, initial-scale=1">'
        '<title>Prototype</title><link rel="stylesheet" href="styles.css">'
        '<script src="app.js" defer></script></head>'
        '<body><main><h1>Prototype</h1></main></body></html>\n',
        encoding="utf-8",
    )
    (destination / "styles.css").write_text("/* Application styles. */\n", encoding="utf-8")
    (destination / "app.js").write_text("// Application behavior.\n", encoding="utf-8")
    subprocess.run(
        [binary, "--root", str(destination), "agent", "install", args.client],
        check=True,
        stdout=subprocess.DEVNULL,
    )
    print(json.dumps({
        "workspace": str(destination),
        "prompt": str(Path(__file__).with_name(args.scenario + ".txt").resolve()),
        "client": args.client,
    }, indent=2))


if __name__ == "__main__":
    main()
