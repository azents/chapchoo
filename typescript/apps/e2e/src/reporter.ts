import { lstat, realpath, rm, unlink } from "node:fs/promises";
import { dirname, resolve, sep } from "node:path";
import { privateOutputRoot, runnerOutput } from "./config/index.js";
import { safeMessage, sanitize } from "@chapchoo/testenv/safety";
import type {
  FullConfig,
  FullResult,
  Reporter,
  TestCase,
  TestError,
  TestResult,
} from "@playwright/test/reporter";

function missing(error: unknown): boolean {
  return error instanceof Error && "code" in error && error.code === "ENOENT";
}

/** Only public Reporter APIs; transient runner artifacts stay inside a 0700 parent. */
export default class SafeReporter implements Reporter {
  private privacyFailed = false;
  private pendingCleanup: Promise<void>[] = [];
  printsToStdio(): boolean {
    return true;
  }
  onBegin(config: FullConfig): void {
    if (
      config.projects.some(
        (project) => resolve(project.outputDir) !== resolve(runnerOutput),
      )
    ) {
      throw new Error(
        "Acceptance reporter refuses an unowned output directory",
      );
    }
  }
  private async removeAttachment(path: string): Promise<void> {
    const absolute = resolve(path);
    if (!absolute.startsWith(`${resolve(runnerOutput)}${sep}`)) {
      this.privacyFailed = true;
      return;
    }
    try {
      const root = await realpath(runnerOutput);
      const parent = await realpath(dirname(absolute));
      if (parent !== root && !parent.startsWith(`${root}${sep}`)) {
        this.privacyFailed = true;
        return;
      }
      const stat = await lstat(absolute);
      if (stat.isDirectory()) {
        this.privacyFailed = true;
        return;
      }
      // A symlink is unlinked itself, never followed to an external target.
      await unlink(absolute);
    } catch (error) {
      if (!missing(error)) {
        this.privacyFailed = true;
      }
    }
  }
  onTestEnd(test: TestCase, result: TestResult): void {
    for (const attachment of result.attachments) {
      if (attachment.path) {
        this.pendingCleanup.push(this.removeAttachment(attachment.path));
      }
    }
    // Do not publish inline buffers or artifact paths through reporter forwarding.
    result.attachments.splice(0);
    process.stdout.write(`${result.status}: ${sanitize(test.title)}\n`);
    if (result.status !== "passed" && result.status !== "skipped") {
      for (const error of result.errors) {
        this.onError(error);
      }
    }
  }
  onError(error: TestError): void {
    process.stderr.write(
      `Failure: ${safeMessage(error.message ?? "Test runner failed")}\n`,
    );
    const location = error.stack?.match(
      /\/e2e\/src\/(accounts\.spec\.ts|fixture\.ts):(\d+):(\d+)/,
    );
    if (location) {
      process.stderr.write(
        `Source: ${location[1]}:${location[2]}:${location[3]}\n`,
      );
    }
  }
  onStdOut(): void {
    /* Raw test and child output is never published. */
  }
  onStdErr(): void {
    /* Raw test and child output is never published. */
  }
  private async cleanOwnedOutput(): Promise<void> {
    try {
      const stat = await lstat(privateOutputRoot);
      if (
        !stat.isDirectory() ||
        stat.isSymbolicLink() ||
        (stat.mode & 0o777) !== 0o700 ||
        stat.uid !== process.getuid?.()
      ) {
        this.privacyFailed = true;
      } else {
        // The exact configured child only: intentional safe PNGs live elsewhere.
        await rm(runnerOutput, { recursive: true, force: true });
        try {
          await lstat(runnerOutput);
          this.privacyFailed = true;
        } catch (error) {
          if (!missing(error)) {
            this.privacyFailed = true;
          }
        }
      }
    } catch {
      this.privacyFailed = true;
    }
  }
  async onEnd(result: FullResult): Promise<{ status: FullResult["status"] }> {
    await Promise.all(this.pendingCleanup);
    this.pendingCleanup = [];
    await this.cleanOwnedOutput();
    const status = this.privacyFailed ? "failed" : result.status;
    if (this.privacyFailed) {
      process.stderr.write("Failure: private runner artifact cleanup failed\n");
    }
    process.stdout.write(`Accounts acceptance: ${status}\n`);
    return { status };
  }
  async onExit(): Promise<void> {
    // Public final hook also removes any late runner bookkeeping output.
    await this.cleanOwnedOutput();
    if (this.privacyFailed) {
      throw new Error(
        "Private acceptance output cleanup could not be confirmed",
      );
    }
  }
}
