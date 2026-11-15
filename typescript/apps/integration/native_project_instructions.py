"""Released-program, model-free project instruction delivery acceptance.

The caller supplies a disposable npm program closure containing stock codex-acp
and Codex. This test never reads operator credentials or calls a live model.
"""

from __future__ import annotations

import argparse
import json
import os
import queue
import shutil
import signal
import subprocess
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path


def model_catalog() -> dict:
    return {
        "models": [
            {
                "slug": "fixture-model",
                "display_name": "Fixture model",
                "description": "Fixed local responses only",
                "default_reasoning_level": "low",
                "supported_reasoning_levels": [
                    {"effort": "low", "description": "Fixture"}
                ],
                "shell_type": "shell_command",
                "visibility": "list",
                "supported_in_api": True,
                "priority": 0,
                "base_instructions": "Model-free delivery fixture.",
                "model_messages": None,
                "supports_reasoning_summaries": False,
                "supports_reasoning_summary_parameter": False,
                "support_verbosity": False,
                "default_verbosity": None,
                "apply_patch_tool_type": "freeform",
                "web_search_tool_type": "text_and_image",
                "truncation_policy": {"mode": "tokens", "limit": 10000},
                "supports_parallel_tool_calls": False,
                "context_window": 128000,
                "input_modalities": ["text"],
                "experimental_supported_tools": [],
            }
        ]
    }


def run(programs: Path, node: str) -> dict:
    adapter = programs / "node_modules/@agentclientprotocol/codex-acp/dist/index.js"
    codex = programs / "node_modules/.bin/codex"
    versions = {}
    for package, expected in (
        ("@agentclientprotocol/codex-acp", "2.6.0"),
        ("@openai/codex", "0.165.0"),
    ):
        version = json.loads(
            (programs / "node_modules" / package / "package.json").read_text()
        )["version"]
        if version != expected:
            raise RuntimeError(
                "Designated native acceptance requires its pinned released programs"
            )
        versions[package] = version
    if not adapter.is_file() or not codex.is_file() or not Path(node).is_file():
        raise RuntimeError("Native acceptance programs are unavailable")
    requests = []
    failures = []
    lock = threading.Lock()

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def do_POST(self):
            if self.path != "/v1/responses":
                self.send_error(404)
                return
            value = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
            with lock:
                requests.append(value)
                number = len(requests)
                if any(
                    tool.get("type") == "function" for tool in value.get("tools", [])
                ):
                    # Declarations are allowed, but no tool call is ever returned.
                    pass
            item = {
                "id": f"fixture-message-{number}",
                "type": "message",
                "status": "completed",
                "role": "assistant",
                "content": [
                    {
                        "type": "output_text",
                        "text": "Fixed fixture completion.",
                        "annotations": [],
                    }
                ],
            }
            events = [
                {"type": "response.output_item.done", "output_index": 0, "item": item},
                {
                    "type": "response.completed",
                    "response": {
                        "id": f"fixture-response-{number}",
                        "status": "completed",
                        "output": [item],
                        "usage": {
                            "input_tokens": 1,
                            "output_tokens": 1,
                            "total_tokens": 2,
                            "input_tokens_details": {"cached_tokens": 0},
                        },
                    },
                },
            ]
            body = "".join(
                f"event: {event['type']}\ndata: {json.dumps(event)}\n\n"
                for event in events
            ).encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/event-stream")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

    server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, daemon=True).start()
    process = None
    reader = None
    try:
        with tempfile.TemporaryDirectory(
            prefix="chapchoo-project-instructions-"
        ) as temporary:
            root = Path(temporary)
            workspace = root / "workspace"
            workspace.mkdir()
            for directory in ("home", "codex"):
                (root / directory).mkdir()
            (root / "models.json").write_text(json.dumps(model_catalog()))
            config = {
                "model_provider": "fixture",
                "model": "fixture-model",
                "model_providers": {
                    "fixture": {
                        "name": "Local fixture",
                        "base_url": f"http://127.0.0.1:{server.server_port}/v1",
                        "wire_api": "responses",
                        "requires_openai_auth": False,
                    }
                },
                "model_catalog_json": str(root / "models.json"),
            }
            environment = {
                "PATH": os.pathsep.join((str(Path(node).parent), "/usr/bin", "/bin")),
                "HOME": str(root / "home"),
                "CODEX_HOME": str(root / "codex"),
                "CODEX_PATH": str(codex),
                "CODEX_CONFIG": json.dumps(config),
                "MODEL_PROVIDER": "fixture",
                "LANG": "C.UTF-8",
            }
            responses = queue.Queue()
            condition = threading.Condition()
            updates = []
            sequence = 0

            def read_messages(child):
                try:
                    for line in child.stdout:
                        message = json.loads(line)
                        if "id" in message and "method" not in message:
                            responses.put(message)
                        elif "id" in message:
                            # A tool/action request would invalidate this fixed-response fixture.
                            failures.append("Unexpected native action request")
                            child.stdin.write(
                                json.dumps(
                                    {
                                        "jsonrpc": "2.0",
                                        "id": message["id"],
                                        "error": {
                                            "code": -32601,
                                            "message": "No fixture action is permitted",
                                        },
                                    }
                                )
                                + "\n"
                            )
                            child.stdin.flush()
                        elif message.get("method") == "session/update":
                            with condition:
                                updates.append(message)
                                condition.notify_all()
                except (ValueError, BrokenPipeError):
                    failures.append("Native fixture framing failed")

            def call(method, params):
                nonlocal sequence
                sequence += 1
                process.stdin.write(
                    json.dumps(
                        {
                            "jsonrpc": "2.0",
                            "id": sequence,
                            "method": method,
                            "params": params,
                        }
                    )
                    + "\n"
                )
                process.stdin.flush()
                response = responses.get(timeout=45)
                if response.get("id") != sequence or "result" not in response:
                    raise RuntimeError(
                        f"Native fixture failed during {method}; response withheld"
                    )
                return response["result"]

            def stop():
                nonlocal process, reader
                if process is not None:
                    if process.poll() is None:
                        os.killpg(process.pid, signal.SIGTERM)
                        try:
                            process.wait(timeout=8)
                        except subprocess.TimeoutExpired:
                            os.killpg(process.pid, signal.SIGKILL)
                            process.wait(timeout=5)
                    reader.join(timeout=3)
                    process = None

            def start():
                nonlocal process, reader
                process = subprocess.Popen(
                    [node, str(adapter)],
                    cwd=workspace,
                    env=environment,
                    stdin=subprocess.PIPE,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.DEVNULL,
                    text=True,
                    start_new_session=True,
                )
                reader = threading.Thread(
                    target=read_messages, args=(process,), daemon=True
                )
                reader.start()
                call(
                    "initialize",
                    {
                        "protocolVersion": 2,
                        "info": {"name": "chapchoo-native-acceptance", "version": "1"},
                        "capabilities": {},
                    },
                )

            def prompt(session, text):
                with condition:
                    offset = len(updates)
                call(
                    "session/prompt",
                    {"sessionId": session, "prompt": [{"type": "text", "text": text}]},
                )

                def finished():
                    running = False
                    for value in updates[offset:]:
                        params = value.get("params", {})
                        update = params.get("update", {})
                        if (
                            params.get("sessionId") != session
                            or update.get("sessionUpdate") != "state_update"
                        ):
                            continue
                        if update.get("state") == "running":
                            running = True
                        if (
                            running
                            and update.get("state") == "idle"
                            and update.get("stopReason") == "end_turn"
                        ):
                            return True
                    return False

                with condition:
                    if not condition.wait_for(finished, timeout=50):
                        raise RuntimeError("Native acceptance is not model completion")

            try:
                instruction = workspace / "AGENTS.md"
                instruction.write_text("PROJECT_ALPHA\n")
                start()
                session = call(
                    "session/new", {"cwd": str(workspace), "mcpServers": []}
                )["sessionId"]
                call(
                    "session/set_title",
                    {"sessionId": session, "title": "Project fixture"},
                )
                stop()
                start()
                call(
                    "session/resume",
                    {"sessionId": session, "cwd": str(workspace), "mcpServers": []},
                )
                prompt(session, "Fixture request one")
                for content, message in (
                    ("PROJECT_BETA\n", "Fixture request two"),
                    ("", "Fixture request three"),
                ):
                    stop()
                    instruction.write_text(content)
                    start()
                    call(
                        "session/resume",
                        {"sessionId": session, "cwd": str(workspace), "mcpServers": []},
                    )
                    prompt(session, message)
                if failures or len(requests) != 3:
                    raise RuntimeError(
                        "Native acceptance request/action count mismatch"
                    )
                for index, (marker, user) in enumerate(
                    (
                        ("PROJECT_ALPHA", "Fixture request one"),
                        ("PROJECT_BETA", "Fixture request two"),
                        ("no longer apply", "Fixture request three"),
                    )
                ):
                    items = requests[index].get("input", [])
                    relevant = [
                        (position, item)
                        for position, item in enumerate(items)
                        if marker in json.dumps(item)
                    ]
                    user_position = next(
                        position
                        for position, item in enumerate(items)
                        if user in json.dumps(item)
                    )
                    if (
                        not relevant
                        or relevant[-1][0] >= user_position
                        or relevant[-1][1].get("role") != "user"
                    ):
                        raise RuntimeError(
                            "Native project instruction role/order mismatch"
                        )
                    if (
                        index == 1
                        and "replace all previously provided AGENTS.md instructions"
                        not in json.dumps(relevant[-1][1])
                    ):
                        raise RuntimeError(
                            "Native project replacement was not observed"
                        )
                return {
                    "versions": versions,
                    "same_native_context": True,
                    "fixture_requests": 3,
                    "native_actions": 0,
                    "alpha_beta_clear_role_and_order": True,
                }
            finally:
                stop()
    finally:
        if process is not None and process.poll() is None:
            os.killpg(process.pid, signal.SIGKILL)
            process.wait(timeout=5)
        server.shutdown()
        server.server_close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--program-root", type=Path, required=True)
    parser.add_argument("--node", default=shutil.which("node"))
    args = parser.parse_args()
    if not args.node:
        raise SystemExit("Node is required")
    print(
        json.dumps(
            run(
                args.program_root.resolve(strict=True),
                str(Path(args.node).resolve(strict=True)),
            )
        )
    )


if __name__ == "__main__":
    main()
