"""Build and exercise the bundled recipes on a local Docker engine (Python 3.11+)."""

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import uuid


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--context", help="Local Docker context, e.g. colima")
    args = parser.parse_args()
    if not hasattr(os, "getuid") or os.getuid() == 0:
        parser.error("run as a non-root Unix user to verify bind-mount ownership")
    if shutil.which("docker") is None or shutil.which("git") is None:
        parser.error("Docker CLI and Git must be on PATH")

    root = Path(__file__).resolve().parents[1]
    recipes = root / "skills/oracle/assets/containerization"
    docker = ["docker"] + (["--context", args.context] if args.context else [])
    env = os.environ.copy()

    def run(*arguments, timeout=60, check=True, capture=True):
        result = subprocess.run(
            [*docker, *arguments], cwd=root, env=env, text=True,
            capture_output=capture, check=False, timeout=timeout,
        )
        if check and result.returncode:
            if capture:
                print(result.stdout, end="", flush=True)
                print(result.stderr, end="", flush=True)
            result.check_returncode()
        return result

    context = args.context or env.get("DOCKER_CONTEXT")
    endpoint = env.get("DOCKER_HOST") if not context else None
    if not endpoint:
        endpoint = run("context", "inspect", *([context] if context else []),
                       "--format", "{{.Endpoints.docker.Host}}").stdout.strip()
    if not endpoint.startswith("unix://"):
        parser.error("use a local Unix-socket engine; remote endpoints are not tested")
    run("info")
    run("compose", "version")
    run("buildx", "version")

    owner = uuid.uuid4().hex
    label = "io.oracle.container-test"
    image = f"oracle-container-test:{owner}"
    containers = []
    temp_parent = root / ".oracle/container-tests"
    temp_parent.mkdir(parents=True, exist_ok=True)
    version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    try:
        run("build", "--pull", "--label", f"{label}={owner}",
            "-f", str(recipes / "Dockerfile"), "-t", image, ".",
            timeout=900, capture=False)
        metadata = json.loads(run("image", "inspect", image).stdout)[0]
        assert metadata["Config"]["User"] == "10001:10001"
        assert metadata["Os"] == "linux"
        print(json.dumps({"image_id": metadata["Id"], "os": metadata["Os"],
                          "architecture": metadata["Architecture"]}), flush=True)

        def direct(*arguments, entrypoint=None, check=True):
            name = f"oracle-test-{owner}-{len(containers)}"
            containers.append(name)
            options = ["run", "--rm", "--name", name, "--label", f"{label}={owner}",
                       "--read-only", "--network", "none", "--cap-drop", "ALL",
                       "--security-opt", "no-new-privileges:true"]
            if entrypoint:
                options += ["--entrypoint", entrypoint]
            return run(*options, image, *arguments, check=check)

        assert direct("--version").stdout.strip() == f"oracle {version}"
        assert direct("-u", entrypoint="id").stdout.strip() == "10001"
        direct("-c", "command -v git && ! command -v cargo && ! command -v rustc",
               entrypoint="sh")
        rejected = direct("/workspace/readonly-probe", entrypoint="touch", check=False)
        assert rejected.returncode != 0 and "Read-only file system" in rejected.stderr

        with tempfile.TemporaryDirectory(prefix="fixture-", dir=temp_parent) as temp:
            project = Path(temp)
            env.update(ORACLE_IMAGE=image, ORACLE_PROJECT_PATH=str(project),
                       ORACLE_UID=str(os.getuid()), ORACLE_GID=str(os.getgid()))
            compose = ["compose", "-p", f"oracle-test-{owner}",
                       "-f", str(recipes / "compose.yaml")]
            run(*compose, "config", "--quiet")
            subprocess.run(["git", "init", "-q", str(project)], check=True)
            (project / "keep.txt").write_text("retained fixture data\n")

            def invoke(*arguments):
                name = f"oracle-test-{owner}-{len(containers)}"
                containers.append(name)
                return run(*compose, "run", "--rm", "--name", name,
                           "--label", f"{label}={owner}", "oracle", *arguments)

            invoke("init", "--name", "container-fixture", "--kind", "cli")
            config = (project / "oracle.toml").read_bytes()
            assert (project / "oracle.toml").stat().st_uid == os.getuid()
            subprocess.run(["git", "add", "."], cwd=project, check=True)
            report = json.loads(invoke("assess", "--format", "json").stdout)
            secret_check = next(f for f in report["findings"] if f["id"] == "SEC-001")
            assert secret_check["status"] == "pass", secret_check
            invoke("init", "--name", "must-not-replace-existing")
            assert (project / "oracle.toml").read_bytes() == config
            assert (project / "keep.txt").read_text() == "retained fixture data\n"
        print("PASS: final image, non-root user, Git inspection, read-only root, "
              "Compose, host ownership, and persistence across fresh containers", flush=True)
    finally:
        # Only remove resources bearing this run's unique ownership label.
        for name in containers:
            result = run("container", "inspect", name, check=False)
            if result.returncode == 0:
                config = json.loads(result.stdout)[0]["Config"]
                if (config.get("Labels") or {}).get(label) == owner:
                    run("container", "rm", "-f", name)
        result = run("image", "inspect", image, check=False)
        if result.returncode == 0:
            config = json.loads(result.stdout)[0]["Config"]
            if (config.get("Labels") or {}).get(label) == owner:
                run("image", "rm", image)


if __name__ == "__main__":
    main()
