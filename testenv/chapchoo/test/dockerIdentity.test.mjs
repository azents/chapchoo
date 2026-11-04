import assert from "node:assert/strict";
import test from "node:test";
import { DockerFixture } from "../dist/docker.js";

void test("native fixture binds one returned opaque Provider identity independently of its registration key", () => {
  const fixture = new DockerFixture({
    socket: "/unused-fixture.sock",
    image: "unused-fixture-image",
    controlHost: null,
  });
  const registration = fixture.registrationKey;
  assert.throws(() => fixture.bindProvider(""));
  assert.throws(() => fixture.bindProvider("x".repeat(129)));
  fixture.bindProvider("opaque-returned-provider");
  assert.equal(fixture.registrationKey, registration);
  assert.throws(() => fixture.bindProvider("opaque-returned-provider"));
  assert.throws(() => fixture.bindProvider("different-provider"));
});
