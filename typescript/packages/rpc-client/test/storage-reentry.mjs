// One retained-volume assertion inside the existing disposable Kubernetes lifecycle.
import { execFile } from "node:child_process";
import { promisify } from "node:util";
const execute = promisify(execFile);

export function storageReentry(kubeconfig, namespace, computerId) {
  if (!kubeconfig || !namespace) {
    throw new Error("Owned cluster storage inputs are required");
  }
  async function kubectl(args) {
    try {
      const result = await execute(
        "kubectl",
        ["--kubeconfig", kubeconfig, "-n", namespace, ...args],
        { timeout: 15000, maxBuffer: 1024 * 1024 },
      );
      return result.stdout;
    } catch {
      throw new Error(
        "Owned cluster storage verification failed; output withheld",
      );
    }
  }
  async function pod() {
    const response = JSON.parse(await kubectl(["get", "pods", "-o", "json"]));
    const candidates = response.items.filter(
      (item) =>
        item.metadata?.annotations?.["chapchoo.io/computer"] === computerId &&
        !item.metadata?.deletionTimestamp &&
        item.status?.phase === "Running",
    );
    if (candidates.length !== 1) {
      throw new Error("Expected one owned current Computer Pod");
    }
    return candidates[0].metadata.name;
  }
  const proof =
    "/workspace/.chapchoo-native/project-source/fsGroup-reentry-proof";
  return {
    async seed() {
      const first = await pod();
      // This test owns this disposable Computer and its sole reserved PVC.
      // Persist the observed fsGroup mode before Stop so drivers that skip a
      // repeated chown still exercise the same restoration boundary.
      await kubectl([
        "exec",
        first,
        "-c",
        "computer-connector",
        "--",
        "sh",
        "-ec",
        `umask 077; mkdir -p /workspace/.chapchoo-native/project-source; printf 'retained-storage-proof' > '${proof}'; chmod 2770 /workspace/.chapchoo-native/project-source`,
      ]);
      return first;
    },
    async verify(previous) {
      const current = await pod();
      if (current === previous) {
        throw new Error("Stop/Start did not replace the physical Computer Pod");
      }
      const observed = await kubectl([
        "exec",
        current,
        "-c",
        "computer-connector",
        "--",
        "sh",
        "-c",
        `test "$(cat '${proof}')" = retained-storage-proof && stat -c '%a' /workspace/.chapchoo-native/project-source`,
      ]);
      if (observed.trim() !== "700") {
        throw new Error(
          "Restored private storage was not admitted with mode 0700",
        );
      }
    },
  };
}
