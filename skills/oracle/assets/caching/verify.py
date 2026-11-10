"""Exercise this standalone Rust/Redis example (Python 3.11+, Unix)."""

import argparse
import json
import os
from pathlib import Path
import platform
import select
import shutil
import socket
import subprocess
import tempfile
import time
import uuid


HERE = Path(__file__).resolve().parent


def run(command, *, env=None, cwd=HERE, timeout=60, check=True):
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True,
                            text=True, timeout=timeout)
    if result.returncode and check:
        print(result.stdout, end="", flush=True)
        print(result.stderr, end="", flush=True)
        result.check_returncode()
    return result


def verify(command, env, mode, expected):
    result = run([*command, mode], env=env, timeout=15)
    assert expected in result.stdout, result.stdout
    print(result.stdout, end="", flush=True)


def outage(command, env, stop):
    process = subprocess.Popen([*command, "outage"], cwd=HERE, env=env,
                               stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, text=True)
    try:
        ready, _, _ = select.select([process.stdout], [], [], 30)
        if not ready or process.stdout.readline().strip() != "READY_FOR_OUTAGE":
            raise RuntimeError("demo did not reach the outage checkpoint")
        stop()
        output, errors = process.communicate("continue\n", timeout=15)
        assert process.returncode == 0, errors
        assert "PASS outage:" in output, output
        print(output, end="", flush=True)
    finally:
        if process.poll() is None:
            process.kill()
            process.wait(timeout=5)


def native(redis_server):
    cargo = shutil.which("cargo")
    server = shutil.which(redis_server)
    if not cargo or not server:
        raise RuntimeError("cargo and the requested redis-server must be executable on PATH")
    print(f"Native: {platform.system()} {platform.machine()}; "
          f"{run([server, '--version']).stdout.strip()}", flush=True)
    with tempfile.TemporaryDirectory(prefix="oracle-cache-") as directory:
        project = Path(directory)
        for source, destination in [("Cargo.toml.example", "Cargo.toml"),
                                    ("Cargo.lock.example", "Cargo.lock"),
                                    ("main.rs", "main.rs")]:
            shutil.copyfile(HERE / source, project / destination)
        env = dict(os.environ, CARGO_TARGET_DIR=str(project / "target"))
        for arguments in [("fmt", "--all", "--check"),
                          ("clippy", "--locked", "--all-targets", "--", "-D", "warnings"),
                          ("test", "--locked"), ("build", "--release", "--locked")]:
            result = run([cargo, *arguments], env=env, cwd=project, timeout=300)
            print(result.stdout + result.stderr, end="", flush=True)
        command = [str(project / "target/release/oracle-cache-example")]
        socket_path = project / "redis.sock"
        if len(str(socket_path).encode()) > 100:
            raise RuntimeError("temporary directory path is too long for a Unix socket; set TMPDIR")
        env["REDIS_URL"] = f"redis+unix://{socket_path}"
        process = None

        def stop():
            if process is not None and process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait(timeout=5)

        with (project / "redis.log").open("w+") as log:
            def start():
                nonlocal process
                process = subprocess.Popen(
                    [server, "--port", "0", "--unixsocket", str(socket_path),
                     "--unixsocketperm", "700", "--save", "", "--appendonly", "no",
                     "--maxmemory", "32mb", "--maxmemory-policy", "allkeys-lru",
                     "--dir", str(project), "--daemonize", "no"],
                    stdout=log, stderr=subprocess.STDOUT)
                deadline = time.monotonic() + 10
                while process.poll() is None and time.monotonic() < deadline:
                    try:
                        with socket.socket(socket.AF_UNIX) as connection:
                            connection.settimeout(0.25)
                            connection.connect(str(socket_path))
                            connection.sendall(b"*1\r\n$4\r\nPING\r\n")
                            if connection.recv(64) == b"+PONG\r\n":
                                return
                    except OSError:
                        pass
                    time.sleep(0.05)
                log.seek(0)
                raise RuntimeError("isolated Redis did not start: " + log.read())

            try:
                start()
                verify(command, env, "verify", "PASS cache:")
                outage(command, env, stop)
                verify(command, env, "fallback", "PASS unavailable:")
                start()
                verify(command, env, "verify", "PASS cache:")
            finally:
                stop()


def containers(context):
    docker = shutil.which("docker")
    if not docker:
        raise RuntimeError("Docker CLI must be executable on PATH")
    command = [docker] + (["--context", context] if context else [])
    env = os.environ.copy()
    selected = context or env.get("DOCKER_CONTEXT")
    endpoint = env.get("DOCKER_HOST") if not selected else None
    if not endpoint:
        endpoint = run([*command, "context", "inspect", *([selected] if selected else []),
                        "--format", "{{.Endpoints.docker.Host}}"], env=env).stdout.strip()
    if not endpoint.startswith("unix://"):
        raise RuntimeError("use a local Unix-socket engine; remote endpoints are not tested")
    run([*command, "info"], env=env)
    run([*command, "compose", "version"], env=env)
    run([*command, "buildx", "version"], env=env)
    owner = uuid.uuid4().hex
    label = "io.oracle.cache-example"
    image = f"oracle-cache-example:{owner}"
    env["ORACLE_CACHE_IMAGE"] = image
    compose = [*command, "compose", "-p", f"oracle-cache-{owner}",
               "-f", str(HERE / "compose.yaml")]
    demo = [*compose, "run", "--rm", "--no-deps", "-T", "demo"]
    try:
        run([*command, "build", "--pull", "--label", f"{label}={owner}",
             "-t", image, "."], env=env, timeout=900)
        metadata = json.loads(run([*command, "image", "inspect", image], env=env).stdout)[0]
        assert metadata["Config"]["User"] == "10001:10001"
        assert metadata["Os"] == "linux"
        print(json.dumps({"image_id": metadata["Id"], "os": metadata["Os"],
                          "architecture": metadata["Architecture"]}), flush=True)
        run([*compose, "config", "--quiet"], env=env)
        run([*compose, "up", "-d", "--wait", "redis"], env=env, timeout=120)
        identifier = run([*compose, "ps", "-q", "redis"], env=env).stdout.strip()
        metadata = json.loads(run([*command, "inspect", identifier], env=env).stdout)[0]
        assert not metadata["HostConfig"]["PortBindings"]
        assert not any(m["Type"] == "volume" for m in metadata["Mounts"])
        print("Redis fixture image: " + metadata["Image"], flush=True)
        verify(demo, env, "verify", "PASS cache:")
        outage(demo, env, lambda: run([*compose, "stop", "redis"], env=env))
        verify(demo, env, "fallback", "PASS unavailable:")
        run([*compose, "up", "-d", "--wait", "redis"], env=env, timeout=30)
        verify(demo, env, "verify", "PASS cache:")
    finally:
        # Compose selects only this run's unique project. No global prune or volumes.
        run([*compose, "down", "--remove-orphans", "--timeout", "5"], env=env)
        result = run([*command, "image", "inspect", image], env=env, check=False)
        if result.returncode == 0:
            labels = json.loads(result.stdout)[0]["Config"].get("Labels") or {}
            if labels.get(label) == owner:
                run([*command, "image", "rm", image], env=env)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native", action="store_true", help="use local Rust and an isolated redis-server")
    parser.add_argument("--redis-server", default="redis-server", help="native Redis executable")
    parser.add_argument("--context", help="local Docker context, e.g. colima")
    args = parser.parse_args()
    if os.name != "posix":
        parser.error("this verifier requires Unix; use the Compose commands on other hosts")
    if args.native and args.context:
        parser.error("--context applies only to the container mode")
    if args.native:
        native(args.redis_server)
    else:
        containers(args.context)
    print("PASS: hits, misses, expiry, invalidation, startup failure, live outage, "
          "and a new client after Redis restart; synthetic sequential fixture", flush=True)


if __name__ == "__main__":
    main()
